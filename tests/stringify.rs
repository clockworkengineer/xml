use xml_lib::{parse, stringify};

#[test]
fn test_roundtrip_stringify() {
    let xml = r#"<data attr="value &amp; &quot;quote&quot;"><item>Text &lt;content&gt;</item></data>"#;
    let doc = parse(xml).expect("Should parse XML");
    let output = stringify(&doc);
    assert!(output.contains("attr=\"value &amp; &quot;quote&quot;\""));
    assert!(output.contains("<item>Text &lt;content&gt;</item>"));
}
