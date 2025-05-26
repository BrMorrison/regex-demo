mod interpreter;
mod regex;

use std::env;
use std::fs;
use std::process;
use std::time;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <regex_file> <text_file>", args[0]);
        process::exit(1);
    }

    let regex_prog = regex::bin::parse_bin(&args[1]).unwrap_or_else(|err| {
        eprintln!("Error parsing regex: {err}");
        process::exit(1);
    });

    let search_text = fs::read_to_string(&args[2]).unwrap_or_else(|err| {
        eprintln!("Error reading text file: {err}");
        process::exit(1);
    });

    let start = time::SystemTime::now();
    let mut matches: Vec<&str> = Vec::new();
    for line in search_text.lines() {
        if let Some((_start, _end)) = interpreter::search(&regex_prog, line) {
            //println!("Matched '{}' in '{line}'", &line[start..end]);
            matches.push(line);
        }
    }
    let end = start.elapsed().unwrap();

    println!("{} matches in {} s", matches.len(), (end.as_micros() as f64 / 1_000_000.0));
    for line in matches {
        println!("{}", line);
    }
}
