//! A streaming iterator over UFO GLIF XML events.
use quick_xml::events::Event;
use std::io::BufRead;

use quick_xml::Reader;

use crate::error::Error;

/// A streaming XML reader that iterates over GLIF events from any [`BufRead`] source.
pub struct GlifEventReader<R: BufRead> {
    reader: Reader<R>,
    buf: Vec<u8>,
}

impl<R: BufRead> GlifEventReader<R> {
    pub fn new(reader: R) -> Self {
        let xml_reader = Reader::from_reader(reader);
        Self {
            reader: xml_reader,
            buf: Vec::new(),
        }
    }
}

impl<R: BufRead> Iterator for GlifEventReader<R> {
    type Item = Result<Event<'static>, Error>;
    /// Returns the next GLIF event from the XML reader.
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Eof) => return None,
                Ok(ev) => {
                    let owned = ev.into_owned();
                    self.buf.clear();
                    return Some(Ok(owned));
                }
                Err(e) => return Some(Err(Error::from(e))),
            }
        }
    }
}
