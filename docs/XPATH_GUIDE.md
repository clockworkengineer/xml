# XPath 1.0 Query Engine Reference Guide

The `xml_lib_rust` XPath subsystem provides an in-memory, zero-copy XPath 1.0 engine designed to execute arbitrary navigational, selection, and computed expressions directly against the contiguous DOM `Document`.

---

## 1. Architectural Overview

The XPath engine operates through three primary stages:

```
                      +-------------------+
                      | XPath Query String|
                      +-------------------+
                                |
                                v
                      +-------------------+
                      |   XPathLexer      |  Tokenizes expression into
                      +-------------------+  operators, axes, tests & literals
                                |
                                v
                      +-------------------+
                      |   XPathParser     |  Constructs recursive
                      +-------------------+  XPathExpr Abstract Syntax Tree (AST)
                                |
                                v
+------------------+  +-------------------+
|  DOM Document    |->|  XPathEvaluator   |  Evaluates AST against context node;
|  (Contiguous)    |  +-------------------+  supports variable & function lookups
+------------------+            |
                                v
                      +-------------------+
                      |    XPathValue     |  NodeSet, Boolean, Number, String
                      +-------------------+
```

### High-Level API Entry Point

```rust
use xml_lib_rust::{Document, XPathEngine, XPathValue};

let xml = r#"
<catalog>
    <book id="bk101" category="fiction">
        <title>The Rust Programming Language</title>
        <price>39.95</price>
    </book>
    <book id="bk102" category="reference">
        <title>XML in a Nutshell</title>
        <price>24.50</price>
    </book>
</catalog>
"#;

let doc = Document::parse_str(xml)?;
let mut engine = XPathEngine::new(&doc);

// Evaluate directly to matching NodeIds:
let nodes = engine.evaluate_nodes("//book[@category='fiction']/title", None)?;
assert_eq!(nodes.len(), 1);
assert_eq!(doc.get_text_content(nodes[0]), "The Rust Programming Language");

// Or evaluate to an XPathValue (NodeSet, Number, Boolean, or String):
let count_val = engine.evaluate("count(//book)", None)?;
assert_eq!(count_val, XPathValue::Number(2.0));
```

---

## 2. Supported XPath Axes (All 13 W3C Standard Axes)

The engine implements all 13 standard axes specified in the W3C XPath 1.0 Recommendation, allowing bidirectional navigation across the node graph.

| Axis Name | Syntax | Shorthand | Direction | Description |
| :--- | :--- | :--- | :--- | :--- |
| `child` | `child::node` | `node` | Forward | Immediate children of the context node |
| `descendant` | `descendant::node` | — | Forward | All descendants (children, grandchildren, etc.) |
| `descendant-or-self` | `descendant-or-self::node` | `//` | Forward | The context node and all its descendants |
| `parent` | `parent::node` | `..` | Reverse | Immediate parent element or root of the context node |
| `ancestor` | `ancestor::node` | — | Reverse | All ancestors up to the document root |
| `ancestor-or-self` | `ancestor-or-self::node` | — | Reverse | The context node and all its ancestors |
| `following-sibling` | `following-sibling::node` | — | Forward | Sibling elements occurring after the context node |
| `preceding-sibling` | `preceding-sibling::node` | — | Reverse | Sibling elements occurring before the context node |
| `following` | `following::node` | — | Forward | All nodes in document order after the context node, excluding descendants |
| `preceding` | `preceding::node` | — | Reverse | All nodes in document order before the context node, excluding ancestors |
| `attribute` | `attribute::name` | `@name` | Unordered | Attributes belonging to the context element |
| `self` | `self::node` | `.` | Context | The context node itself |
| `namespace` | `namespace::prefix` | — | Unordered | In-scope namespace declarations for the context element |

### Axis Traversal Visualized

Consider the following XML fragment:

```xml
<a>
  <b>
    <c id="target"/>
    <d/>
  </b>
  <e/>
</a>
```

If `<c id="target"/>` is the context node:
- `parent::*` selects `<b>`.
- `ancestor::*` selects `<b>` and `<a>`.
- `following-sibling::*` selects `<d/>`.
- `following::*` selects `<d/>` and `<e/>`.
- `preceding-sibling::*` yields an empty set.
- `ancestor-or-self::*` selects `<c>`, `<b>`, and `<a>`.

---

## 3. Node Tests & Predicates

### Node Tests

| Node Test | Example | Description |
| :--- | :--- | :--- |
| **Name** | `/catalog/book` | Matches elements with the exact local name or QName |
| **Wildcard (`*`)** | `/catalog/*` | Matches any element child regardless of tag name |
| **Attribute Wildcard (`@*`)** | `/book/@*` | Matches any attribute attached to the element |
| **`node()`** | `/catalog/node()` | Matches any child node (element, text, comment, PI) |
| **`text()`** | `//title/text()` | Matches text and CDATA nodes |
| **`comment()`** | `//comment()` | Matches XML comment nodes |

### Predicates & Compound Logic

Predicates filter node-sets enclosed within brackets `[...]`:

```xpath
//book[price > 20.00 and @category != 'fiction']
```

