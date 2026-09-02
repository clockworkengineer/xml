use xml_lib::{
    Attribute, Document, NodeKind, parse, stringify,
    EntityMapper, DtdValidator,
};

#[test]
fn test_element_node_creation_and_attributes() {
    let mut doc = Document::new();
    let elem_id = doc.add_node(NodeKind::Element {
        name: "test".to_string(),
        attributes: vec![
            Attribute::new("attr1", "value1"),
            Attribute::new("attr2", "value2"),
            Attribute::new("attr3", "value3"),
        ],
    });

    let node = doc.get_node(elem_id).expect("Element node exists");
    assert_eq!(node.kind.name(), "test");
    if let NodeKind::Element { name, attributes } = &node.kind {
        assert_eq!(name, "test");
        assert_eq!(attributes.len(), 3);
        assert_eq!(attributes[0].name, "attr1");
        assert_eq!(attributes[0].value, "value1");
        assert_eq!(attributes[1].name, "attr2");
        assert_eq!(attributes[1].value, "value2");
        assert_eq!(attributes[2].name, "attr3");
        assert_eq!(attributes[2].value, "value3");
    } else {
        panic!("Expected NodeKind::Element");
    }
}

#[test]
fn test_element_node_with_children() {
    let mut doc = Document::new();
    let parent_id = doc.add_node(NodeKind::Element {
        name: "element".to_string(),
        attributes: vec![],
    });
    let child_id = doc.add_node(NodeKind::Text("child content".to_string()));
    doc.append_child(parent_id, child_id).unwrap();

    let children = doc.get_children(parent_id);
    assert_eq!(children.len(), 1);
    assert_eq!(children[0], child_id);

    let child_node = doc.get_node(child_id).unwrap();
    assert_eq!(child_node.kind, NodeKind::Text("child content".to_string()));
    assert_eq!(doc.get_text_content(parent_id), "child content");
}

#[test]
fn test_element_xml_parsing_integration() {
    let xml = r#"<?xml version="1.0"?>
<root><element attr='1'>data</element></root>"#;
    let doc = parse(xml).expect("XML parse clean");

    let root_elem_id = doc.root_element_id().expect("Has root element");
    let root_node = doc.get_node(root_elem_id).unwrap();
    assert_eq!(root_node.kind.name(), "root");

    let children = doc.get_children(root_elem_id);
    assert_eq!(children.len(), 1);

    let elem_id = children[0];
    let elem_node = doc.get_node(elem_id).unwrap();
    assert_eq!(elem_node.kind.name(), "element");

    if let NodeKind::Element { name, attributes } = &elem_node.kind {
        assert_eq!(name, "element");
        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes[0].name, "attr");
        assert_eq!(attributes[0].value, "1");
    } else {
        panic!("Expected Element node");
    }

    assert_eq!(doc.get_text_content(elem_id), "data");
}

#[test]
fn test_cdata_node_creation_and_attributes() {
    let mut doc = Document::new();
    
    // Default / empty CDATA
    let empty_cdata = doc.add_node(NodeKind::CData("".to_string()));
    assert_eq!(doc.get_node(empty_cdata).unwrap().kind, NodeKind::CData("".to_string()));

    // CDATA with escaped content
    let cdata_id = doc.add_node(NodeKind::CData("&apos;Test&apos;".to_string()));
    let node = doc.get_node(cdata_id).unwrap();
    if let NodeKind::CData(val) = &node.kind {
        assert_eq!(val, "&apos;Test&apos;");
    } else {
        panic!("Expected CData");
    }

    // Large content
    let large_text = "x".repeat(10000);
    let large_id = doc.add_node(NodeKind::CData(large_text.clone()));
    if let NodeKind::CData(val) = &doc.get_node(large_id).unwrap().kind {
        assert_eq!(val, &large_text);
    }

    // Special characters
    let special = "<>&\"'\n\t";
    let special_id = doc.add_node(NodeKind::CData(special.to_string()));
    if let NodeKind::CData(val) = &doc.get_node(special_id).unwrap().kind {
        assert_eq!(val, special);
    }

    // Embedded CDATA end marker
    let tricky = "]]> inside CDATA";
    let tricky_id = doc.add_node(NodeKind::CData(tricky.to_string()));
    if let NodeKind::CData(val) = &doc.get_node(tricky_id).unwrap().kind {
        assert_eq!(val, tricky);
    }

    // Unicode characters
    let unicode = "\u{1D11E}\u{1D122}\u{1D12B}"; // 𝄞𝄢𝄫
    let unicode_id = doc.add_node(NodeKind::CData(unicode.to_string()));
    if let NodeKind::CData(val) = &doc.get_node(unicode_id).unwrap().kind {
        assert_eq!(val, unicode);
    }
}

