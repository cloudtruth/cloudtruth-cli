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

#[cfg(test)]
mod test {
    use cloudtruth_restapi::apis::projects_api::remove_null_values;

    const LOCALHOST: &str = "\"values\":{\"https://localhost:8000/api/v1/environments/889d8b04-aa04-4915-b462-b70a7b79e130/\":null}";

    #[test]
    fn test_single() {
        let updated = remove_null_values(LOCALHOST);
        assert_eq!(updated.as_str(), "\"values\":{}");
    }

    #[test]
    fn test_double() {
        // Previously, the regex would get greedy and take everything until "final" values...
        // It would only return a single: "values":{}
        let double = format!("{},{}", LOCALHOST, LOCALHOST);
        let updated = remove_null_values(double.as_str());
        assert_eq!(updated.as_str(), "\"values\":{},\"values\":{}");
    }
}
