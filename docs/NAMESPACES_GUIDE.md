# XML Namespaces 1.0/1.1 Specification Guide

This document provides a comprehensive technical guide to the XML Namespaces subsystem in `xml_lib_rust`, covering W3C Namespaces in XML 1.0 (Third Edition) and 1.1 compliance, scoping rules, prefix lookups, and practical examples.

---

## Table of Contents

1. [Introduction to XML Namespaces](#1-introduction-to-xml-namespaces)
2. [QNames, Prefixes, and Local Names](#2-qnames-prefixes-and-local-names)
3. [Hierarchical Scoping & Inheritance](#3-hierarchical-scoping--inheritance)
4. [DOM Inspection & Lookup APIs](#4-dom-inspection--lookup-apis)
5. [Namespace-Aware Querying](#5-namespace-aware-querying)
6. [Streaming & Serialization with Namespaces](#6-streaming--serialization-with-namespaces)
7. [Real-World Examples (SOAP, SVG, Atom)](#7-real-world-examples)

---

## 1. Introduction to XML Namespaces

XML Namespaces differentiate elements and attributes with identical local names that belong to different vocabularies, preventing naming collisions when combining multiple schemas into a single document.

A namespace is identified by an **Internationalized Resource Identifier (IRI)** or **Uniform Resource Identifier (URI)** (e.g. `http://schemas.xmlsoap.org/soap/envelope/`). Because full URIs are too verbose to prefix every tag, XML binds short **prefixes** (e.g. `soap`) to URIs using special `xmlns` attribute declarations.

```xml
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
               xmlns:m="http://example.org/calculator">
  <soap:Body>
    <m:Add>
      <m:val>5</m:val>
    </m:Add>
  </soap:Body>
</soap:Envelope>
```

---

## 2. QNames, Prefixes, and Local Names

In `xml_lib_rust`, qualified names are represented by the [`QName`](../library/src/namespace/mod.rs) struct:

```rust
pub struct QName<'a> {
    pub prefix: Option<&'a str>,
    pub local_name: &'a str,
}
```

### Decomposing Names

- **Prefix**: The identifier before the single colon `:` (`"soap"` in `"soap:Envelope"`).
- **Local Name**: The identifier after the colon (`"Envelope"` in `"soap:Envelope"`).
- **Unprefixed Elements**: Elements without a colon (`<item>`) have `prefix: None` and `local_name: "item"`.

```rust
use xml_lib_rust::QName;

let qname = QName::parse("soap:Envelope");
assert_eq!(qname.prefix, Some("soap"));
assert_eq!(qname.local_name, "Envelope");

let unprefixed = QName::parse("title");
assert_eq!(unprefixed.prefix, None);
assert_eq!(unprefixed.local_name, "title");
```

---

## 3. Hierarchical Scoping & Inheritance

Namespaces are declared using attributes:
- `xmlns="http://example.org/default"` binds the **default namespace** for the element and all unprefixed descendant elements.
- `xmlns:prefix="http://example.org/custom"` binds the prefix `prefix` to the given URI.

### Scoping Rules

1. **Child Inheritance**: Namespace bindings declared on an ancestor element are automatically in scope for all descendant elements.
2. **Default Namespace Inheritance**: Unprefixed child elements automatically inherit the enclosing default namespace unless overridden.
3. **Prefix Shadowing (Overriding)**: A child element may redeclare an existing prefix with a different URI; inside that subtree, the new URI takes precedence.
4. **Un-declaring Default Namespace**: In XML 1.0/1.1, setting `xmlns=""` clears the default namespace so descendant unprefixed elements have no namespace.

```xml
<!-- Default namespace is A -->
<root xmlns="http://example.com/nsA">
  <child1/> <!-- Belongs to nsA -->
  
  <!-- Override default namespace to B -->
  <child2 xmlns="http://example.com/nsB">
    <grandchild/> <!-- Belongs to nsB -->
  </child2>
  
  <!-- Clear default namespace -->
  <child3 xmlns="">
    <grandchild/> <!-- No namespace -->
  </child3>
</root>
```

---

## 4. DOM Inspection & Lookup APIs

[`Document`](../library/src/document.rs) provides direct inspection and resolution methods on any `NodeId`:

| Method | Return Type | Description |
| :--- | :--- | :--- |
| `doc.get_prefix(node_id)` | `Option<&str>` | Returns the tag's prefix (e.g. `"soap"` or `None`) |
| `doc.get_local_name(node_id)` | `&str` | Returns the local name without prefix |
| `doc.get_namespace_uri(node_id)` | `Option<String>` | Resolves the in-scope URI bound to the element's prefix or default namespace |
| `doc.lookup_namespace_uri(node_id, prefix)` | `Option<String>` | Looks up the URI bound to `prefix` from this node's scope |
| `doc.lookup_prefix(node_id, uri)` | `Option<String>` | Reverse lookup: finds the prefix bound to `uri` in this node's scope |

### Code Example

```rust
use xml_lib_rust::parse;

let xml = r#"<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
    <soap:Body><data>123</data></soap:Body>
</soap:Envelope>"#;

let doc = parse(xml)?;
let root_id = doc.root_element_id().unwrap();

assert_eq!(doc.get_prefix(root_id), Some("soap"));
assert_eq!(doc.get_local_name(root_id), "Envelope");
assert_eq!(
    doc.get_namespace_uri(root_id).as_deref(),
    Some("http://schemas.xmlsoap.org/soap/envelope/")
);

// Resolve prefix from child scope
let body_id = doc.get_children(root_id)[0];
assert_eq!(
    doc.lookup_prefix(body_id, "http://schemas.xmlsoap.org/soap/envelope/").as_deref(),
    Some("soap")
);
```

---

## 5. Namespace-Aware Querying

When searching documents containing multiple schemas, searching by local name alone can lead to false positives. `xml_lib_rust` provides `get_elements_by_tag_name_ns`:

```rust
use xml_lib_rust::parse;

let xml = r#"<catalog xmlns:book="http://example.com/books" xmlns:music="http://example.com/music">
    <book:item id="b1">Book Title</book:item>
    <music:item id="m1">Track Title</music:item>
</catalog>"#;

let doc = parse(xml)?;

// Find only items belonging to the books namespace
let book_items = doc.get_elements_by_tag_name_ns("http://example.com/books", "item");
assert_eq!(book_items.len(), 1);
assert_eq!(doc.get_attribute(book_items[0], "id"), Some("b1"));

// Wildcard search: all elements within the music namespace
let all_music = doc.get_elements_by_tag_name_ns("http://example.com/music", "*");
assert_eq!(all_music.len(), 1);
```

---

## 6. Streaming & Serialization with Namespaces

### The `NamespaceScope` Stack

Under the hood, `xml_lib_rust` uses a zero-allocation [`NamespaceScope`](../library/src/namespace/mod.rs) stack:

```rust
use xml_lib_rust::NamespaceScope;

let mut scope = NamespaceScope::new();

// Push root element scope
scope.push_scope();
scope.declare_prefix(Some("soap"), "http://schemas.xmlsoap.org/soap/envelope/");
scope.declare_default("http://example.com/default");

assert_eq!(scope.resolve_prefix(Some("soap")), Some("http://schemas.xmlsoap.org/soap/envelope/"));
assert_eq!(scope.resolve_default(), Some("http://example.com/default"));

// Pop when closing element
scope.pop_scope();
```

---

## 7. Real-World Examples

### SOAP 1.2 Request
```rust
use xml_lib_rust::{parse, stringify};

let soap_msg = r#"<?xml version="1.0"?>
<env:Envelope xmlns:env="http://www.w3.org/2003/05/soap-envelope"
              xmlns:rpc="http://www.w3.org/2003/05/soap-rpc">
  <env:Header/>
  <env:Body>
    <rpc:GetStockPrice>
      <rpc:Symbol>RUST</rpc:Symbol>
    </rpc:GetStockPrice>
  </env:Body>
</env:Envelope>"#;

let doc = parse(soap_msg)?;
let envelope_id = doc.root_element_id().unwrap();

println!("Root Namespace: {}", doc.get_namespace_uri(envelope_id).unwrap());
let rpc_elements = doc.get_elements_by_tag_name_ns("http://www.w3.org/2003/05/soap-rpc", "GetStockPrice");
assert_eq!(rpc_elements.len(), 1);
```

### SVG embedded inside HTML
```xml
<html xmlns="http://www.w3.org/1999/xhtml"
      xmlns:svg="http://www.w3.org/2000/svg">
  <body>
    <h1>Vector Graphic</h1>
    <svg:svg width="100" height="100">
      <svg:circle cx="50" cy="50" r="40" stroke="green" fill="yellow"/>
    </svg:svg>
  </body>
</html>
```
