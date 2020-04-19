use std::env;
use std::process;
use wordfindgen::Config;

// The main entry point to the program
// This fn gathers the command line args into a config struct
// Then calls the fn that generates the puzzle
//
// The first arg should be the name of a text file with the words to place in the puzzle
// If a second argument is included the puzzle will be more difficult,
//    by also placing words right to left (backwards)
//
// The output csv can be opened, formatted, and printed from any spreadsheet program
// It works best if the puzzle grid characters are centered vertically and horizontally with
// borders drawn on all sides
fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("There is a problem with your command line: {}", err);
        process::exit(1);
    });
    
    if let Err(e) = wordfindgen::run(config) {
        eprintln!("There was an error generating: {}", e);
        process::exit(1);
    }
    
    println!("Done!");
}
