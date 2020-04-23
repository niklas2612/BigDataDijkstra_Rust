
use raft::prelude::Message;
use slog::*;
use std::collections::HashMap;
use slog::{Drain, Logger};

use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};
use raft::*;
use raft::prelude::*;
use raft::storage::MemStorage;

type ProposeCallback = Box<dyn Fn() + Send>;

enum Msg {
    Propose {
        id: u8,
        cb: ProposeCallback,
    },
    // Here we don't use Raft Message, so use dead_code to
    // avoid the compiler warning.
    #[allow(dead_code)]
    Raft(Message),
}

// A simple example about how to use the Raft library in Rust.
fn main() {
    // Create a storage for Raft, and here we just use a simple memory storage.
    // You need to build your own persistent storage in your production.
    // Please check the Storage trait in src/storage.rs to see how to implement one.
    let storage = MemStorage::default();

    let decorator = slog_term::TermDecorator::new().build(); //add formating to the logs ourputting
    let drain = slog_term::FullFormat::new(decorator).build().fuse(); // make output threadsafe
    let drain = slog_async::Async::new(drain)
        .chan_size(4096)
        .overflow_strategy(slog_async::OverflowStrategy::Block)
        .build()
        .fuse();
    let logger = slog::Logger::root(drain, o!("tag" => format!("[{}]", 2))); // Define the output of the log 
                                        // output maps "tag" -> [2]
                                        
    // Create the configuration for the Raft node.
    let cfg = Config {
        // The unique ID for the Raft node.
        id: 1,
        // Election tick is for how long the follower may campaign again after
        // it doesn't receive any message from the leader.
        election_tick: 10,
        // Heartbeat tick is for how long the leader needs to send
        // a heartbeat to keep alive.
        heartbeat_tick: 3,
        // The max size limits the max size of each appended message. Mostly, 1 MB is enough.
        max_size_per_msg: 1024 * 1024 * 1024,
        // Max inflight msgs that the leader sends messages to follower without
        // receiving ACKs.
        max_inflight_msgs: 256,
        peers: vec![1],
        // The Raft applied index.
        // You need to save your applied index when you apply the committed Raft logs.
        applied: 0,
        ..Default::default()
    };

   // let m= Message::new();
    // Create the Raft node.
    let mut r = RawNode::new(&cfg, storage, vec![]).unwrap();
                             // config, storage, list of peers
    let (sender, receiver) = mpsc::channel();
     // define sender and receiver as asynchronious connection
    // Use another thread to propose a Raft request.
    send_propose(logger.clone(), sender);

    // Loop forever to drive the Raft.
    let mut t = Instant::now(); // starts a timer 
    let mut timeout = Duration::from_millis(100);       // set timeout to 100 ms

    // Use a HashMap to hold the `propose` callbacks.
    let mut cbs = HashMap::new();

    // Loop for checking if Msg was sent after timeout
        loop {                      
        match receiver.recv_timeout(timeout) {
            Ok(Msg::Propose { id, cb }) => {     
                cbs.insert(id, cb);
                println!("{}", id); // replicat raft log for other followers; safe request ID and respective callback function
                r.propose(vec![], vec![id]).unwrap();        // propose: Raft node ansteuern, add request to raft log
                //  Parameters: context, data 
                println!("request sent");
            }

             
            Ok(Msg::Raft(a)) =>
            {
                r.step(a).unwrap();  // State maschine advances (schreitet voran) with message m 
                println!("advances state maschine");
            }
            Err(RecvTimeoutError::Timeout) =>
            {
                println!("timeout");
                ();
            } 
            Err(RecvTimeoutError::Disconnected) => return,
        }

        let d = t.elapsed();
        t = Instant::now();
        if d >= timeout {
            timeout = Duration::from_millis(100);
            // We drive Raft every 100ms.
            r.tick();  // tick internal clock 
            println!("reset clock to stop new timeout");
        } else {
            timeout -= d;
        }
        on_ready(&mut r, &mut cbs);
       
    }
}

fn on_ready(r: &mut RawNode<MemStorage>, cbs: &mut HashMap<u8, ProposeCallback>) {
    if !r.has_ready() {
        return;
    }

    // The Raft is ready, we can do something now.
    let mut ready = r.ready();

    let is_leader = r.raft.leader_id == r.raft.id;
    if is_leader {
        // If the peer is leader, the leader can send messages to other followers ASAP.
        let msgs = ready.messages.drain(..);
        for _msg in msgs {
            // Here we only have one peer, so can ignore this.
        }
    }

    if !raft::is_empty_snap(ready.snapshot()) {
        // This is a snapshot, we need to apply the snapshot at first.
        // when receiving snapshot from leader
        r.mut_store()
            .wl()
            .apply_snapshot(ready.snapshot().clone())
            .unwrap();
    }

    if !ready.entries().is_empty() {
        // Append entries to the Raft log
        // entries that are commited but not pushed in Raft log
        r.mut_store().wl().append(ready.entries()).unwrap();
    }

    if let Some(hs) = ready.hs() {
        // Raft HardState changed, and we need to persist it.
        // changed when note vote for new leader or commit index has increased
        r.mut_store().wl().set_hardstate(hs.clone());
    }

    if !is_leader {
        // If not leader, the follower needs to reply the messages to
        // the leader after appending Raft entries.
        let msgs = ready.messages.drain(..);
        for _msg in msgs {
            // Send messages to other peers.
        }
    }

    if let Some(committed_entries) = ready.committed_entries.take() {
        let mut _last_apply_index = 0;
        for entry in committed_entries {
            // Mostly, you need to save the last apply index to resume applying
            // after restart. Here we just ignore this because we use a Memory storage.
            _last_apply_index = entry.index;

            if entry.data.is_empty() {
                // Emtpy entry, when the peer becomes Leader it will send an empty entry.
                continue;
            }

            if entry.get_entry_type() == EntryType::EntryNormal {
                if let Some(cb) = cbs.remove(entry.data.get(0).unwrap()) {
                    cb();
                }
            }

            // TODO: handle EntryConfChange
        }
    }

    // Advance the Raft
    // inform raw node that the last results where saved
    r.advance(ready);
}

fn send_propose(logger: Logger, sender: mpsc::Sender<Msg>) {
    println!("call prpose");
    // spawn a thread to send message
    thread::spawn(move || {
        //move sendeer and logger into new thread
        // Wait some time and send the request to the Raft.
        thread::sleep(Duration::from_secs(10));   //send request after 10 secs

        let (s1, r1) = mpsc::channel::<u8>();

        info!(logger, "propose a request");

        // Send a command to the Raft, wait for the Raft to apply it
        // and get the result.
        sender
            .send(Msg::Propose {
                id: 1,
                cb: Box::new(move || {
                    s1.send(0).unwrap();
                }),

            })
            .unwrap();  

        let n = r1.recv().unwrap();
        assert_eq!(n, 0);

        info!(logger, "receive the propose callback");
    });
}