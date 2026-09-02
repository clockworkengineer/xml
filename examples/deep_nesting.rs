use xml_lib::{parse_with_options, ParseOptions};

fn main() {
    println!("--- XML Deep Nesting Example ---");

    // Generate nested XML: <level1><level2>...</level2></level1>
    let depth = 50;
    let mut xml = String::new();
    for i in 1..=depth {
        xml.push_str(&format!("<level{i}>"));
    }
    xml.push_str("Deep Content");
    for i in (1..=depth).rev() {
        xml.push_str(&format!("</level{i}>"));
    }

    let mut options = ParseOptions::default();
    options.max_nesting_depth = 100;

    match parse_with_options(&xml, options) {
        Ok(doc) => println!("Successfully parsed document with nesting depth {depth}. Total nodes: {}", doc.len()),
        Err(err) => eprintln!("Parse failed: {err}"),
    }

    // Now attempt with lower depth limit
    let mut strict_options = ParseOptions::default();
    strict_options.max_nesting_depth = 20;

    match parse_with_options(&xml, strict_options) {
        Ok(_) => println!("Unexpected success"),
        Err(err) => println!("Correctly rejected deep nesting with security limit: {err}"),
    }
}
