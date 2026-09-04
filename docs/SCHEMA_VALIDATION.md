# Schema Validation Guide (DTD & XSD)

The `xml_lib_rust` validation subsystem provides structural, syntactic, and semantic validation for XML documents. It includes native implementations of both **Document Type Definitions (DTD)** and **W3C XML Schema Definition (XSD)**, all unified under the [`XmlValidator`](../library/src/validator.rs) trait.

---

## 1. The Unified `XmlValidator` Trait

Any validator in `xml_lib_rust` implements the common `XmlValidator` trait:

```rust
pub trait XmlValidator {
    /// Validates the provided DOM Document, returning Ok(()) on success,
    /// or a descriptive XmlError on violation.
    fn validate(&self, doc: &Document) -> Result<()>;
}
```

This abstraction allows applications to decouple document consumption from validation strategies, and easily chain multiple validators or plug in custom business rules.

---

## 2. Document Type Definition (DTD) Validation

The DTD subsystem supports internal subsets, external subsets (via a user-configurable resolver), element content models, attribute defaults and constraints, and ID/IDREF referential integrity.

### 2.1 Content Models Supported

| Content Model | DTD Syntax | Description |
| :--- | :--- | :--- |
| **EMPTY** | `<!ELEMENT br EMPTY>` | Must have no child elements and no text content |
| **ANY** | `<!ELEMENT container ANY>` | Any combination of elements and text allowed |
| **Mixed** | `<!ELEMENT p (#PCDATA \| b \| i)*>` | Character data mixed with specified child elements |
| **Children** | `<!ELEMENT note (to, from, heading, body)>` | Ordered sequence or choice of child elements |

### 2.2 Attribute Constraints & Types

The `<!ATTLIST>` declaration specifies attribute requirements:
- **`#REQUIRED`**: Attribute must be present on every instance of the element.
- **`#IMPLIED`**: Attribute is optional.
- **`#FIXED "val"`**: If present, attribute must equal `"val"`. If omitted, defaults to `"val"`.
- **`"default"`**: Default literal injected if the attribute is omitted.
- **`ID` / `IDREF`**: Ensures all IDs within the document are globally unique, and every IDREF points to an existing ID.

### 2.3 Default Attribute Injection (`apply_defaults`)

Unlike passive validators, `DtdValidator` can mutate a document to inject declared default attributes that were omitted in the source XML:

```rust
use xml_lib_rust::{Document, DtdValidator};

let xml = r#"
<!DOCTYPE library [
    <!ELEMENT library (book*)>
    <!ELEMENT book EMPTY>
    <!ATTLIST book
        id CDATA #REQUIRED
        status CDATA "available"
        format CDATA #FIXED "paperback">
]>
<library>
    <book id="b1"/>
</library>
"#;

let mut doc = Document::parse_str(xml)?;
let mut validator = DtdValidator::new();

// 1. Validate structure
validator.validate(&doc)?;

// 2. Automatically inject missing default attributes ("status" and "format")
let injected_count = validator.apply_defaults(&mut doc)?;
assert_eq!(injected_count, 2);

let book_id = doc.get_elements_by_tag_name("book")[0];
assert_eq!(doc.get_attribute(book_id, "status"), Some("available"));
assert_eq!(doc.get_attribute(book_id, "format"), Some("paperback"));
```

### 2.4 External Subset Resolution Hook

For documents referencing external DTD entities (`<!DOCTYPE note SYSTEM "note.dtd">`), register an `ExternalSubsetResolver`:

```rust
use xml_lib_rust::DtdValidator;

let mut validator = DtdValidator::new();
validator.set_external_resolver(|system_id, public_id| {
    match system_id {
        "rules.dtd" => Some(r#"
            <!ELEMENT catalog (item+)>
            <!ELEMENT item (#PCDATA)>
            <!ATTLIST item id ID #REQUIRED>
        "#.to_string()),
        _ => None, // Entity not found
    }
});
```

---

## 3. W3C XML Schema (XSD) Validation

