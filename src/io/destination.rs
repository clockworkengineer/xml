use crate::io::encoding::Format;

#[derive(Debug, Clone)]
pub struct XmlDestination {
    pub buffer: String,
    pub format: Format,
}

impl XmlDestination {
    pub fn new(format: Format) -> Self {
        Self {
            buffer: String::new(),
            format,
        }
    }

    pub fn write_str(&mut self, s: &str) {
        self.buffer.push_str(s);
    }

    pub fn write_char(&mut self, ch: char) {
        self.buffer.push(ch);
    }

    pub fn into_bytes(self) -> Vec<u8> {
        match self.format {
            Format::Utf8 => self.buffer.into_bytes(),
            Format::Utf8Bom => {
                let mut bytes = vec![0xEF, 0xBB, 0xBF];
                bytes.extend_from_slice(self.buffer.as_bytes());
                bytes
            }
            Format::Utf16Be => {
                let mut bytes = Vec::new();
                for u in self.buffer.encode_utf16() {
                    bytes.extend_from_slice(&u.to_be_bytes());
                }
                bytes
            }
            Format::Utf16Le => {
                let mut bytes = Vec::new();
                for u in self.buffer.encode_utf16() {
                    bytes.extend_from_slice(&u.to_le_bytes());
                }
                bytes
            }
            Format::Utf32Be => {
                let mut bytes = Vec::new();
                for ch in self.buffer.chars() {
                    bytes.extend_from_slice(&(ch as u32).to_be_bytes());
                }
                bytes
            }
            Format::Utf32Le => {
                let mut bytes = Vec::new();
                for ch in self.buffer.chars() {
                    bytes.extend_from_slice(&(ch as u32).to_le_bytes());
                }
                bytes
            }
        }
    }
}
