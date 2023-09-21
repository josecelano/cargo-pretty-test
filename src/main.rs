use pretty_test::app::make_pretty;
use std::io::{self, Read};
use std::process;

fn main() {
    let mut input = String::new();

    io::stdin()
        .read_to_string(&mut input)
        .expect("Failed to read from stdin");

    if input.is_empty() {
        println!("No input provided. Exiting...");
        process::exit(1);
    }

    if let Some(pretty_output) = make_pretty(&input) {
        println!("{pretty_output}");
    }
}
