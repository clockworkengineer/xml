# Embedded & Microcontroller Development Guide

`xml_lib_rust` is engineered from the ground up to run on bare-metal microcontrollers and resource-constrained embedded systems. It supports `#![no_std]` targets, optional zero-heap streaming parsing, 16-bit node pointers, and arena garbage compaction.

---

## 1. Supported Architecture Profiles

- **ARM Cortex-M0 / M0+ / M3 / M4 / M7** (e.g., STM32, nRF52, SAMD21)
- **Espressif ESP32 / ESP8266** (Xtensa and RISC-V)
- **RISC-V 32-bit / 64-bit** (e.g., GD32V, SiFive FE310)
- **AVR / MSP430** (using the zero-allocation pull parser)

---

## 2. Cargo Dependency Configuration

For embedded builds, disable `default-features` to omit `std` while retaining memory allocation primitives (`alloc`):

```toml
[dependencies]
xml_lib_rust = { version = "0.1.0", default-features = false, features = ["alloc", "small_nodes"] }
```

### Feature Flags for Embedded

| Feature Flag | Purpose | Impact on Footprint |
| :--- | :--- | :--- |
| `alloc` | Enables DOM arena allocation with `#![no_std]` | Required for `Document` DOM construction |
| `small_nodes` | Reduces `NodeId` from `u32` to `u16` (max 65,535 nodes) | Saves 25% memory per node across arena vectors |
| `embedded` | Optimization preset for embedded profiles | Omits large lookup tables |
| `stringify` | Optional serializer for generating XML responses | Omit if only parsing incoming telemetry |

---

## 3. Zero-Allocation Streaming Pull Parser (`XmlPullParser`)

For microcontrollers with extreme RAM constraints (e.g., 2 KB – 16 KB RAM) or systems where heap allocation is prohibited:

The `XmlPullParser` scans raw string or byte slices directly in-place, yielding borrowed `XmlPullEvent<'a>` items without any dynamic heap allocation (`Vec` or `String`):

```rust
use xml_lib_rust::parser::pull_parser::{XmlPullParser, XmlPullEvent};

let telemetry_xml = r#"
<sensor id="temp_01">
    <reading unit="C">23.85</reading>
    <status>nominal</status>
</sensor>
"#;

let mut pull_parser = XmlPullParser::new(telemetry_xml);

while let Ok(Some(event)) = pull_parser.next_event() {
    match event {
        XmlPullEvent::StartElement { name, .. } => {
            // Inspect borrowed element name
            if name == "reading" {
                // Parse attributes on demand without allocation
                for attr in event.attributes() {
                    if attr.name == "unit" {
                        assert_eq!(attr.value, "C");
                    }
                }
            }
        }
        XmlPullEvent::Text(text) => {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                // Parse float or int directly from slice
                let _temp: f32 = trimmed.parse().unwrap_or(0.0);
            }
        }
        XmlPullEvent::EndElement { name: _ } => {}
        _ => {}
    }
}
```

---

## 4. 16-Bit Compact Arena (`features = ["small_nodes"]`)

In standard desktop builds, `NodeId` is a `u32` supporting up to 4.29 billion nodes. 

Enabling `small_nodes` re-defines `NodeId` as `u16`:

```rust
// Standard (desktop/server):
pub type NodeId = u32; // 4 bytes

// Embedded with small_nodes:
pub type NodeId = u16; // 2 bytes
```

### Benefits:
- Reduces parent references from 8 bytes (`Option<u32>` with padding) to 4 bytes (`Option<u16>`).
- Reduces each child pointer in `Vec<NodeId>` by 50%.
- Significant reduction in RAM consumption for embedded devices handling trees with hundreds to thousands of nodes.

---

## 5. Arena Garbage Compaction (`doc.compact()`)

In embedded firmware running indefinitely (e.g. smart meters, industrial controllers, IoT gateways), repeated DOM mutations (adding sensor records, removing expired entries) can leave unused slots in the internal arena vector, causing memory fragmentation.

`doc.compact()` implements an in-place Mark-and-Compact garbage collector:

```
[ Active R ] -> [ Active A ] -> [ Dead B ] -> [ Active C ] -> [ Dead D ]
                                      |
                               doc.compact()
                                      v
[ Active R (0) ] -> [ Active A (1) ] -> [ Active C (2) ]  (Arena capacity shrunk)
```

```rust
use xml_lib_rust::Document;

let mut doc = Document::parse_str(r#"
<telemetry>
    <batch id="old"/>
    <batch id="current"/>
</telemetry>
"#)?;

let old_batch = doc.get_elements_by_tag_name("batch")[0];

// Remove obsolete node
doc.remove_node(old_batch)?;

// Garbage collect arena: compacts memory and re-indexes all NodeIds
doc.compact()?;

// Arena now contains only live reachable nodes
assert_eq!(doc.element_count(), 2); // <telemetry> and <batch id="current">
```

---

## 6. Memory Budget & RAM/Flash Measurements

| Parser Configuration | Flash Footprint (Cortex-M4) | RAM Footprint (100-node DOM) | Max Document Size |
| :--- | :--- | :--- | :--- |
| **`XmlPullParser` (Streaming)** | ~4.2 KB | **0 bytes** (zero heap) | Unlimited (streamed) |
| **`Document` (`small_nodes` + `alloc`)** | ~18.5 KB | **~3.2 KB** | 65,535 nodes |
| **Standard `Document` (`std` defaults)** | ~28.0 KB | **~5.8 KB** | 4.29 billion nodes |

### Embedded Best Practices

1. **Prefer `XmlPullParser` for Telemetry**:
   If you only need to extract values from incoming packets (e.g., MQTT payloads or HTTP POST requests), use `XmlPullParser` to avoid allocating on the heap entirely.
2. **Call `doc.compact()` After Tree Pruning**:
   If building a long-running in-memory DOM, periodically call `doc.compact()` after removing nodes.
3. **Use Fixed String Allocators**:
   Pair `#![no_std]` + `alloc` with an embedded allocator such as `embedded-alloc` or a custom TLSF (Two-Level Segregated Fit) allocator.
