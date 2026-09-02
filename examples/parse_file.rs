use xml_lib::{parse, stringify};

fn main() {
    println!("--- XML Parse File Example ---");
    let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<note>
  <to>Tove</to>
  <from>Jani</from>
  <heading>Reminder</heading>
  <body>Don't forget me this weekend!</body>
</note>"#;

    match parse(xml_content) {
        Ok(doc) => {
            println!("Successfully parsed XML document with {} nodes.", doc.len());
            if let Some(root_id) = doc.root_element_id() {
                if let Some(node) = doc.get_node(root_id) {
                    println!("Root element name: <{}>", node.kind.name());
                }
            }
            println!("Serialized Output:\n{}", stringify(&doc));
        }
        Err(err) => {
            eprintln!("Failed to parse XML: {err}");
        }
    }
}
