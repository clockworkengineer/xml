use xml_lib::{parse, XPathEngine};

fn main() {
    println!("--- XPath Functions Example ---");
    let xml = r#"<bookstore>
  <book category="cooking"><title>Everyday Italian</title><price>30.00</price></book>
  <book category="web"><title>Learning XML</title><price>39.95</price></book>
</bookstore>"#;

    let doc = parse(xml).expect("Parse clean");
    let xpath = XPathEngine::new(&doc);

    // count(//book)
    let count = xpath.evaluate("count(//book)", None).unwrap();
    println!("count(//book) = {:?}", count);

    // sum(//price)
    let sum = xpath.evaluate("sum(//price)", None).unwrap();
    println!("sum(//price) = {:?}", sum);

    // concat & substring
    let concat = xpath.evaluate("concat('Total Books: ', count(//book))", None).unwrap();
    println!("concat() = {:?}", concat);

    let round = xpath.evaluate("round(39.95)", None).unwrap();
    println!("round(39.95) = {:?}", round);
}