#[test]
fn test_cdata_stringify_output() {
    let mut doc = Document::new();
    let cdata_id = doc.add_node(NodeKind::CData("stringify test".to_string()));
    let root_id = doc.root_id().unwrap();
    doc.append_child(root_id, cdata_id).unwrap();

    let output = stringify(&doc);
    assert!(output.contains("<![CDATA[stringify test]]>"));
}

#[test]
fn test_comment_node_creation_and_attributes() {
    let mut doc = Document::new();

    // Default / empty comment
    let empty_id = doc.add_node(NodeKind::Comment("".to_string()));
    assert_eq!(doc.get_node(empty_id).unwrap().kind, NodeKind::Comment("".to_string()));

    // Comment node
    let comment_id = doc.add_node(NodeKind::Comment("This is a test comment.".to_string()));
    if let NodeKind::Comment(val) = &doc.get_node(comment_id).unwrap().kind {
        assert_eq!(val, "This is a test comment.");
    } else {
        panic!("Expected Comment node");
    }

    // Large comment
    let large_text = "x".repeat(10000);
    let large_id = doc.add_node(NodeKind::Comment(large_text.clone()));
    if let NodeKind::Comment(val) = &doc.get_node(large_id).unwrap().kind {
        assert_eq!(val, &large_text);
    }

    // Special characters
    let special = "<>&\"'\n\t";
    let special_id = doc.add_node(NodeKind::Comment(special.to_string()));
    if let NodeKind::Comment(val) = &doc.get_node(special_id).unwrap().kind {
        assert_eq!(val, special);
    }

    // Embedded dashes
    let tricky = "-- inside comment";
    let tricky_id = doc.add_node(NodeKind::Comment(tricky.to_string()));
    if let NodeKind::Comment(val) = &doc.get_node(tricky_id).unwrap().kind {
        assert_eq!(val, tricky);
    }

    // Unicode characters
    let unicode = "\u{1D11E}\u{1D122}\u{1D12B}";
    let unicode_id = doc.add_node(NodeKind::Comment(unicode.to_string()));
    if let NodeKind::Comment(val) = &doc.get_node(unicode_id).unwrap().kind {
        assert_eq!(val, unicode);
    }
}

#[test]
fn test_comment_stringify_output() {
    let mut doc = Document::new();
    let comment_id = doc.add_node(NodeKind::Comment("stringify test".to_string()));
    let root_id = doc.root_id().unwrap();
    doc.append_child(root_id, comment_id).unwrap();

    let output = stringify(&doc);
    assert!(output.contains("<!--stringify test-->"));
}

