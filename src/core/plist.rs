//! Property list (`.plist`) file parser.
use std::io::BufReader;
use std::io::Cursor;
use std::path::Path;

use plist::Value;

use crate::error::Result;
use crate::provider::Provider;

/// Parser for reading and querying property list (`.plist`) files inside a UFO font file system.
pub struct PlistParser {
    provider: Box<dyn Provider>,
}

impl PlistParser {
    /// Creates a new parser from a [`Provider`] instance.
    pub fn new(provider: Box<dyn Provider>) -> Result<Self> {
        Ok(Self { provider })
    }

    /// Reads a plist file at `path` and parses it into a [`Value`].
    pub fn parse_plist(&self, path: &Path) -> Result<Value> {
        let reader = self.open_plist(path)?;
        let value = Value::from_reader(reader)?;
        Ok(value)
    }

    /// Opens a `.plist` file from the UFO file system and returns a buffered reader.
    fn open_plist(&self, path: &Path) -> Result<BufReader<Cursor<Vec<u8>>>> {
        let data = self.provider.read(path)?;
        let cursor = Cursor::new(data);
        let file = BufReader::new(cursor);
        Ok(file)
    }
}
