use xml_lib::{parse, XsdValidator};

fn main() {
    println!("--- XSD Basic Validation Example ---");
    let schema = r#"<?xml version="1.0"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="note" type="xs:string"/>
</xs:schema>"#;

    let mut validator = XsdValidator::new();
    validator.parse_schema(schema).unwrap();

    let doc = parse("<note>Hello World</note>").unwrap();
    match validator.validate(&doc) {
        Ok(_) => println!("XSD Validation passed clean!"),
        Err(err) => eprintln!("XSD Validation failed: {err}"),
    }
}
