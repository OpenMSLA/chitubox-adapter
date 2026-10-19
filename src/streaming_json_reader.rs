use anyhow::{Result, bail};
use serde_json::Value;

pub struct StreamingJsonReader {
    limit: usize,
    buffer: Vec<u8>,
    stack: Vec<u8>,
    in_string: bool,
    escaped: bool,
}

impl StreamingJsonReader {
    /// Crete new reader
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            buffer: Vec::new(),
            stack: Vec::new(),
            in_string: false,
            escaped: false,
        }
    }

    pub fn on_frame(&mut self, chunk: &[u8]) -> Result<Vec<Value>> {
        let mut messages = Vec::new();
        for &byte in chunk {
            if self.buffer.is_empty() {
                if b" \r\n\t".contains(&byte) {
                    continue;
                }

                if byte != b'{' {
                    bail!("Not a json");
                }
            }

            self.buffer.push(byte);

            if self.buffer.len() > self.limit {
                bail!("JSON exceeded limit: {} bytes", self.limit);
            }

            if self.in_string {
                if self.escaped {
                    self.escaped = false;
                } else if byte == b'\\' {
                    self.escaped = true;
                } else if byte == b'"' {
                    self.in_string = false;
                }
            } else if byte == b'"' {
                self.in_string = true;
            } else if byte == b'{' || byte == b'[' {
                self.stack.push(byte);
            } else if byte == b'}' || byte == b']' {
                let expected = if byte == b'}' { b'{' } else { b'[' };

                if self.stack.pop() != Some(expected) {
                    bail!("Unexpected braces");
                }

                if self.stack.is_empty() {
                    let message: Value = serde_json::from_slice(&self.buffer)?;

                    if !message.is_object() {
                        bail!("Json object was expected");
                    }

                    messages.push(message);
                    self.buffer.clear();
                }
            }
        }

        Ok(messages)
    }
}

impl Default for StreamingJsonReader {
    fn default() -> Self {
        Self::new(1024 * 1024)
    }
}
