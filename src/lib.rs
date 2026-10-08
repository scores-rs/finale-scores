//! Read Finale `.musx` score files and the EnigmaXML inside them.
//!
//! `.musx` is the file format of Finale 2014 and later (Finale was
//! discontinued in 2024). It is undocumented: a zip archive whose
//! `score.dat` entry is the score as EnigmaXML, gzip-compressed and
//! scrambled with a fixed keystream. EnigmaXML in turn is a database dump
//! rather than a nested document - flat records that reference each other
//! by number. This crate follows the reverse-engineering of Robert
//! Patterson's MIT-licensed [`musxdom`] and [`denigma`] projects, and
//! Deguerre's work on the scrambling.
//!
//! # Reading
//!
//! ```no_run
//! use finale_scores::{MusxFile, enigma_string};
//!
//! let file = MusxFile::open("song.musx")?;
//! let doc = &file.document;
//! if let Some(title) = doc.file_info("title") {
//!     println!("{}", enigma_string::plain_text(title));
//! }
//! for staff in doc.score_staves() {
//!     for measure in &doc.others.meas_specs {
//!         let Some(hold) = doc.details.gfholds.get(&(staff, measure.cmper)) else {
//!             continue;
//!         };
//!         for entry in doc.frame_entries(hold.frames[0]) {
//!             println!("staff {staff} measure {}: {} EDU", measure.cmper, entry.dura);
//!         }
//!     }
//! }
//! # Ok::<(), finale_scores::FinaleError>(())
//! ```
//!
//! # Organization
//!
//! [`MusxFile`] is the file, [`container`] its archive and scrambling,
//! [`Document`] the EnigmaXML inside it. The document's sections each have
//! a module with their records, named after their EnigmaXML elements:
//!
//! - [`others`] - score-wide definitions keyed by `cmper` (and `inci`):
//!   staves, measures, frames, clef lists, fonts, articulation and
//!   expression definitions, smart shapes, text blocks.
//! - [`details`] - data on a staff and measure (`cmper1`/`cmper2`) or an
//!   entry (`entnum`): a measure's layers and clef, independent key and
//!   time signatures, tuplets, articulation assignments.
//! - [`entries`] - the entries (notes, chords, rests) themselves.
//! - [`texts`] - text content, as Enigma strings with inline formatting
//!   commands, which [`enigma_string`] decodes.
//!
//! [`edu`] and [`key_signature`] hold the encodings of time (EDUs) and
//! of key-relative pitch those records use.
//!
//! Only the records this crate's users have needed are read so far, and
//! only the score's: linked parts' records are skipped.
//!
//! [`musxdom`]: https://github.com/rpatters1/musxdom
//! [`denigma`]: https://github.com/rpatters1/denigma

pub mod container;
pub mod details;
mod document;
pub mod edu;
pub mod enigma_string;
pub mod entries;
mod error;
pub mod key_signature;
pub mod others;
pub mod texts;
mod xml;

pub use document::{Details, Document, Others};
pub use error::{FinaleError, Result};

use std::path::Path;

/// A `.musx` file (or bare EnigmaXML, as `denigma` extracts it).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MusxFile {
    pub document: Document,
}

impl MusxFile {
    /// Reads a `.musx` archive, or EnigmaXML.
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let xml = container::enigma_xml(data)?;
        Ok(Self {
            document: Document::parse(&xml)?,
        })
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_bytes(&std::fs::read(path)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_archives_and_bare_xml() {
        let xml = r#"<finale><texts><fileInfo type="title">T</fileInfo></texts></finale>"#;
        let packed = container::pack(xml).unwrap();
        let file = MusxFile::from_bytes(&packed).unwrap();
        assert_eq!(file.document.file_info("title"), Some("T"));
        assert_eq!(MusxFile::from_bytes(xml.as_bytes()).unwrap(), file);
        assert!(MusxFile::from_bytes(b"not a zip").is_err());
    }
}
