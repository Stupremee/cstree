mod common;

use common::TestLang;
use cstree::{GreenNodeBuilder, SyntaxKind, SyntaxNode, TextRange};
use lasso::Resolver;

#[derive(Debug)]
enum Element<'s> {
    Node(Vec<Element<'s>>),
    Token(&'s str),
}

fn two_level_tree() -> Element<'static> {
    use Element::*;
    Node(vec![
        Node(vec![Token("0.0"), Token("0.1")]),
        Node(vec![Token("1.0")]),
        Node(vec![Token("2.0"), Token("2.1"), Token("2.2")]),
    ])
}

fn build_tree<D>(root: &Element<'_>) -> (SyntaxNode<TestLang, D>, impl Resolver) {
    let mut builder = GreenNodeBuilder::new();
    build_recursive(root, &mut builder, 0);
    let (node, interner) = builder.finish();
    (SyntaxNode::new_root(node), interner.unwrap())
}

fn build_recursive(root: &Element<'_>, builder: &mut GreenNodeBuilder, mut from: u16) -> u16 {
    match root {
        Element::Node(children) => {
            builder.start_node(SyntaxKind(from));
            for child in children {
                from = build_recursive(child, builder, from + 1);
            }
            builder.finish_node();
        }
        Element::Token(text) => {
            builder.token(SyntaxKind(from), *text);
        }
    }
    from
}

#[test]
fn create() {
    let tree = two_level_tree();
    let (tree, resolver) = build_tree::<()>(&tree);
    assert_eq!(tree.syntax_kind(), SyntaxKind(0));
    assert_eq!(tree.kind(), SyntaxKind(0));
    {
        let leaf1_0 = tree.children().nth(1).unwrap().children_with_tokens().nth(0).unwrap();
        let leaf1_0 = leaf1_0.into_token().unwrap();
        assert_eq!(leaf1_0.syntax_kind(), SyntaxKind(5));
        assert_eq!(leaf1_0.kind(), SyntaxKind(5));
        assert_eq!(leaf1_0.text(&resolver), "1.0");
        assert_eq!(leaf1_0.text_range(), TextRange::at(6.into(), 3.into()));
    }
    {
        let node2 = tree.children().nth(2).unwrap();
        assert_eq!(node2.syntax_kind(), SyntaxKind(6));
        assert_eq!(node2.kind(), SyntaxKind(6));
        assert_eq!(node2.children_with_tokens().count(), 3);
        assert_eq!(node2.text(&resolver), "2.02.12.2");
    }
}

#[test]
fn data() {
    let tree = two_level_tree();
    let (tree, _resolver) = build_tree::<String>(&tree);
    {
        let node2 = tree.children().nth(2).unwrap();
        assert_eq!(*node2.try_set_data("data".into()).unwrap(), "data");
        let data = node2.get_data().unwrap();
        assert_eq!(data.as_str(), "data");
        node2.set_data("payload".into());
        let data = node2.get_data().unwrap();
        assert_eq!(data.as_str(), "payload");
    }
    {
        let node2 = tree.children().nth(2).unwrap();
        assert!(node2.try_set_data("already present".into()).is_err());
        let data = node2.get_data().unwrap();
        assert_eq!(data.as_str(), "payload");
        node2.set_data("new data".into());
    }
    {
        let node2 = tree.children().nth(2).unwrap();
        let data = node2.get_data().unwrap();
        assert_eq!(data.as_str(), "new data");
        node2.clear_data();
        // re-use `data` after node data was cleared
        assert_eq!(data.as_str(), "new data");
    }
    {
        let node2 = tree.children().nth(2).unwrap();
        assert_eq!(node2.get_data(), None);
    }
}