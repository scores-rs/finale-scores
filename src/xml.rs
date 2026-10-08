//! A minimal element tree over EnigmaXML, which the typed records are
//! read from.
//!
//! EnigmaXML is a flat database dump rather than a nested document: the
//! `<others>`, `<details>` and `<entries>` sections hold records keyed by
//! `cmper`/`inci`/`entnum` attributes that reference each other by number,
//! so reading needs random access. A small owned tree is the simplest way
//! to get it.

use crate::error::{FinaleError, Result};
use quick_xml::events::{BytesStart, Event as XmlEvent};
use quick_xml::reader::Reader;

#[derive(Debug, Default)]
pub(crate) struct Element {
    pub name: String,
    attrs: Vec<(String, String)>,
    pub children: Vec<Element>,
    text: String,
}

impl Element {
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    pub fn attr_u32(&self, name: &str) -> Option<u32> {
        self.attr(name)?.trim().parse().ok()
    }

    pub fn child(&self, name: &str) -> Option<&Element> {
        self.children.iter().find(|c| c.name == name)
    }

    pub fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Element> {
        self.children.iter().filter(move |c| c.name == name)
    }

    /// A boolean flag: true when the (usually empty) child element is present.
    pub fn flag(&self, name: &str) -> bool {
        self.child(name).is_some()
    }

    /// A numeric child's value. Finale leaves out elements holding their
    /// default, which is 0 for every numeric field read here.
    pub fn int(&self, name: &str) -> i64 {
        self.child(name)
            .and_then(|c| c.text.trim().parse().ok())
            .unwrap_or(0)
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn child_text(&self, name: &str) -> Option<&str> {
        self.child(name).map(|c| c.text.as_str())
    }

    /// Whether this record belongs to the score rather than to a linked
    /// part: part records carry a non-zero `part` attribute.
    pub fn is_score_record(&self) -> bool {
        self.attr("part").is_none_or(|p| p == "0")
    }
}

/// Parses EnigmaXML into its `<finale>` root element.
pub(crate) fn parse(xml: &str) -> Result<Element> {
    let mut reader = Reader::from_str(xml);
    // The bottom of the stack is a synthetic document node.
    let mut stack = vec![Element::default()];
    loop {
        let event = reader
            .read_event()
            .map_err(|e| FinaleError::Xml(e.to_string()))?;
        match event {
            XmlEvent::Eof => break,
            XmlEvent::Start(e) => stack.push(open(&e)),
            XmlEvent::Empty(e) => {
                let element = open(&e);
                stack.last_mut().unwrap().children.push(element);
            }
            XmlEvent::End(_) => {
                if stack.len() < 2 {
                    return Err(unbalanced());
                }
                let element = stack.pop().unwrap();
                stack.last_mut().unwrap().children.push(element);
            }
            XmlEvent::Text(text) => {
                let raw = text.into_inner();
                if let Ok(unescaped) = quick_xml::escape::unescape(&raw) {
                    stack.last_mut().unwrap().text.push_str(&unescaped);
                }
            }
            XmlEvent::GeneralRef(r) => {
                // Entity references arrive separately from the text around them.
                let name = r.xml10_content();
                let resolved = match name.as_ref() {
                    "amp" => Some('&'),
                    "lt" => Some('<'),
                    "gt" => Some('>'),
                    "quot" => Some('"'),
                    "apos" => Some('\''),
                    _ => r.resolve_char_ref().ok().flatten(),
                };
                if let Some(c) = resolved {
                    stack.last_mut().unwrap().text.push(c);
                }
            }
            _ => {}
        }
    }
    let document = stack.pop().unwrap();
    if !stack.is_empty() {
        return Err(unbalanced());
    }
    document
        .children
        .into_iter()
        .find(|e| e.name == "finale")
        .ok_or_else(|| FinaleError::Xml("no <finale> root".into()))
}

fn unbalanced() -> FinaleError {
    FinaleError::Xml("unbalanced elements".into())
}

fn open(e: &BytesStart) -> Element {
    let attrs = e
        .attributes()
        .flatten()
        .filter_map(|a| {
            let value = a
                .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                .ok()?
                .into_owned();
            Some((a.key.local_name().as_ref().to_string(), value))
        })
        .collect();
    Element {
        name: e.local_name().as_ref().to_string(),
        attrs,
        children: Vec::new(),
        text: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_records_and_entities() {
        let root = parse(
            r#"<?xml version="1.0"?><finale xmlns="x"><texts><fileInfo type="title">A &amp; B&#x21;</fileInfo></texts></finale>"#,
        )
        .unwrap();
        let info = root.child("texts").unwrap().child("fileInfo").unwrap();
        assert_eq!(info.attr("type"), Some("title"));
        assert_eq!(info.text(), "A & B!");
        assert!(parse("<other/>").is_err());
        assert!(parse("<finale><a></finale>").is_err());
    }
}
