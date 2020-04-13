use serde_json::{Result, Value};
use serde_json::json;
use std::fs::File;
use std::io::prelude::*;

fn untyped_example(data:String) -> Result<()> {
    
    // Parse the string of data into serde_json::Value.
    let v: Value = serde_json::from_str(&data)?;     // make clear why & operator


    println!("Please call {} at the number {}", v["p1"]["From"], v["To"]);
    // Access parts of the data by indexing with square brackets.

    

    Ok(())
}

fn main() {
    
    let mut file = File::open("input_data/info.json").expect("not god");

    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("schmal");

    println!("Content: \n\n{}", contents);
    untyped_example(contents).unwrap();
}