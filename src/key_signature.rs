//! Key signatures (`<keySig><key>`) and the key-relative way Finale stores
//! note pitches.

/// A `<keySig>`: its `<key>` value holds the mode in the high byte (0
/// major, 1 minor, others custom) and the signed fifths in the low byte.
/// Nonlinear (custom) keys have one of the two top bits set.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KeySig {
    pub key: i64,
}

impl KeySig {
    /// A linear key's (fifths, minor); `None` for a nonlinear key.
    pub fn linear(&self) -> Option<(i8, bool)> {
        let value = self.key as u16;
        if value & 0xC000 != 0 {
            return None;
        }
        Some(((value & 0xFF) as u8 as i8, value >> 8 == 1))
    }
}

/// A spelled pitch: diatonic step (C = 0 .. B = 6), alteration in
/// semitones and octave (4 holds middle C).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpelledPitch {
    pub step: u8,
    pub alter: i8,
    pub octave: i8,
}

/// The pitch of a note in a linear key: `harm_lev` counts diatonic steps
/// from the key's tonic in the middle-C octave, `harm_alt` is the
/// alteration relative to the key signature (musxdom's
/// `KeySignature::calcPitch`).
pub fn pitch(harm_lev: i64, harm_alt: i64, fifths: i8, minor: bool) -> SpelledPitch {
    const MAJOR_SHARPS: [i64; 8] = [0, 4, 1, 5, 2, 6, 3, 0];
    const MINOR_SHARPS: [i64; 8] = [5, 2, 6, 3, 0, 4, 1, 5];
    const MAJOR_FLATS: [i64; 8] = [0, 3, 6, 2, 5, 1, 4, 0];
    const MINOR_FLATS: [i64; 8] = [5, 1, 4, 0, 3, 6, 2, 5];
    let fifths = fifths.clamp(-7, 7);
    let index = fifths.unsigned_abs() as usize;
    let tonal_center = match (fifths >= 0, minor) {
        (true, false) => MAJOR_SHARPS[index],
        (true, true) => MINOR_SHARPS[index],
        (false, false) => MAJOR_FLATS[index],
        (false, true) => MINOR_FLATS[index],
    };
    let level = tonal_center + harm_lev;
    let step = level.rem_euclid(7) as usize;
    let octave = level.div_euclid(7) + 4;
    let alter = harm_alt + key_alteration(step, fifths) as i64;
    SpelledPitch {
        step: step as u8,
        alter: alter as i8,
        octave: octave as i8,
    }
}

/// The accidental a key signature puts on `step` (C=0 .. B=6).
fn key_alteration(step: usize, fifths: i8) -> i8 {
    const SHARP_ORDER: [usize; 7] = [3, 0, 4, 1, 5, 2, 6];
    const FLAT_ORDER: [usize; 7] = [6, 2, 5, 1, 4, 0, 3];
    let (order, sign) = if fifths >= 0 {
        (SHARP_ORDER, 1)
    } else {
        (FLAT_ORDER, -1)
    };
    let count = fifths.unsigned_abs() as usize;
    if order[..count].contains(&step) {
        sign
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(step: u8, alter: i8, octave: i8) -> SpelledPitch {
        SpelledPitch {
            step,
            alter,
            octave,
        }
    }

    #[test]
    fn decodes_linear_keys() {
        let key = |key| KeySig { key }.linear();
        assert_eq!(key(0), Some((0, false)));
        assert_eq!(key(253), Some((-3, false)));
        assert_eq!(key(509), Some((-3, true)));
        assert_eq!(key(0x4000), None);
    }

    #[test]
    fn pitches_are_relative_to_the_tonic() {
        assert_eq!(pitch(6, 0, 0, false), p(6, 0, 4));
        assert_eq!(pitch(0, 0, 0, false), p(0, 0, 4));
        // E-flat major: the tonic is E-flat 4, the 4th step up A-flat 4.
        assert_eq!(pitch(0, 0, -3, false), p(2, -1, 4));
        assert_eq!(pitch(3, 0, -3, false), p(5, -1, 4));
        // A minor: the tonic is A 4; one step down, raised, is G-sharp 4.
        assert_eq!(pitch(-1, 1, 0, true), p(4, 1, 4));
        // D major, an octave below: D 3, and its F-sharp naturalized.
        assert_eq!(pitch(-7, 0, 2, false), p(1, 0, 3));
        assert_eq!(pitch(2, -1, 2, false), p(3, 0, 4));
    }
}