#[test]
fn test_processing_instruction_node() {
    let mut doc = Document::new();

    // Standard PI
    let pi_id = doc.add_node(NodeKind::ProcessingInstruction {
        target: "xml-stylesheet".to_string(),
        data: "type='text/xsl' href='style.xsl'".to_string(),
    });

    let node = doc.get_node(pi_id).unwrap();
    if let NodeKind::ProcessingInstruction { target, data } = &node.kind {
        assert_eq!(target, "xml-stylesheet");
        assert_eq!(data, "type='text/xsl' href='style.xsl'");
    } else {
        panic!("Expected ProcessingInstruction");
    }

    // Empty target & data
    let empty_pi = doc.add_node(NodeKind::ProcessingInstruction {
        target: "".to_string(),
        data: "".to_string(),
    });
    if let NodeKind::ProcessingInstruction { target, data } = &doc.get_node(empty_pi).unwrap().kind {
        assert_eq!(target, "");
        assert_eq!(data, "");
    }

    // Special characters
    let special_pi = doc.add_node(NodeKind::ProcessingInstruction {
        target: "xml-stylesheet-π".to_string(),
        data: "type='text/xsl' href='style.xsl' & π".to_string(),
    });
    if let NodeKind::ProcessingInstruction { target, data } = &doc.get_node(special_pi).unwrap().kind {
        assert_eq!(target, "xml-stylesheet-π");
        assert_eq!(data, "type='text/xsl' href='style.xsl' & π");
    }

    // Long params
    let long_params = "a".repeat(1000);
    let long_pi = doc.add_node(NodeKind::ProcessingInstruction {
        target: "xml-stylesheet".to_string(),
        data: long_params.clone(),
    });
    if let NodeKind::ProcessingInstruction { data, .. } = &doc.get_node(long_pi).unwrap().kind {
        assert_eq!(data, &long_params);
    }
}

#[test]
fn test_declaration_node() {
    let mut doc = Document::new();

    let decl_id = doc.add_node(NodeKind::Declaration {
        version: "1.0".to_string(),
        encoding: Some("UTF-8".to_string()),
        standalone: Some(true),
    });
    doc.set_declaration_id(decl_id);

    assert_eq!(doc.declaration_id(), Some(decl_id));
    let decl_node = doc.get_node(decl_id).unwrap();
    if let NodeKind::Declaration { version, encoding, standalone } = &decl_node.kind {
        assert_eq!(version, "1.0");
        assert_eq!(encoding.as_deref(), Some("UTF-8"));
        assert_eq!(*standalone, Some(true));
    } else {
        panic!("Expected Declaration");
    }

    // Empty declaration fields
    let empty_decl = doc.add_node(NodeKind::Declaration {
        version: "".to_string(),
        encoding: None,
        standalone: None,
    });
    if let NodeKind::Declaration { version, encoding, standalone } = &doc.get_node(empty_decl).unwrap().kind {
        assert_eq!(version, "");
        assert_eq!(*encoding, None);
        assert_eq!(*standalone, None);
    }

    // Special characters
    let special = "<>&\"'\n\t";
    let special_decl = doc.add_node(NodeKind::Declaration {
        version: special.to_string(),
        encoding: Some(special.to_string()),
        standalone: Some(false),
    });
    if let NodeKind::Declaration { version, encoding, .. } = &doc.get_node(special_decl).unwrap().kind {
        assert_eq!(version, special);
        assert_eq!(encoding.as_deref(), Some(special));
    }
}

