use xml_lib::{parse_with_options, ParseOptions};

fn main() {
    println!("--- XML Large Attributes Example ---");
    let mut xml = String::from("<widget ");
    for i in 0..100 {
        xml.push_str(&format!("attr{i}=\"value{i}\" "));
    }
    xml.push_str("/>");

    let mut options = ParseOptions::default();
    options.max_attribute_count = 500;

    let doc = parse_with_options(&xml, options).expect("Parse clean");
    if let Some(root_id) = doc.root_element_id() {
        if let Some(node) = doc.get_node(root_id) {
            if let xml_lib::NodeKind::Element { attributes, .. } = &node.kind {
                println!("Successfully parsed element with {} attributes.", attributes.len());
            }
        }
    }
}
