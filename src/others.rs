//! Records of the `<others>` section: score-wide definitions keyed by
//! `cmper` (and `inci` where a key has several).

use crate::key_signature::KeySig;
use crate::xml::Element;

/// `staffSpec`: a staff's settings.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StaffSpec {
    pub cmper: u32,
    /// The `textBlock` of the staff's full name.
    pub full_name: u32,
    /// Index into the clef table.
    pub default_clef: i64,
    /// The staff has its own time signatures (in `details/floats`).
    pub float_time: bool,
    /// The staff has its own key signatures (in `details/floats`).
    pub float_keys: bool,
    pub transposition: Option<StaffTransposition>,
}

/// A staff's `<transposition>`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct StaffTransposition {
    pub keysig: Option<KeySigTransposition>,
    /// `<noKeyOpt/>`: written keys aren't simplified.
    pub no_key_opt: bool,
}

/// Key-signature transposition (`<keysig><interval>`/`<adjust>`): written
/// notes are `interval` diatonic steps above concert pitch, and the
/// written key `adjust` fifths sharper. A B♭ clarinet is 1/2.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KeySigTransposition {
    pub interval: i64,
    pub adjust: i64,
}

impl KeySigTransposition {
    /// The written-to-sounding interval as (diatonic steps, semitones),
    /// if it's a whole number of semitones.
    pub fn sounding_interval(&self) -> Option<(i8, i8)> {
        // fifths = 7 * chromatic - 12 * diatonic, for the written-to-sounding
        // interval, i.e. -adjust = 7 * chromatic + 12 * interval.
        let chromatic = -self.adjust - 12 * self.interval;
        if chromatic % 7 != 0 {
            return None;
        }
        Some((
            i8::try_from(-self.interval).ok()?,
            i8::try_from(chromatic / 7).ok()?,
        ))
    }

    /// A transposing staff's written key: the concert key moved by
    /// `adjust` fifths. Unless the staff opts out (`simplify` false, from
    /// `<noKeyOpt/>`), Finale simplifies a written key of 7 or more
    /// accidentals to its enharmonic equivalent (D-sharp major to E-flat
    /// major), which also respells every note by a letter step: returns
    /// the key and that step.
    pub fn written_key(&self, concert_fifths: i8, simplify: bool) -> (i8, i32) {
        let mut fifths = concert_fifths as i64 + self.adjust;
        let mut respell = 0;
        if simplify && self.adjust != 0 {
            while fifths.abs() >= 7 {
                let direction = fifths.signum();
                fifths -= 12 * direction;
                respell += direction as i32;
            }
        }
        (fifths as i8, respell)
    }
}

/// `instUsed`: one staff in a staff list. Staff list 0 is Scroll View's,
/// which holds every staff in score order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InstUsed {
    pub cmper: u32,
    pub inci: u32,
    /// The staff (`staffSpec` cmper).
    pub inst: Option<u32>,
}

/// A time signature as `measSpec` and `floats` store it.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TimeSig {
    /// Beats, or with `alt_num` the `timeUpper` composite numerator.
    pub beats: i64,
    /// The beat in EDU.
    pub divbeat: i64,
    /// `beats` is a composite numerator (2+2+3).
    pub alt_num: bool,
    /// The denominator is composite (3/8+2/4).
    pub alt_den: bool,
}

impl TimeSig {
    fn read(e: &Element, divbeat: &str) -> Self {
        Self {
            beats: e.int("beats"),
            divbeat: e.int(divbeat),
            alt_num: e.flag("altNumTsig"),
            alt_den: e.flag("altDenTsig"),
        }
    }
}

/// `measSpec`: a measure, keyed by its number.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MeasSpec {
    pub cmper: u32,
    pub time: TimeSig,
    pub key_sig: Option<KeySig>,
    /// The right barline style; `None` for no barline.
    pub barline: Option<String>,
    /// A forward repeat at the start.
    pub for_rep_bar: bool,
    /// A backward repeat at the end.
    pub bac_rep_bar: bool,
}

/// `frameSpec`: a frame's (one layer of one measure on one staff) range
/// of entries, `startEntry` to `endEntry` along the entries' `next` links.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FrameSpec {
    pub cmper: u32,
    pub inci: u32,
    pub start_entry: Option<u32>,
    pub end_entry: u32,
}

/// `clefList`: one clef of a mid-measure clef change.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClefList {
    pub cmper: u32,
    pub inci: u32,
    /// Index into the clef table.
    pub clef: i64,
}

/// `fontName`: a font family, referenced by id (0: the default music
/// font).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FontName {
    pub cmper: u32,
    pub name: Option<String>,
}

