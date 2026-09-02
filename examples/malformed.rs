use xml_lib::parse;

fn main() {
    println!("--- XML Malformed Errors Example ---");
    let malformed_xmls = [
        "<note><to>Alice</from></note>", // mismatched tag
        "<note attr=\"unclosed>",        // unclosed attribute
        "<root><child></root>",          // unclosed inner tag
    ];

    for (i, xml) in malformed_xmls.iter().enumerate() {
        match parse(xml) {
            Ok(_) => println!("Case {}: Unexpected pass", i + 1),
            Err(err) => println!("Case {}: Correctly caught syntax error:\n  -> {err}", i + 1),
        }
    }
}
