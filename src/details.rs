//! Records of the `<details>` section: data attached to a staff and
//! measure (`cmper1`/`cmper2`) or to an entry (`entnum`), with `inci` for
//! several of a kind.

use crate::key_signature::KeySig;
use crate::others::{TimeSig, float_time};
use crate::xml::Element;

/// `gfhold`: a staff's content in one measure - its clef and the frame of
/// each of the four layers.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Gfhold {
    /// The staff.
    pub cmper1: u32,
    /// The measure.
    pub cmper2: u32,
    /// Index into the clef table, when there's no clef list.
    pub clef_id: i64,
    /// The measure has mid-measure clef changes, in this `clefList`
    /// (`None` if the id can't be read).
    pub clef_list_id: Option<Option<u32>>,
    /// The frame (`frameSpec` cmper) of layers 1 to 4; 0 for none.
    pub frames: [u32; 4],
}

/// `floats`: a staff's independent time and key signature in one measure.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Floats {
    pub cmper1: u32,
    pub cmper2: u32,
    pub has_time: bool,
    pub time: TimeSig,
    pub has_key: bool,
    pub key_sig: Option<KeySig>,
}

/// `tupletDef`: a tuplet starting at an entry: `symbolic_num` notes of
/// `symbolic_dur` EDU in the time of `ref_num` notes of `ref_dur` EDU.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TupletDef {
    pub entnum: u32,
    pub inci: u32,
    pub symbolic_num: i64,
    pub symbolic_dur: i64,
    pub ref_num: i64,
    pub ref_dur: i64,
}

impl TupletDef {
    /// The tuplet's (actual, normal) ratio, expressed in its own note value
    /// where possible (3 eighths in the time of a quarter is 3:2, not 3:1).
    pub fn ratio(&self) -> Option<(i64, i64)> {
        let (num, dur) = (self.symbolic_num, self.symbolic_dur);
        if num <= 0 || dur <= 0 || self.ref_num <= 0 || self.ref_dur <= 0 {
            return None;
        }
        let reference = self.ref_num * self.ref_dur;
        Some(if reference % dur == 0 {
            (num, reference / dur)
        } else {
            let actual = num * dur;
            let divisor = gcd(actual, reference);
            (actual / divisor, reference / divisor)
        })
    }

    /// A tuplet with no reference duration takes its entries out of time.
    pub fn is_zero(&self) -> bool {
        self.ref_num * self.ref_dur == 0
    }
}

pub(crate) fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 {
        a.abs().max(1)
    } else {
        gcd(b, a % b)
    }
}

/// `articAssign`: an articulation on an entry.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ArticAssign {
    pub entnum: u32,
    pub inci: u32,
    /// `articDef` id.
    pub artic_def: u32,
    pub hide: bool,
}

fn attr(e: &Element, name: &str) -> u32 {
    e.attr_u32(name).unwrap_or(0)
}

impl Gfhold {
    pub(crate) fn read(e: &Element) -> Self {
        Self {
            cmper1: attr(e, "cmper1"),
            cmper2: attr(e, "cmper2"),
            clef_id: e.int("clefID"),
            clef_list_id: e
                .child("clefListID")
                .map(|list| list.text().trim().parse().ok()),
            frames: [1, 2, 3, 4].map(|layer| e.int(&format!("frame{layer}")) as u32),
        }
    }
}

impl Floats {
    pub(crate) fn read(e: &Element) -> Self {
        Self {
            cmper1: attr(e, "cmper1"),
            cmper2: attr(e, "cmper2"),
            has_time: e.flag("hasTime"),
            time: float_time(e),
            has_key: e.flag("hasKey"),
            key_sig: e.child("keySig").map(|k| KeySig { key: k.int("key") }),
        }
    }
}

impl TupletDef {
    pub(crate) fn read(e: &Element) -> Self {
        Self {
            entnum: attr(e, "entnum"),
            inci: attr(e, "inci"),
            symbolic_num: e.int("symbolicNum"),
            symbolic_dur: e.int("symbolicDur"),
            ref_num: e.int("refNum"),
            ref_dur: e.int("refDur"),
        }
    }
}

impl ArticAssign {
    pub(crate) fn read(e: &Element) -> Self {
        Self {
            entnum: attr(e, "entnum"),
            inci: attr(e, "inci"),
            artic_def: e.int("articDef") as u32,
            hide: e.flag("hide"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def(symbolic_num: i64, symbolic_dur: i64, ref_num: i64, ref_dur: i64) -> TupletDef {
        TupletDef {
            symbolic_num,
            symbolic_dur,
            ref_num,
            ref_dur,
            ..Default::default()
        }
    }

    #[test]
    fn tuplet_ratios() {
        assert_eq!(def(3, 1024, 2, 1024).ratio(), Some((3, 2)));
        assert_eq!(def(3, 512, 1, 1024).ratio(), Some((3, 2)));
        assert_eq!(def(5, 768, 1, 2048).ratio(), Some((15, 8)));
        assert_eq!(def(3, 512, 0, 0).ratio(), None);
        assert!(def(3, 512, 0, 0).is_zero());
    }
}