/// `articDef`: an articulation symbol.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ArticDef {
    pub cmper: u32,
    /// The main symbol's character, in `font_main`.
    pub char_main: u32,
    /// `fontName` id.
    pub font_main: u32,
    /// The main symbol is a shape, not a character.
    pub main_is_shape: bool,
    /// An arpeggio/trill-extension "copy" articulation.
    pub copy_mode: bool,
}

/// `measExprAssign`: an expression placed in a measure (cmper).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MeasExprAssign {
    pub cmper: u32,
    pub inci: u32,
    /// `textExprDef` id.
    pub text_expr_id: u32,
    /// Position in the measure, in EDU.
    pub horz_edu_off: i64,
    /// The staff: a `staffSpec` cmper, or -1 for the top staff, -2 for the
    /// bottom one.
    pub staff_assign: i64,
    /// The layer it applies to (0: all).
    pub layer: u32,
    pub hidden: bool,
    /// Which staves show it (`partOnly`, ...).
    pub show_staff_list: Option<String>,
}

/// `textExprDef`: a text expression's definition.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TextExprDef {
    pub cmper: u32,
    /// The `texts/expression` holding its text.
    pub text_id_key: u32,
}

/// `smartShape`: a slur, hairpin, line, ...
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SmartShape {
    pub cmper: u32,
    /// `slurAuto`, `cresc`, ... (trimmed).
    pub shape_type: Option<String>,
    pub hidden: bool,
    /// The ends are attached to entries rather than beats.
    pub entry_based: bool,
    pub start_term_seg: Option<TermSeg>,
    pub end_term_seg: Option<TermSeg>,
}

/// A smart shape's start or end segment.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TermSeg {
    pub end_pt: Option<EndPt>,
    pub end_pt_adj: Option<EndPtAdj>,
}

/// Where a smart shape's end is attached.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct EndPt {
    /// The staff.
    pub inst: u32,
    /// The measure.
    pub meas: u32,
    /// The position in the measure (beat-attached shapes), in EDU.
    pub edu: i64,
    /// The entry (entry-attached shapes).
    pub entry_num: u32,
}

/// A manual adjustment of a smart shape's end.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct EndPtAdj {
    pub y: i64,
}

/// `timeUpper`: a composite time signature's numerator, one item per
/// added group (2+2+3).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TimeUpper {
    pub cmper: u32,
    pub items: Vec<TimeUpperItem>,
}

/// One group of a composite numerator: `integer` beats plus a fraction.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TimeUpperItem {
    pub integer: i64,
    pub frac: i64,
}

impl TimeUpper {
    /// The total beats, if every group is whole beats (2+2+3 is 7).
    pub fn whole_beats(&self) -> Option<i64> {
        let mut total = 0;
        for item in &self.items {
            if item.frac != 0 {
                return None;
            }
            total += item.integer;
        }
        (total > 0).then_some(total)
    }
}

/// `textBlock`: a block of text, its content in `texts/blockText`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TextBlock {
    pub cmper: u32,
    pub text_id: u32,
}

fn cmper(e: &Element) -> u32 {
    e.attr_u32("cmper").unwrap_or(0)
}

fn inci(e: &Element) -> u32 {
    e.attr_u32("inci").unwrap_or(0)
}

impl StaffSpec {
    pub(crate) fn read(e: &Element) -> Self {
        let transposition = e.child("transposition").map(|t| StaffTransposition {
            keysig: t.child("keysig").map(|k| KeySigTransposition {
                interval: k.int("interval"),
                adjust: k.int("adjust"),
            }),
            no_key_opt: t.flag("noKeyOpt"),
        });
        Self {
            cmper: cmper(e),
            full_name: e.int("fullName") as u32,
            default_clef: e.int("defaultClef"),
            float_time: e.flag("floatTime"),
            float_keys: e.flag("floatKeys"),
            transposition,
        }
    }
}

impl InstUsed {
    pub(crate) fn read(e: &Element) -> Self {
        Self {
            cmper: cmper(e),
            inci: inci(e),
            inst: e.child_text("inst").and_then(|t| t.parse().ok()),
        }
    }
}

impl MeasSpec {
    pub(crate) fn read(e: &Element) -> Self {
        Self {
            cmper: cmper(e),
            time: TimeSig::read(e, "divbeat"),
            key_sig: e.child("keySig").map(|k| KeySig { key: k.int("key") }),
            barline: e.child_text("barline").map(str::to_string),
            for_rep_bar: e.flag("forRepBar"),
            bac_rep_bar: e.flag("bacRepBar"),
        }
    }
}

impl FrameSpec {
    pub(crate) fn read(e: &Element) -> Self {
        Self {
            cmper: cmper(e),
            inci: inci(e),
            start_entry: e.child("startEntry").map(|_| e.int("startEntry") as u32),
            end_entry: e.int("endEntry") as u32,
        }
    }
}

