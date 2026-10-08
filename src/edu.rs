//! EDUs ("ENIGMA durational units"), Finale's unit of time, and the
//! durations and time signatures expressed in them.

/// EDUs in a quarter note.
pub const EDU_PER_QUARTER: i64 = 1024;
/// EDUs in a whole note.
pub const EDU_PER_WHOLE: i64 = 4 * EDU_PER_QUARTER;

/// A symbolic (pre-tuplet) duration's note value and dots: the highest set
/// bit of `dura` is the note value (4096 EDU = whole), each set bit
/// directly below it a dot. Returns the undotted value in EDU.
pub fn note_value(dura: i64) -> (u32, u8) {
    let dura = dura.clamp(1, u32::MAX as i64) as u32;
    let top = 31 - dura.leading_zeros();
    let mut dots = 0;
    let mut bit = top;
    while bit > 0 && dura & (1 << (bit - 1)) != 0 {
        dots += 1;
        bit -= 1;
    }
    (1 << top, dots)
}

/// A time signature's (beats, beat type) from a measure's `beats` x
/// `divbeat` (the beat in EDU). Compound meters store a dotted beat: 6/8
/// is 2 x 1536.
pub fn time_signature(beats: i64, divbeat: i64) -> Option<(i64, i64)> {
    if beats <= 0 || divbeat <= 0 {
        return None;
    }
    let (beats, beat) = if EDU_PER_WHOLE % divbeat == 0 {
        (beats, divbeat)
    } else if divbeat % 3 == 0 && EDU_PER_WHOLE % (divbeat / 3) == 0 {
        (beats * 3, divbeat / 3)
    } else {
        return None;
    };
    Some((beats, EDU_PER_WHOLE / beat))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_values_count_dots_from_the_top_bit() {
        assert_eq!(note_value(1024), (1024, 0));
        assert_eq!(note_value(1536), (1024, 1));
        assert_eq!(note_value(3584), (2048, 2));
        assert_eq!(note_value(4096), (4096, 0));
        assert_eq!(note_value(128), (128, 0));
    }

    #[test]
    fn time_signatures() {
        assert_eq!(time_signature(4, 1024), Some((4, 4)));
        assert_eq!(time_signature(3, 512), Some((3, 8)));
        assert_eq!(time_signature(2, 1536), Some((6, 8)));
        assert_eq!(time_signature(2, 2048), Some((2, 2)));
        assert_eq!(time_signature(0, 1024), None);
    }
}
