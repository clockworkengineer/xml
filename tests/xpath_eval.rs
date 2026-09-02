use xml_lib::{parse, XPathEngine, XPathValue};

#[test]
fn test_xpath_evaluation() {
    let xml = r#"<catalog>
  <book id="1">
    <title>Rust Programming</title>
    <price>49.99</price>
  </book>
  <book id="2">
    <title>XML Masterclass</title>
    <price>29.99</price>
  </book>
</catalog>"#;

    let doc = parse(xml).expect("Should parse catalog");
    let xpath = XPathEngine::new(&doc);

    let book_nodes = xpath.evaluate_nodes("/catalog/book", None).expect("XPath should succeed");
    assert_eq!(book_nodes.len(), 2);

    let count_val = xpath.evaluate("count(/catalog/book)", None).expect("Count should succeed");
    assert_eq!(count_val, XPathValue::Number(2.0));

    let title_val = xpath.evaluate("/catalog/book[1]/title", None).expect("Title query should succeed");
    if let XPathValue::NodeSet(ns) = title_val {
        assert_eq!(ns.len(), 1);
    } else {
        panic!("Expected NodeSet");
    }
}
