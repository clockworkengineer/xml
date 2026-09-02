use xml_lib::{parse, DtdValidator};

#[test]
fn test_dtd_validation() {
    let xml = r#"<?xml version="1.0"?>
<!DOCTYPE note [
  <!ELEMENT note (to,from,heading,body)>
  <!ATTLIST note category CDATA #REQUIRED>
  <!ELEMENT to EMPTY>
]>
<note category="reminder">
  <to/>
</note>"#;

    let doc = parse(xml).expect("Should parse XML with DTD");
    let dtd = DtdValidator::new();
    assert!(dtd.validate(&doc).is_ok());
}
