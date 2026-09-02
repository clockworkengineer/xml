use xml_lib::{parse, XPathEngine};

fn main() {
    println!("--- XPath Basic Navigation Example ---");
    let xml = r#"<bookstore>
  <book category="cooking">
    <title>Everyday Italian</title>
    <price>30.00</price>
  </book>
  <book category="web">
    <title>Learning XML</title>
    <price>39.95</price>
  </book>
</bookstore>"#;

    let doc = parse(xml).expect("Parse clean");
    let xpath = XPathEngine::new(&doc);

    // Query 1: //book/title
    let titles = xpath.evaluate_nodes("//book/title", None).expect("Evaluate titles");
    println!("Selected Titles (total {}):", titles.len());
    for tid in titles {
        println!("  - {}", doc.get_text_content(tid));
    }

    // Query 2: /bookstore/book[2]/price
    let prices = xpath.evaluate_nodes("/bookstore/book[2]/price", None).expect("Evaluate price");
    if let Some(&pid) = prices.first() {
        println!("Second book price: {}", doc.get_text_content(pid));
    }
}