impl ClefList {
    pub(crate) fn read(e: &Element) -> Self {
        Self {
            cmper: cmper(e),
            inci: inci(e),
            clef: e.int("clef"),
        }
    }
}

impl FontName {
    pub(crate) fn read(e: &Element) -> Self {
        Self {
            cmper: cmper(e),
            name: e.child_text("name").map(str::to_string),
        }
    }
}

impl ArticDef {
    pub(crate) fn read(e: &Element) -> Self {
        Self {
            cmper: cmper(e),
            char_main: e.int("charMain") as u32,
            font_main: e.int("fontMain") as u32,
            main_is_shape: e.flag("mainIsShape"),
            copy_mode: e.flag("copyMode"),
        }
    }
}

impl MeasExprAssign {
    pub(crate) fn read(e: &Element) -> Self {
        Self {
            cmper: cmper(e),
            inci: inci(e),
            text_expr_id: e.int("textExprID") as u32,
            horz_edu_off: e.int("horzEduOff"),
            staff_assign: e.int("staffAssign"),
            layer: e.int("layer") as u32,
            hidden: e.flag("hidden"),
            show_staff_list: e.child_text("showStaffList").map(str::to_string),
        }
    }
}

impl TextExprDef {
    pub(crate) fn read(e: &Element) -> Self {
        Self {
            cmper: cmper(e),
            text_id_key: e.int("textIDKey") as u32,
        }
    }
}

impl SmartShape {
    pub(crate) fn read(e: &Element) -> Self {
        let segment = |name: &str| {
            e.child(name).map(|s| TermSeg {
                end_pt: s.child("endPt").map(|p| EndPt {
                    inst: p.int("inst") as u32,
                    meas: p.int("meas") as u32,
                    edu: p.int("edu"),
                    entry_num: p.int("entryNum") as u32,
                }),
                end_pt_adj: s.child("endPtAdj").map(|a| EndPtAdj { y: a.int("y") }),
            })
        };
        Self {
            cmper: cmper(e),
            shape_type: e.child_text("shapeType").map(|t| t.trim().to_string()),
            hidden: e.flag("hidden"),
            entry_based: e.flag("entryBased"),
            start_term_seg: segment("startTermSeg"),
            end_term_seg: segment("endTermSeg"),
        }
    }
}

impl TimeUpper {
    pub(crate) fn read(e: &Element) -> Self {
        Self {
            cmper: cmper(e),
            items: e
                .children_named("tudata")
                .map(|t| TimeUpperItem {
                    integer: t.int("integer"),
                    frac: t.int("frac"),
                })
                .collect(),
        }
    }
}

impl TextBlock {
    pub(crate) fn read(e: &Element) -> Self {
        Self {
            cmper: cmper(e),
            text_id: e.int("textID") as u32,
        }
    }
}

/// `floats` (in `details`) shares measSpec's time signature fields.
pub(crate) fn float_time(e: &Element) -> TimeSig {
    TimeSig::read(e, "divBeat")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn written_keys_simplify() {
        let t = |adjust| KeySigTransposition {
            interval: 0,
            adjust,
        };
        // Concert C-sharp major on a B-flat instrument: D-sharp is E-flat.
        assert_eq!(t(2).written_key(7, true), (-3, 1));
        assert_eq!(t(2).written_key(7, false), (9, 0));
        // Concert C-flat major on an A instrument: E-double-flat is D.
        assert_eq!(t(-3).written_key(-7, true), (2, -1));
        // Concert B major on a B-flat instrument: C-sharp is D-flat.
        assert_eq!(t(2).written_key(5, true), (-5, 1));
        // Concert keys aren't touched.
        assert_eq!(t(0).written_key(7, true), (7, 0));
    }

    #[test]
    fn sounding_intervals() {
        let t = |interval, adjust| KeySigTransposition { interval, adjust }.sounding_interval();
        assert_eq!(t(1, 2), Some((-1, -2))); // B-flat clarinet
        assert_eq!(t(8, 2), Some((-8, -14))); // bass clarinet
        assert_eq!(t(5, 3), Some((-5, -9))); // alto sax
        assert_eq!(t(-1, -2), Some((1, 2))); // D trumpet
    }

    #[test]
    fn composite_numerators() {
        let upper = |items: &[(i64, i64)]| TimeUpper {
            cmper: 1,
            items: items
                .iter()
                .map(|&(integer, frac)| TimeUpperItem { integer, frac })
                .collect(),
        };
        assert_eq!(upper(&[(2, 0), (2, 0), (3, 0)]).whole_beats(), Some(7));
        assert_eq!(upper(&[(2, 0), (1, 512)]).whole_beats(), None);
        assert_eq!(upper(&[]).whole_beats(), None);
    }
}
