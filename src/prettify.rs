use crate::regex::re;
use colored::Colorize;
use std::{
    borrow::Cow,
    collections::{btree_map::Entry, BTreeMap},
};
use termtree::{GlyphPalette, Tree};

pub type TestTree<'s> = Tree<Cow<'s, str>>;

/// Make the cargo test output pretty.
#[must_use]
pub fn make_pretty<'s, S>(root: S, lines: impl Iterator<Item = &'s str>) -> Option<TestTree<'s>>
where
    S: Into<Cow<'s, str>>,
{
    let mut path = BTreeMap::new();
    for line in lines {
        let cap = re().tree.captures(line)?;
        let mut split = cap.name("split")?.as_str().split("::");
        let status = cap.name("status")?.as_str();
        let next = split.next();
        make_node(split, status, &mut path, next);
    }
    let mut tree = Tree::new(root.into());
    for (name, child) in path {
        make_tree(name, &child, &mut tree);
    }
    Some(tree)
}

#[derive(Debug)]
enum Node<'s> {
    Path(BTreeMap<&'s str, Node<'s>>),
    Status(&'s str),
}

/// Add paths to Node.
fn make_node<'s>(
    mut split: impl Iterator<Item = &'s str>,
    status: &'s str,
    path: &mut BTreeMap<&'s str, Node<'s>>,
    key: Option<&'s str>,
) {
    let Some(key) = key else { return };
    let next = split.next();
    match path.entry(key) {
        Entry::Vacant(empty) => {
            if next.is_some() {
                let mut btree = BTreeMap::new();
                make_node(split, status, &mut btree, next);
                empty.insert(Node::Path(btree));
            } else {
                empty.insert(Node::Status(status));
            }
        }
        Entry::Occupied(mut node) => {
            if let Node::Path(btree) = node.get_mut() {
                make_node(split, status, btree, next);
            }
        }
    }
}

/// Add Node to Tree.
fn make_tree<'s>(root: &'s str, node: &Node<'s>, parent: &mut TestTree<'s>) {
    match node {
        Node::Path(btree) => {
            let mut testtree = Tree::new(root.into());
            for (path, child) in btree {
                make_tree(path, child, &mut testtree);
            }
            parent.push(testtree);
        }
        Node::Status(s) => {
            let status = Status::new(s);
            let testtree = Tree::new(status.set_color(root));
            parent.push(testtree.with_glyphs(status.glyph()));
        }
    }
}

#[derive(Clone, Copy)]
pub enum Status {
    Ok,
    Ignored,
    Failed,
}

impl Status {
    pub fn new(status: &str) -> Status {
        if status.ends_with("ok") {
            // including the case that should panic and did panic
            Status::Ok
        } else if status.starts_with("ignored") {
            Status::Ignored
        } else {
            // including should panic but didn't panic
            Status::Failed
        }
    }

    pub const fn icon(self) -> &'static str {
        match self {
            Status::Ok => "─ ✅ ",
            Status::Ignored => "─ 🔕 ",
            Status::Failed => "─ ❌ ",
        }
    }

    /// Display with a status icon
    pub fn glyph(self) -> GlyphPalette {
        let mut glyph = GlyphPalette::new();
        glyph.item_indent = self.icon();
        glyph
    }

    pub fn set_color(self, s: &str) -> Cow<'_, str> {
        match self {
            Status::Ok => s.into(),
            Status::Ignored => s.bright_black().to_string().into(),
            Status::Failed => s.red().to_string().into(),
        }
    }
}

pub const ICON_NOTATION: &str = "
Icon Notation:
─ ✅ pass (including the case that should panic and did panic)
─ ❌ fail (including the case that should panic but didn't panic)
─ 🔕 ignored (with reason omitted)
";
