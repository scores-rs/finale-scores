//! The `.musx` container: a zip archive whose `score.dat` entry is the
//! EnigmaXML document, gzip-compressed and then scrambled with a fixed
//! keystream.
//!
//! The scrambling was worked out by Deguerre and is documented in Robert
//! Patterson's `denigma`/`musxdom` projects (MIT): it is BSD `rand()`
//! seeded with a fixed state, reseeded every 128 KiB, XORed over the
//! compressed bytes.

use crate::error::{FinaleError, Result};
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use std::io::{Cursor, Read, Write};

/// The archive entry holding the score.
pub const SCORE_ENTRY: &str = "score.dat";
/// The archive entry naming the file type, as Finale writes it.
pub const MIMETYPE_ENTRY: &str = "mimetype";
pub const MIMETYPE: &str = "application/vnd.makemusic.notation";

const INITIAL_STATE: u32 = 0x2800_6D45;
const RESET_LIMIT: usize = 0x20000;

/// Extracts the EnigmaXML document from a `.musx` archive, or takes
/// `data` as it is if it already is EnigmaXML (as `denigma` extracts it).
pub fn enigma_xml(data: &[u8]) -> Result<String> {
    if !data.starts_with(b"PK") {
        let text = std::str::from_utf8(data).map_err(|_| FinaleError::NotMusx)?;
        if text
            .trim_start_matches('\u{feff}')
            .trim_start()
            .starts_with('<')
        {
            return Ok(text.to_string());
        }
    }
    let mut archive = zip::ZipArchive::new(Cursor::new(data))?;
    let mut scrambled = Vec::new();
    archive
        .by_name(SCORE_ENTRY)
        .map_err(|source| FinaleError::MissingEntry {
            entry: SCORE_ENTRY,
            source,
        })?
        .read_to_end(&mut scrambled)?;
    descramble(&mut scrambled);
    let mut xml = String::new();
    GzDecoder::new(scrambled.as_slice())
        .read_to_string(&mut xml)
        .map_err(|source| FinaleError::Compression {
            entry: SCORE_ENTRY,
            source,
        })?;
    Ok(xml)
}

/// Packs EnigmaXML into a `.musx` archive the way Finale does: a
/// `mimetype` entry, then `score.dat` compressed and scrambled.
pub fn pack(enigma_xml: &str) -> Result<Vec<u8>> {
    let mut gz = GzEncoder::new(Vec::new(), Compression::default());
    gz.write_all(enigma_xml.as_bytes())?;
    let mut dat = gz.finish()?;
    descramble(&mut dat);
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default();
    zip.start_file(MIMETYPE_ENTRY, options)?;
    zip.write_all(MIMETYPE.as_bytes())?;
    zip.start_file(SCORE_ENTRY, options)?;
    zip.write_all(&dat)?;
    Ok(zip.finish()?.into_inner())
}

/// XORs `buffer` with Finale's keystream. The operation is its own
/// inverse, so this also scrambles.
pub fn descramble(buffer: &mut [u8]) {
    let mut state = INITIAL_STATE;
    for (i, byte) in buffer.iter_mut().enumerate() {
        if i % RESET_LIMIT == 0 {
            state = INITIAL_STATE;
        }
        state = state.wrapping_mul(0x41c6_4e6d).wrapping_add(0x3039);
        let upper = state >> 16;
        *byte ^= (upper + upper / 255) as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descrambling_is_its_own_inverse() {
        let original: Vec<u8> = (0..300_000u32).map(|i| (i % 253) as u8).collect();
        let mut buffer = original.clone();
        descramble(&mut buffer);
        assert_ne!(buffer, original);
        descramble(&mut buffer);
        assert_eq!(buffer, original);
    }

    #[test]
    fn packs_and_extracts() {
        let xml = "<finale><texts/></finale>";
        assert_eq!(enigma_xml(&pack(xml).unwrap()).unwrap(), xml);
        // Bare EnigmaXML passes through.
        assert_eq!(enigma_xml(xml.as_bytes()).unwrap(), xml);
        assert!(enigma_xml(b"not a zip").is_err());
    }
}
