use std::io::{Error, ErrorKind, Write};
use std::str;

/// This is a string buffer that behaves like a `Write` element, so it can be used in place of an
/// output stream (e.g. stdout).
pub struct StringWriter {
    string: String,
}

impl StringWriter {
    /// Create a new `StringWriter`
    pub fn new() -> StringWriter {
        StringWriter {
            string: String::new(),
        }
    }

    /// Return a reference to the internally written `String`. This is test code (not used in the
    /// main product), so is allowed to be dead.
    #[allow(dead_code)]
    pub fn as_string(&self) -> &str {
        &self.string
    }
}

impl Default for StringWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Write for StringWriter {
    fn write(&mut self, data: &[u8]) -> Result<usize, Error> {
        let string = match str::from_utf8(data) {
            Ok(s) => s,
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("Cannot decode utf8 string : {}", e),
                ))
            }
        };
        self.string.push_str(string);
        Ok(data.len())
    }

    fn flush(&mut self) -> Result<(), Error> {
        // Nothing to do here
        Ok(())
    }
}
