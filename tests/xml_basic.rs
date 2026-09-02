use xml_lib::{parse, stringify, NodeKind};

#[test]
fn test_basic_parsing() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<root attr="test">
  <child>Hello World</child>
</root>"#;

    let doc = parse(xml).expect("Parsing should succeed");
    assert_eq!(doc.len(), 8);

    let root_elem_id = doc.root_element_id().expect("Should have root element");
    let root_node = doc.get_node(root_elem_id).expect("Root node exists");
    
    if let NodeKind::Element { name, attributes } = &root_node.kind {
        assert_eq!(name, "root");
        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes[0].name, "attr");
        assert_eq!(attributes[0].value, "test");
    } else {
        panic!("Expected Element node");
    }

    let stringified = stringify(&doc);
    assert!(stringified.contains("<root attr=\"test\">"));
    assert!(stringified.contains("<child>Hello World</child>"));
}
