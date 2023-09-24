use cargo_pretty_test::{
    fetch::{cargo_test, parse_cargo_test_output},
    prettify::ICON_NOTATION,
};

fn main() {
    let output = cargo_test();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let tree = parse_cargo_test_output(&stderr, &stdout);
    // println!("{head}\n{tree}\n{detail}\n{ICON_NOTATION}");
    println!("{tree}\n{ICON_NOTATION}");
}
