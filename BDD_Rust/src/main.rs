mod input_output;

use input_output::*;

fn main() {
    
    let json_input=read_input();    

    show_input(&json_input);
    input_parse(json_input).unwrap(); // parsing json to extract relevant data
}