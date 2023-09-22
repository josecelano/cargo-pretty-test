use cargo_pretty_test::{app::make_pretty, regex::get_lines_of_tests};
use std::process::Command;

fn main() {
    let output = Command::new("cargo")
        .arg("test")
        .output()
        .expect("`cargo test` failed");
    let text = String::from_utf8_lossy(&output.stdout);
    if let Some(tree) = make_pretty(get_lines_of_tests(&text)) {
        println!("{tree}");
    }
}
