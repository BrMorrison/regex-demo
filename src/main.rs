mod interpreter;
mod regex;

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <regex_file> <text_file>", args[0]);
        process::exit(1);
    }

    let regex_prog = regex::parse_regex(&args[1]).unwrap_or_else(|err| {
        eprintln!("Error parsing regex: {err}");
        process::exit(1);
    });

    let search_text = fs::read_to_string(&args[2]).unwrap_or_else(|err| {
        eprintln!("Error reading text file: {err}");
        process::exit(1);
    });

    for line in search_text.lines() {
        if interpreter::search(&regex_prog, line) {
            println!("{}", line);
        }
    }
}
