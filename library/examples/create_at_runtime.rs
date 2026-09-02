use xml_lib::{stringify, Attribute, Document, NodeKind};

fn main() {
    println!("--- XML Create At Runtime Example ---");
    let mut doc = Document::new();

    // Declaration
    let decl_id = doc.add_node(NodeKind::Declaration {
        version: "1.0".into(),
        encoding: Some("UTF-8".into()),
        standalone: Some(true),
    });
    doc.set_declaration_id(decl_id);

    // Root Element
    let root_elem_id = doc.add_node(NodeKind::Element {
        name: "library".into(),
        attributes: vec![Attribute {
            name: "location".into(),
            value: "Main Campus".into(),
        }],
    });
    let root_id = doc.root_id().unwrap();
    doc.append_child(root_id, root_elem_id).unwrap();

    // Book Element
    let book_id = doc.add_node(NodeKind::Element {
        name: "book".into(),
        attributes: vec![Attribute {
            name: "isbn".into(),
            value: "978-0-306-40615-7".into(),
        }],
    });
    doc.append_child(root_elem_id, book_id).unwrap();

    // Title Element & Text
    let title_id = doc.add_node(NodeKind::Element {
        name: "title".into(),
        attributes: Vec::new(),
    });
    let title_text_id = doc.add_node(NodeKind::Text("Rust Programming Language".into()));
    doc.append_child(title_id, title_text_id).unwrap();
    doc.append_child(book_id, title_id).unwrap();

    println!("Constructed DOM Document:\n{}", stringify(&doc));
}