- **Position indexing**: `//book[1]` (Note: XPath is 1-indexed!) or `//book[last()]`.
- **Relational operators**: `=`, `!=`, `<`, `<=`, `>`, `>=`.
- **Boolean operators**: `and`, `or`, `not(...)`.
- **Arithmetic operators**: `+`, `-`, `*`, `div`, `mod`.
- **Set union**: `//title | //price` combines multiple queries into a deduplicated node-set.

---

## 4. Built-in Function Reference

The evaluator provides standard XPath 1.0 library functions:

### 4.1 Node-Set Functions
- `position()`: Returns the 1-based ordinal position of the context node within its evaluated set.
- `last()`: Returns the total count of nodes in the context set.
- `count(node-set)`: Returns the number of nodes in the specified set.
- `id(string)`: Looks up elements by their unique XML `id` or `xml:id` attribute.
- `local-name([node-set])`: Returns the local name of the first node in the set or context node.
- `name([node-set])`: Returns the qualified tag name.
- `namespace-uri([node-set])`: Returns the resolved namespace URI of the node.

### 4.2 String Functions
- `string([object])`: Converts any object to its string representation.
- `concat(s1, s2, ...)`: Concatenates two or more strings.
- `starts-with(str, prefix)`: Returns `true` if `str` begins with `prefix`.
- `ends-with(str, suffix)`: Returns `true` if `str` terminates with `suffix`.
- `contains(str, substr)`: Returns `true` if `substr` occurs inside `str`.
- `substring(str, start, [len])`: 1-based substring extraction.
- `substring-before(str, pattern)`: Text preceding first occurrence of `pattern`.
- `substring-after(str, pattern)`: Text following first occurrence of `pattern`.
- `string-length([str])`: Character count of string.
- `normalize-space([str])`: Strips leading/trailing whitespace and collapses inner runs to single spaces.
- `translate(str, from, to)`: Character-by-character mapping and replacement.
- `lower-case(str)`: Converts ASCII/Unicode characters to lower case.
- `upper-case(str)`: Converts ASCII/Unicode characters to upper case.
- `replace(str, pattern, replacement)`: Global literal string replacement.

### 4.3 Boolean Functions
- `boolean(object)`: Converts node-sets, numbers, or strings to boolean.
- `not(bool)`: Negates boolean condition.
- `true()`: Constant `true`.
- `false()`: Constant `false`.
- `lang(string)`: Tests whether context node matches the language tag in `xml:lang`.

### 4.4 Number Functions
- `number([object])`: Coerces value to IEEE 754 64-bit float (`f64`).
- `sum(node-set)`: Sums numeric representations of all nodes in set.
- `floor(number)`: Greatest integer not greater than argument.
- `ceiling(number)`: Smallest integer not less than argument.
- `round(number)`: Rounds to nearest integer.

---

## 5. Dynamic Variable Bindings (`$var`)

You can pass runtime variables into queries without string concatenation:

```rust
use xml_lib_rust::{Document, XPathEngine, XPathValue};

let doc = Document::parse_str(r#"
<inventory>
    <item sku="A12" stock="14" min="20"/>
    <item sku="B88" stock="45" min="10"/>
    <item sku="C04" stock="3"  min="5"/>
</inventory>
"#)?;

let mut engine = XPathEngine::new(&doc);

// Bind dynamic threshold variable
engine.set_variable("threshold", XPathValue::Number(10.0));

// Evaluate expression referencing $threshold
let low_stock_items = engine.evaluate_nodes(
    "//item[@stock < $threshold]",
    None
)?;

assert_eq!(low_stock_items.len(), 1);
assert_eq!(doc.get_attribute(low_stock_items[0], "sku"), Some("C04"));
```

---

## 6. User-Defined Custom Functions

Extend the XPath engine with application-specific functions:

```rust
use xml_lib_rust::{Document, XPathEngine, XPathValue, XmlError};

let doc = Document::parse_str(r#"
<users>
    <user email="ALICE@EXAMPLE.COM" role="admin"/>
    <user email="bob@sample.org" role="member"/>
</users>
"#)?;

let mut engine = XPathEngine::new(&doc);

// Register custom function "is-domain"
engine.register_function("is-domain", |args| {
    if args.len() != 2 {
        return Err(XmlError::XPathError("is-domain expects 2 args".into()));
    }
    let email = match &args[0] {
        XPathValue::String(s) => s.to_lowercase(),
        _ => return Ok(XPathValue::Boolean(false)),
    };
    let domain = match &args[1] {
        XPathValue::String(s) => s.to_lowercase(),
        _ => return Ok(XPathValue::Boolean(false)),
    };
    Ok(XPathValue::Boolean(email.ends_with(&format!("@{}", domain))))
});

let matches = engine.evaluate_nodes(
    "//user[is-domain(@email, 'example.com')]",
    None
)?;
assert_eq!(matches.len(), 1);
assert_eq!(doc.get_attribute(matches[0], "role"), Some("admin"));
```

---

## 7. Performance & Optimization Tips

1. **Reuse `XPathEngine` Instances**:
   Creating an `XPathEngine` is lightweight, but registering multiple custom functions or variables is amortized over multiple queries against the same document.
2. **Prefer Root Context**:
   Passing `None` as the context node defaults directly to the document root, skipping manual node lookup.
3. **Use Specific Axes When Possible**:
   Using `child::` (`/`) is faster than deep document scans with `descendant::` (`//`), especially for large trees.
