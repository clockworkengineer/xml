use xml_lib::{parse, XPathEngine};

fn main() {
    println!("--- XPath Predicates Example ---");
    let xml = r#"<bookstore>
  <book category="cooking"><title>Everyday Italian</title></book>
  <book category="children"><title>Harry Potter</title></book>
  <book category="web"><title>Learning XML</title></book>
</bookstore>"#;

    let doc = parse(xml).expect("Parse clean");
    let xpath = XPathEngine::new(&doc);

    // Attribute Predicate: //book[@category='web']
    let web_books = xpath.evaluate_nodes("//book[@category='web']", None).unwrap();
    println!("Web books count: {}", web_books.len());

    // Position Predicate: //book[1]
    let first_book = xpath.evaluate_nodes("//book[1]", None).unwrap();
    println!("First book selected ID: {:?}", first_book.first());
}
