use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::fs::File;
use std::io::prelude::*;

const FILE_PATH: &'static str = "input_data/info.json";

#[derive(Serialize, Deserialize)]
struct Input {

  paths: Vec<PathInformation>, 
}

#[derive(Serialize, Deserialize)]
struct PathInformation{      // struct so save relevant path information

    from: String,
    to: String,
    costs: u16,
}
pub fn input_parse(data:String) -> Result<()> {
    
    // Parse the string of data into a Input object
    let i: Input = serde_json::from_str(&data)?;

    // Do things just like with any other Rust data structure.
    println!("Distance from {} to {} is {}", i.paths[0].from, i.paths[0].to, i.paths[0].costs);
    println!("Distance from {} to {} is {}", i.paths[1].from, i.paths[1].to, i.paths[1].costs);
    // Access parts of the data by indexing with square brackets.

    Ok(())
}

 pub fn read_input()-> String {

  let mut file = File::open(FILE_PATH).expect("could not open file");
  let mut file_input = String::new();
  file.read_to_string(&mut file_input).expect("could not read file");
  return file_input;
}

pub fn show_input(input: &String) {

  println!(" Json Input: \n\n{}", input);
}