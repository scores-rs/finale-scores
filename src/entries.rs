//! The `<entries>` section: entries (a note, chord or rest) keyed by
//! `entnum`, each linked to its neighbours in a frame by `prev`/`next`.

use crate::xml::Element;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Entry {
    pub entnum: u32,
    pub prev: u32,
    pub next: u32,
    /// The symbolic (pre-tuplet) duration in EDU.
    pub dura: i64,
    pub is_valid: bool,
    /// A note or chord; otherwise a rest.
    pub is_note: bool,
    pub grace_note: bool,
    /// Tuplets (`details/tupletDef`) start here.
    pub tuplet_start: bool,
    /// In its layer's second voice.
    pub v2: bool,
    pub notes: Vec<Note>,
}

/// One note of an entry.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Note {
    pub id: u32,
    /// Diatonic steps from the key's tonic in the middle-C octave (see
    /// [`crate::key_signature::pitch`]).
    pub harm_lev: i64,
    /// Alteration relative to the key signature.
    pub harm_alt: i64,
    pub tie_start: bool,
    pub tie_end: bool,
}

impl Entry {
    pub(crate) fn read(e: &Element) -> Self {
        Self {
            entnum: e.attr_u32("entnum").unwrap_or(0),
            prev: e.attr_u32("prev").unwrap_or(0),
            next: e.attr_u32("next").unwrap_or(0),
            dura: e.int("dura"),
            is_valid: e.flag("isValid"),
            is_note: e.flag("isNote"),
            grace_note: e.flag("graceNote"),
            tuplet_start: e.flag("tupletStart"),
            v2: e.flag("v2"),
            notes: e
                .children_named("note")
                .map(|n| Note {
                    id: n.attr_u32("id").unwrap_or(0),
                    harm_lev: n.int("harmLev"),
                    harm_alt: n.int("harmAlt"),
                    tie_start: n.flag("tieStart"),
                    tie_end: n.flag("tieEnd"),
                })
                .collect(),
        }
    }

    /// Whether the entry is a rest: not a note, or a note with no notes.
    pub fn is_rest(&self) -> bool {
        !self.is_note || self.notes.is_empty()
    }
}
