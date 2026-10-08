//! The `<texts>` section: text content, keyed by number, in Enigma string
//! form (see [`crate::enigma_string`]).

use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Texts {
    /// `blockText`s, by number (a `textBlock`'s `text_id`).
    pub block_texts: HashMap<u32, String>,
    /// `expression`s, by number (a `textExprDef`'s `text_id_key`).
    pub expressions: HashMap<u32, String>,
    /// `fileInfo`, by type (`title`, `composer`, `copyright`, ...).
    pub file_info: HashMap<String, String>,
}
