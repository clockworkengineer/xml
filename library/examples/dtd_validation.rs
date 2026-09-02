use xml_lib::{parse, DtdValidator};

fn main() {
    println!("--- XML DTD Validation Example ---");
    let xml_valid = r#"<?xml version="1.0"?>
<!DOCTYPE note [
  <!ELEMENT note (to,from,heading,body)>
  <!ATTLIST note category CDATA #REQUIRED>
  <!ELEMENT to EMPTY>
  <!ELEMENT from EMPTY>
  <!ELEMENT heading EMPTY>
  <!ELEMENT body EMPTY>
]>
<note category="reminder">
  <to/>
  <from/>
  <heading/>
  <body/>
</note>"#;

    let doc = parse(xml_valid).expect("Parse clean");
    let validator = DtdValidator::new();

    match validator.validate(&doc) {
        Ok(_) => println!("DTD Validation passed clean!"),
        Err(err) => eprintln!("DTD Validation failed: {err}"),
    }

    let xml_invalid = r#"<?xml version="1.0"?>
<!DOCTYPE note [
  <!ATTLIST note category CDATA #REQUIRED>
]>
<note>
  <to/>
</note>"#;

    let doc_inv = parse(xml_invalid).expect("Parse clean");
    match validator.validate(&doc_inv) {
        Ok(_) => println!("Unexpected pass"),
        Err(err) => println!("Validation correctly detected error: {err}"),
    }
}
