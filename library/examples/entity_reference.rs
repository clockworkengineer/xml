use xml_lib::{parse, EntityMapper};

fn main() {
    println!("--- XML Entity Reference Example ---");
    let xml = r#"<data>AT&amp;T &lt;Corp&gt; &quot;Quote&quot; &#65; &#x42;</data>"#;

    let doc = parse(xml).expect("Parse clean");
    let text = doc.get_text_content(doc.root_element_id().unwrap());
    println!("Parsed Raw Content: \"{text}\"");

    let mut mapper = EntityMapper::default();
    let expanded = mapper.expand(&text).expect("Expand entities");
    println!("Expanded Content : \"{expanded}\"");

    // Register Custom Entity
    mapper.register("custom", "Special Product Value");
    let custom_expanded = mapper.expand("Product: &custom;").unwrap();
    println!("Custom Entity    : \"{custom_expanded}\"");
}
