use cargo_pretty_test::{app::make_pretty, regex::parse_cargo_test};
use std::process::Command;

fn main() {
    let output = Command::new("cargo")
        .arg("test")
        .output()
        .expect("`cargo test` failed");
    let text = String::from_utf8_lossy(&output.stdout);
    let (head, tree, detail) = parse_cargo_test(&text);
    if let Some(tree) = make_pretty(tree.into_iter()) {
        println!("{head}\n{tree}\n{detail}");
    }
}
