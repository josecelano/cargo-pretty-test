use cargo_pretty_test::app::make_pretty;
use regex_lite::Regex;
use std::process::Command;

fn main() {
    let output = Command::new("cargo")
        .arg("test")
        .output()
        .expect("`cargo test` failed");
    let text = String::from_utf8_lossy(&output.stdout);
    let re = Regex::new(r"(?m)^test \S+ \.\.\. \S+$").expect("regex pattern error");
    if let Some(tree) = make_pretty(re.find_iter(&text).map(|m| m.as_str())) {
        println!("{tree}");
    }
}
