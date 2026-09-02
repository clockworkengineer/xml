use xml_lib::{parse_bytes, XmlSource};

fn main() {
    println!("--- XML Encoding & BOM Example ---");

    // UTF-8 with BOM bytes
    let utf8_bom_bytes = [0xEF, 0xBB, 0xBF, b'<', b'r', b'o', b'o', b't', b'>', b'O', b'K', b'<', b'/', b'r', b'o', b'o', b't', b'>'];
    let source = XmlSource::from_bytes(&utf8_bom_bytes).expect("BOM detection");
    println!("Detected BOM format: {:?}", source.format());

    let doc = parse_bytes(&utf8_bom_bytes).expect("Parse BOM bytes");
    println!("Successfully parsed XML with BOM detection: {} nodes", doc.len());
}
