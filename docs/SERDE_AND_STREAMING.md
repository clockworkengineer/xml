# Serde Data Binding & Streaming I/O Guide

`xml_lib_rust` provides seamless integration with the Rust [`serde`](https://serde.rs) framework for strongly typed data serialization and deserialization, as well as high-throughput streaming from arbitrary `std::io::Read` inputs and legacy character set decoding.

---

## 1. Enabling Serde Support

Serde integration is feature-gated. In your `Cargo.toml`:

```toml
[dependencies]
xml_lib_rust = { version = "0.1.0", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
```

---

## 2. Strongly Typed Deserialization (`from_str`)

The `xml_lib_rust::serde_impl::from_str` function deserializes an XML string directly into any type implementing `serde::Deserialize`:

```rust
use serde::Deserialize;
use xml_lib_rust::serde_impl::from_str;

#[derive(Debug, Deserialize, PartialEq)]
struct ServerConfig {
    host: String,
    port: u16,
    debug: bool,
    workers: Option<usize>,
}

let xml = r#"
<config>
    <host>127.0.0.1</host>
    <port>8080</port>
    <debug>true</debug>
    <workers>4</workers>
</config>
"#;

let config: ServerConfig = from_str(xml)?;
assert_eq!(config.host, "127.0.0.1");
assert_eq!(config.port, 8080);
assert_eq!(config.debug, true);
assert_eq!(config.workers, Some(4));
```

### Attribute & Element Transparency

The deserializer automatically binds both XML attributes and child elements to struct fields:

```rust
#[derive(Debug, Deserialize)]
struct User {
    id: u64,          // Can match attribute id="1" OR element <id>1</id>
    username: String,
}

let xml_with_attr = r#"<user id="42"><username>ferris</username></user>"#;
let user: User = from_str(xml_with_attr)?;
assert_eq!(user.id, 42);
assert_eq!(user.username, "ferris");
```

### Collections & Sequences

Repeated child elements map directly to `Vec<T>`:

```rust
#[derive(Debug, Deserialize)]
struct Item {
    sku: String,
    qty: u32,
}

#[derive(Debug, Deserialize)]
struct Order {
    order_id: String,
    items: Vec<Item>,
}

let xml = r#"
<order>
    <order_id>ORD-9981</order_id>
    <items>
        <item><sku>A100</sku><qty>2</qty></item>
        <item><sku>B200</sku><qty>5</qty></item>
    </items>
</order>
"#;

let order: Order = from_str(xml)?;
assert_eq!(order.items.len(), 2);
assert_eq!(order.items[0].sku, "A100");
```

---

## 3. Serialization (`to_string` & `to_string_with_root`)

`xml_lib_rust::serde_impl` allows serializing Rust data structures directly into valid UTF-8 XML strings with proper XML character escaping (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`).

### Standard Serialization (`to_string`)

Defaults to wrapping the payload in a top-level `<root>` tag:

```rust
use serde::Serialize;
use xml_lib_rust::serde_impl::to_string;

#[derive(Serialize)]
struct Status {
    healthy: bool,
    uptime_seconds: u64,
}

let status = Status { healthy: true, uptime_seconds: 3600 };
let xml = to_string(&status)?;
assert_eq!(xml, "<root><healthy>true</healthy><uptime_seconds>3600</uptime_seconds></root>");
```

### Custom Root Tag (`to_string_with_root`)

```rust
use xml_lib_rust::serde_impl::to_string_with_root;

let xml = to_string_with_root("system_status", &status)?;
assert_eq!(xml, "<system_status><healthy>true</healthy><uptime_seconds>3600</uptime_seconds></system_status>");
```

---

## 4. Streaming I/O (`parse_reader`)

When reading large documents or streaming over networks, loading the entire payload into a contiguous `String` in advance is inefficient. `parse_reader` processes any stream implementing `std::io::Read`:

```rust
use std::fs::File;
use std::io::BufReader;
use xml_lib_rust::parse_reader;

let file = File::open("large_payload.xml")?;
let reader = BufReader::new(file);

// Stream directly from disk into DOM
let doc = parse_reader(reader)?;
println!("Root element: {:?}", doc.get_root_element_name());
```

`parse_reader` automatically auto-detects UTF-8, UTF-8 BOM, UTF-16 LE, and UTF-16 BE headers from the stream preamble.

---

## 5. Legacy & Multi-Byte Character Encodings

`xml_lib_rust` supports legacy single-byte encodings via `XmlSource::from_bytes_with_encoding`:

| Encoding Name | Identifier | Supported Characters |
| :--- | :--- | :--- |
| **UTF-8** | `"UTF-8"`, `"UTF8"` | Universal Unicode character set |
| **ISO-8859-1 (Latin-1)** | `"ISO-8859-1"`, `"LATIN1"` | Western European languages (bytes 0–255 map 1:1 to U+0000..U+00FF) |
| **Windows-1252** | `"WINDOWS-1252"`, `"CP1252"` | Latin-1 superset with typographics (`€`, `‘`, `’`, `“`, `”`, `™`) |
| **7-bit US-ASCII** | `"US-ASCII"`, `"ASCII"` | 7-bit standard ASCII (bytes > 127 return validation error) |

### Example: Decoding a Windows-1252 Payload

```rust
use xml_lib_rust::io::source::XmlSource;
use xml_lib_rust::parser::XmlParser;
use xml_lib_rust::options::ParseOptions;

// Byte slice containing Windows-1252 byte 0x80 (Euro symbol €)
let raw_bytes: &[u8] = b"<price currency=\"\x80\">100</price>";

let source = XmlSource::from_bytes_with_encoding(raw_bytes, "WINDOWS-1252")?;
let mut parser = XmlParser::new(source, ParseOptions::default());
let doc = parser.parse()?;

let root_id = doc.root_element_id().unwrap();
assert_eq!(doc.get_attribute(root_id, "currency"), Some("€"));
```