The `XsdValidator` parses W3C XML Schema (`<xs:schema>`) definitions and performs strict structural, compositor, and facet validations on DOM trees.

### 3.1 Model Group Compositors

The engine validates three primary structural compositors:

| Compositor | Tag | Description |
| :--- | :--- | :--- |
| **Sequence** | `<xs:sequence>` | Child elements must appear in exact declared order |
| **Choice** | `<xs:choice>` | Exactly one of the alternative child elements must appear |
| **All** | `<xs:all>` | All declared elements must appear, but may appear in any order |

### 3.2 Cardinality (`minOccurs` / `maxOccurs`)

Occurrences can be constrained per element or compositor:
- `minOccurs="0"`: Optional element.
- `minOccurs="1"`: Mandatory element (default).
- `maxOccurs="1"`: At most one occurrence (default).
- `maxOccurs="unbounded"`: Arbitrary count allowed.

### 3.3 Simple Types & Restriction Facets

The validator enforces type boundaries and restriction facets:

| Facet | XSD Tag | Supported Data Types | Description |
| :--- | :--- | :--- | :--- |
| **Minimum Value** | `xs:minInclusive` | `xs:integer`, numeric | Numeric lower bound (inclusive) |
| **Maximum Value** | `xs:maxInclusive` | `xs:integer`, numeric | Numeric upper bound (inclusive) |
| **Minimum Length** | `xs:minLength` | `xs:string` | String minimum character length |
| **Maximum Length** | `xs:maxLength` | `xs:string` | String maximum character length |
| **Enumeration** | `xs:enumeration` | `xs:string` | Explicit list of allowable string literals |
| **Pattern** | `xs:pattern` | `xs:string` | Regex or substring matching filter |

### 3.4 Named Complex Types & Attributes

`XsdValidator` supports both global named `<xs:complexType name="...">` references and inline anonymous complex types:

```rust
use xml_lib_rust::{Document, XsdValidator, XmlValidator};

let schema_xml = r#"
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
    <xs:element name="person" type="PersonType"/>

    <xs:complexType name="PersonType">
        <xs:sequence>
            <xs:element name="name" type="xs:string"/>
            <xs:element name="age" type="xs:integer">
                <xs:simpleType>
                    <xs:restriction base="xs:integer">
                        <xs:minInclusive value="0"/>
                        <xs:maxInclusive value="120"/>
                    </xs:restriction>
                </xs:simpleType>
            </xs:element>
        </xs:sequence>
        <xs:attribute name="role" type="xs:string" use="required"/>
    </xs:complexType>
</xs:schema>
"#;

let valid_xml = r#"
<person role="admin">
    <name>Alice Smith</name>
    <age>30</age>
</person>
"#;

let mut validator = XsdValidator::new();
validator.parse_schema(schema_xml)?;

let doc = Document::parse_str(valid_xml)?;
assert!(validator.validate(&doc).is_ok());

let invalid_xml = r#"
<person role="admin">
    <name>Old Bob</name>
    <age>150</age> <!-- Exceeds maxInclusive="120" -->
</person>
"#;

let invalid_doc = Document::parse_str(invalid_xml)?;
assert!(validator.validate(&invalid_doc).is_err());
```

---

## 4. Custom Business Rule Validators

Because `XmlValidator` is a public trait, application code can implement custom validators that combine DOM queries, XPath checks, or cryptographic assertions:

```rust
use xml_lib_rust::{Document, XmlValidator, XmlError, Result};

pub struct InventoryValidator {
    pub max_total_items: usize,
}

impl XmlValidator for InventoryValidator {
    fn validate(&self, doc: &Document) -> Result<()> {
        let items = doc.get_elements_by_tag_name("item");
        if items.len() > self.max_total_items {
            return Err(XmlError::ValidationError(format!(
                "Order exceeds maximum allowed item count of {}: found {}",
                self.max_total_items,
                items.len()
            )));
        }
        Ok(())
    }
}
```
