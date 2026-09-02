use xml_lib::{parse, XPathEngine, XPathValue};

const BOOKSTORE_XML: &str = r#"<?xml version="1.0"?>
<bookstore>
  <book category="cooking">
    <title lang="en">Everyday Italian</title>
    <author>Giada De Laurentiis</author>
    <year>2005</year>
    <price>30.00</price>
  </book>
  <book category="children">
    <title lang="en">Harry Potter</title>
    <author>J K. Rowling</author>
    <year>2005</year>
    <price>29.99</price>
  </book>
  <book category="web">
    <title lang="en">XQuery Kick Start</title>
    <author>James McGovern</author>
    <author>Per Bothner</author>
    <year>2003</year>
    <price>49.99</price>
  </book>
  <book category="web">
    <title lang="en">Learning XML</title>
    <author>Erik T. Ray</author>
    <year>2003</year>
    <price>39.95</price>
  </book>
</bookstore>"#;

#[test]
fn test_xpath_basic_navigation() {
    let doc = parse(BOOKSTORE_XML).expect("Bookstore parse clean");
    let xpath = XPathEngine::new(&doc);

    // /bookstore
    let root_nodes = xpath.evaluate_nodes("/bookstore", None).expect("Select root");
    assert_eq!(root_nodes.len(), 1);

    // //book
    let all_books = xpath.evaluate_nodes("//book", None).expect("Select all books");
    assert_eq!(all_books.len(), 4);

    // /bookstore/book
    let direct_books = xpath.evaluate_nodes("/bookstore/book", None).expect("Direct books");
    assert_eq!(direct_books.len(), 4);

    // //book/title
    let titles = xpath.evaluate_nodes("//book/title", None).expect("Book titles");
    assert_eq!(titles.len(), 4);

    // /bookstore/*
    let root_children = xpath.evaluate_nodes("/bookstore/*", None).expect("Root elements");
    assert_eq!(root_children.len(), 4);
}

#[test]
fn test_xpath_predicates_and_functions() {
    let doc = parse(BOOKSTORE_XML).expect("Bookstore parse clean");
    let xpath = XPathEngine::new(&doc);

    // count(//book)
    let count = xpath.evaluate("count(//book)", None).unwrap();
    assert_eq!(count, XPathValue::Number(4.0));

    // count(//author) -> 5 authors (3rd book has 2)
    let author_count = xpath.evaluate("count(//author)", None).unwrap();
    assert_eq!(author_count, XPathValue::Number(5.0));

    // //book[@category='cooking']
    let cooking = xpath.evaluate_nodes("//book[@category='cooking']", None).unwrap();
    assert_eq!(cooking.len(), 1);

    // //book[@category='web']
    let web = xpath.evaluate_nodes("//book[@category='web']", None).unwrap();
    assert_eq!(web.len(), 2);

    // contains
    let contains_res = xpath.evaluate("contains(/bookstore/book[1]/title, 'Italian')", None).unwrap();
    assert_eq!(contains_res, XPathValue::Boolean(true));
}

#[test]
fn test_xpath_syntax_and_errors() {
    let doc = parse(BOOKSTORE_XML).expect("Parse clean");
    let xpath = XPathEngine::new(&doc);

    assert!(xpath.evaluate("//book[", None).is_err());
    assert!(xpath.evaluate("unknownFunction()", None).is_err());
    assert!(xpath.evaluate("", None).is_err());

    let empty = xpath.evaluate_nodes("//nonexistent", None).unwrap();
    assert!(empty.is_empty());
}
