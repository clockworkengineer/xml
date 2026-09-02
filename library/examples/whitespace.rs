use xml_lib::{parse, NodeKind};

fn main() {
    println!("--- XML Whitespace Handling Example ---");
    let xml = "<root>\n  <item>  spaced content  </item>\n</root>";

    let doc = parse(xml).expect("Parse clean");
    let root_id = doc.root_element_id().unwrap();

    let children = doc.get_children(root_id);
    println!("Total direct children of <root> (including whitespace text nodes): {}", children.len());

    let element_children: Vec<_> = children
        .into_iter()
        .filter(|&c_id| doc.get_node(c_id).map_or(false, |c| matches!(c.kind, NodeKind::Element { .. })))
        .collect();

    println!("Element-only children count: {}", element_children.len());
}
