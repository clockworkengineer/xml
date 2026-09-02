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

#[test]
fn test_xpath_attribute_predicates() {
    let xml = r#"<inventory>
  <item id="101" category="electronics">Laptop</item>
  <item id="102" category="books">Rust Book</item>
</inventory>"#;

    let doc = parse(xml).expect("Should parse inventory");
    let xpath = XPathEngine::new(&doc);

    let result = xpath.evaluate_nodes("/inventory/item[@category='books']", None)
        .expect("Attribute predicate should succeed");
    assert_eq!(result.len(), 1);

    let id_query = xpath.evaluate_nodes("//item[@id='101']", None)
        .expect("Deep search with attribute predicate should succeed");
    assert_eq!(id_query.len(), 1);
}

#[test]
fn test_xpath_functions() {
    let xml = r#"<store>
  <product>Widget A</product>
  <product>Gadget B</product>
</store>"#;

    let doc = parse(xml).expect("Should parse store");
    let xpath = XPathEngine::new(&doc);

    let count = xpath.evaluate("count(//product)", None).expect("count() should work");
    assert_eq!(count, XPathValue::Number(2.0));

    let contains_res = xpath.evaluate("contains(/store/product, 'Widget')", None)
        .expect("contains() should work");
    assert_eq!(contains_res, XPathValue::Boolean(true));
}

#[test]
fn test_xpath_syntax_error() {
    let xml = r#"<root><child/></root>"#;
    let doc = parse(xml).expect("Should parse");
    let xpath = XPathEngine::new(&doc);

    let invalid = xpath.evaluate("/root/[invalid", None);
    assert!(invalid.is_err(), "Invalid XPath syntax should return an error");
}
