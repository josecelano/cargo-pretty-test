use cargo_pretty_test::{
    fetch::cargo_test,
    prettify::{make_pretty, ICON_NOTATION},
    regex::{parse_cargo_test_output, ParsedCargoTestOutput},
};

fn main() {
    let output = cargo_test();
    let text = String::from_utf8_lossy(&output.stdout);
    let ParsedCargoTestOutput { head, tree, detail } = parse_cargo_test_output(&text);
    if let Some(tree) = make_pretty(tree.into_iter()) {
        println!("{head}\n{tree}\n{detail}\n{ICON_NOTATION}");
    }
}
