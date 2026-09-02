use xml_lib::{stringify, Attribute, Document, NodeKind};

fn main() {
    println!("--- Fibonacci XML Document Generation Example ---");

    let mut doc = Document::new();
    let root_elem_id = doc.add_node(NodeKind::Element {
        name: "fibonacci_sequence".into(),
        attributes: Vec::new(),
    });
    let root_id = doc.root_id().unwrap();
    doc.append_child(root_id, root_elem_id).unwrap();

    let mut a: u64 = 0;
    let mut b: u64 = 1;

    for index in 0..15 {
        let term_elem_id = doc.add_node(NodeKind::Element {
            name: "term".into(),
            attributes: vec![Attribute {
                name: "index".into(),
                value: index.to_string(),
            }],
        });
        let val_text_id = doc.add_node(NodeKind::Text(a.to_string()));
        doc.append_child(term_elem_id, val_text_id).unwrap();
        doc.append_child(root_elem_id, term_elem_id).unwrap();

        let next = a + b;
        a = b;
        b = next;
    }

    println!("Generated Fibonacci XML Output:\n{}", stringify(&doc));
}
