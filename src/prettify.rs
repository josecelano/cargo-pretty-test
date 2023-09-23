use std::collections::{btree_map::Entry, BTreeMap};
use termtree::{GlyphPalette, Tree};

/// Make the cargo test output pretty.
#[must_use]
pub fn make_pretty<'s>(ty: &'s str, lines: impl Iterator<Item = &'s str>) -> Option<Tree<&'s str>> {
    let mut path = BTreeMap::new();
    for line in lines {
        let mut iter = line.trim().splitn(3, ' ');
        let mut split = iter.nth(1)?.split("::");
        let next = split.next();
        let status = iter.next()?;
        make_node(split, status, &mut path, next);
    }
    let mut tree = Tree::new(ty);
    for (root, child) in path {
        make_tree(root, &child, &mut tree);
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
fn make_tree<'s>(root: &'s str, node: &Node<'s>, parent: &mut Tree<&'s str>) {
    match node {
        Node::Path(btree) => {
            let mut t = Tree::new(root);
            for (path, child) in btree {
                make_tree(path, child, &mut t);
            }
            parent.push(t);
        }
        Node::Status(s) => {
            parent.push(Tree::new(root).with_glyphs(set_status(s)));
        }
    }
}

/// Display with a status icon
fn set_status(status: &str) -> GlyphPalette {
    let mut glyph = GlyphPalette::new();
    glyph.item_indent = if status.ends_with("ok") {
        // including the case that should panic and did panic
        "─ ✅ "
    } else if status.starts_with("... ignored") {
        "─ 🔕 "
    } else {
        // including should panic but didn't panic
        "─ ❌ "
    };
    glyph
}

pub const ICON_NOTATION: &str = "
Icon Notation:
─ ✅ pass (including the case that should panic and did panic)
─ ❌ fail (including the case that should panic but didn't panic)
─ 🔕 ignored (with reason omitted)
";