#[test]
fn test_declaration_stringify_output() {
    let mut doc = Document::new();
    let decl_id = doc.add_node(NodeKind::Declaration {
        version: "1.0".to_string(),
        encoding: Some("UTF-8".to_string()),
        standalone: Some(true),
    });
    let prolog_id = doc.prolog_id().unwrap();
    doc.append_child(prolog_id, decl_id).unwrap();
    doc.set_declaration_id(decl_id);

    let output = stringify(&doc);
    assert!(output.contains(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#));
}

#[test]
fn test_content_text_node() {
    let mut doc = Document::new();

    let text_id = doc.add_node(NodeKind::Text("This is test content.".to_string()));
    if let NodeKind::Text(val) = &doc.get_node(text_id).unwrap().kind {
        assert_eq!(val, "This is test content.");
    } else {
        panic!("Expected Text node");
    }

    // Append text by updating NodeKind::Text
    if let Some(node) = doc.get_node_mut(text_id) {
        if let NodeKind::Text(val) = &mut node.kind {
            val.push_str(" More content.");
        }
    }
    assert_eq!(doc.get_text_content(text_id), "This is test content. More content.");

    // Unicode characters
    let unicode = "\u{1D11E}\u{1D122}\u{1D12B}";
    let unicode_id = doc.add_node(NodeKind::Text(unicode.to_string()));
    assert_eq!(doc.get_text_content(unicode_id), unicode);
}

#[test]
fn test_prolog_node_and_children() {
    let mut doc = Document::new();
    let prolog_id = doc.prolog_id().expect("Document has prolog");
    assert_eq!(doc.get_node(prolog_id).unwrap().kind, NodeKind::Prolog);

    let decl_id = doc.add_node(NodeKind::Declaration {
        version: "1.0".to_string(),
        encoding: Some("UTF-8".to_string()),
        standalone: Some(true),
    });
    doc.append_child(prolog_id, decl_id).unwrap();

    let children = doc.get_children(prolog_id);
    assert_eq!(children.len(), 1);
    assert_eq!(children[0], decl_id);
}

#[test]
fn test_prolog_xml_integration() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<root></root>"#;
    let doc = parse(xml).expect("XML parse clean");

    let prolog_id = doc.prolog_id().expect("Has prolog");
    let children = doc.get_children(prolog_id);
    assert!(!children.is_empty());
}

#[test]
fn test_root_node_and_children() {
    let mut doc = Document::new();
    let root_container_id = doc.root_id().expect("Document has root container");
    assert_eq!(doc.get_node(root_container_id).unwrap().kind, NodeKind::Root);

    let root_elem_id = doc.add_node(NodeKind::Element {
        name: "root".to_string(),
        attributes: vec![Attribute::new("attr1", "1")],
    });
    doc.append_child(root_container_id, root_elem_id).unwrap();

    assert_eq!(doc.root_element_id(), Some(root_elem_id));
    let children = doc.get_children(root_container_id);
    assert_eq!(children.len(), 1);
    assert_eq!(children[0], root_elem_id);
}

#[test]
fn test_root_xml_integration() {
    let xml = r#"<?xml version="1.0"?>
<root attr1='1'><child>data</child></root>"#;
    let doc = parse(xml).expect("XML parse clean");

    let root_elem_id = doc.root_element_id().expect("Root element present");
    let root_node = doc.get_node(root_elem_id).unwrap();
    assert_eq!(root_node.kind.name(), "root");

    if let NodeKind::Element { attributes, .. } = &root_node.kind {
        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes[0].name, "attr1");
        assert_eq!(attributes[0].value, "1");
    }

    let children = doc.get_children(root_elem_id);
    assert_eq!(children.len(), 1);
    let child_node = doc.get_node(children[0]).unwrap();
    assert_eq!(child_node.kind.name(), "child");
    assert_eq!(doc.get_text_content(children[0]), "data");
}

#[test]
fn test_dtd_node_creation_and_elements() {
    let validator = DtdValidator::new();
    assert!(validator.validate(&Document::new()).is_ok());
}

#[test]
fn test_entity_reference_node() {
    let mut doc = Document::new();

    // Entity Reference Node
    let entity_id = doc.add_node(NodeKind::EntityReference("foo".to_string()));
    let node = doc.get_node(entity_id).unwrap();
    assert_eq!(node.kind, NodeKind::EntityReference("foo".to_string()));
    assert_eq!(node.kind.name(), "foo");

    // EntityMapper resolution
    let mut mapper = EntityMapper::default();
    mapper.register("foo", "bar");
    assert_eq!(mapper.expand("&foo;").unwrap(), "bar");
    assert!(mapper.expand("&unknown;").is_err());
}

#[test]
fn test_variant_node_types() {
    let mut doc = Document::new();

    let prolog = doc.add_node(NodeKind::Prolog);
    let elem = doc.add_node(NodeKind::Element { name: "test".to_string(), attributes: vec![] });
    let text = doc.add_node(NodeKind::Text("text".to_string()));

    assert_eq!(doc.get_node(prolog).unwrap().kind.name(), "#prolog");
    assert_eq!(doc.get_node(elem).unwrap().kind.name(), "test");
    assert_eq!(doc.get_node(text).unwrap().kind.name(), "#text");

    // Parent / Child relationship
    doc.append_child(elem, text).unwrap();
    assert_eq!(doc.get_children(elem), vec![text]);
    assert_eq!(doc.parent_id(text), Some(elem));
}
