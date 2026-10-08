//! The EnigmaXML document, its records indexed by the keys they reference
//! each other with.

use crate::details::{ArticAssign, Floats, Gfhold, TupletDef};
use crate::entries::Entry;
use crate::error::Result;
use crate::others::{
    ArticDef, ClefList, FontName, FrameSpec, InstUsed, MeasExprAssign, MeasSpec, SmartShape,
    StaffSpec, TextBlock, TextExprDef, TimeUpper,
};
use crate::texts::Texts;
use crate::xml::{self, Element};
use std::collections::HashMap;

/// The score's records. Linked parts' records (a non-zero `part`
/// attribute) aren't read: only the score is.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Document {
    pub others: Others,
    pub details: Details,
    /// `<entries>`, by entnum.
    pub entries: HashMap<u32, Entry>,
    pub texts: Texts,
}

/// The `<others>` records this crate reads.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Others {
    pub staff_specs: HashMap<u32, StaffSpec>,
    /// Every staff list's items, in document order.
    pub inst_used: Vec<InstUsed>,
    /// In measure order.
    pub meas_specs: Vec<MeasSpec>,
    /// By (cmper, inci).
    pub frame_specs: HashMap<(u32, u32), FrameSpec>,
    /// By (cmper, inci).
    pub clef_lists: HashMap<(u32, u32), ClefList>,
    pub font_names: HashMap<u32, FontName>,
    pub artic_defs: HashMap<u32, ArticDef>,
    /// By measure, in inci order.
    pub meas_expr_assigns: HashMap<u32, Vec<MeasExprAssign>>,
    pub text_expr_defs: HashMap<u32, TextExprDef>,
    /// In document order.
    pub smart_shapes: Vec<SmartShape>,
    pub time_uppers: HashMap<u32, TimeUpper>,
    pub text_blocks: HashMap<u32, TextBlock>,
}

/// The `<details>` records this crate reads.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Details {
    /// By (staff, measure).
    pub gfholds: HashMap<(u32, u32), Gfhold>,
    /// By (staff, measure).
    pub floats: HashMap<(u32, u32), Floats>,
    /// By entry, in inci order.
    pub tuplet_defs: HashMap<u32, Vec<TupletDef>>,
    /// By entry, in inci order.
    pub artic_assigns: HashMap<u32, Vec<ArticAssign>>,
}

/// A section's records that belong to the score (not a linked part).
fn records<'a>(root: &'a Element, section: &str) -> impl Iterator<Item = &'a Element> {
    root.child(section)
        .into_iter()
        .flat_map(|s| s.children.iter())
        .filter(|r| r.is_score_record())
}

impl Document {
    /// Parses EnigmaXML.
    pub fn parse(xml: &str) -> Result<Self> {
        Ok(Self::from_root(&xml::parse(xml)?))
    }

    fn from_root(root: &Element) -> Self {
        let mut doc = Document::default();
        let others = &mut doc.others;
        for record in records(root, "others") {
            match record.name.as_str() {
                "staffSpec" => {
                    let spec = StaffSpec::read(record);
                    others.staff_specs.insert(spec.cmper, spec);
                }
                "instUsed" => others.inst_used.push(InstUsed::read(record)),
                "measSpec" => others.meas_specs.push(MeasSpec::read(record)),
                "frameSpec" => {
                    let frame = FrameSpec::read(record);
                    others.frame_specs.insert((frame.cmper, frame.inci), frame);
                }
                "clefList" => {
                    let clef = ClefList::read(record);
                    others.clef_lists.insert((clef.cmper, clef.inci), clef);
                }
                "fontName" => {
                    let font = FontName::read(record);
                    // A nameless record leaves an earlier one in place.
                    if font.name.is_some() {
                        others.font_names.insert(font.cmper, font);
                    }
                }
                "articDef" => {
                    let def = ArticDef::read(record);
                    others.artic_defs.insert(def.cmper, def);
                }
                "measExprAssign" => {
                    let assign = MeasExprAssign::read(record);
                    others
                        .meas_expr_assigns
                        .entry(assign.cmper)
                        .or_default()
                        .push(assign);
                }
                "textExprDef" => {
                    let def = TextExprDef::read(record);
                    others.text_expr_defs.insert(def.cmper, def);
                }
                "smartShape" => others.smart_shapes.push(SmartShape::read(record)),
                "timeUpper" => {
                    let upper = TimeUpper::read(record);
                    others.time_uppers.insert(upper.cmper, upper);
                }
                "textBlock" => {
                    let block = TextBlock::read(record);
                    others.text_blocks.insert(block.cmper, block);
                }
                _ => {}
            }
        }
        let details = &mut doc.details;
        for record in records(root, "details") {
            match record.name.as_str() {
                "gfhold" => {
                    let hold = Gfhold::read(record);
                    details.gfholds.insert((hold.cmper1, hold.cmper2), hold);
                }
                "floats" => {
                    let floats = Floats::read(record);
                    details
                        .floats
                        .insert((floats.cmper1, floats.cmper2), floats);
                }
                "tupletDef" => {
                    let def = TupletDef::read(record);
                    details.tuplet_defs.entry(def.entnum).or_default().push(def);
                }
                "articAssign" => {
                    let assign = ArticAssign::read(record);
                    details
                        .artic_assigns
                        .entry(assign.entnum)
                        .or_default()
                        .push(assign);
                }
                _ => {}
            }
        }
        for record in records(root, "entries").filter(|r| r.name == "entry") {
            if record.attr_u32("entnum").is_some() {
                let entry = Entry::read(record);
                doc.entries.insert(entry.entnum, entry);
            }
        }
        let texts = &mut doc.texts;
        for record in records(root, "texts") {
            let text = record.text().to_string();
            match record.name.as_str() {
                "blockText" => {
                    if let Some(number) = record.attr_u32("number") {
                        texts.block_texts.insert(number, text);
                    }
                }
                "expression" => {
                    if let Some(number) = record.attr_u32("number") {
                        texts.expressions.insert(number, text);
                    }
                }
                "fileInfo" => {
                    if let Some(kind) = record.attr("type") {
                        texts.file_info.insert(kind.to_string(), text);
                    }
                }
                _ => {}
            }
        }
        doc.others
            .meas_specs
            .sort_unstable_by_key(|meas| meas.cmper);
        for assigns in doc.others.meas_expr_assigns.values_mut() {
            assigns.sort_by_key(|a| a.inci);
        }
        for defs in doc.details.tuplet_defs.values_mut() {
            defs.sort_by_key(|d| d.inci);
        }
        for assigns in doc.details.artic_assigns.values_mut() {
            assigns.sort_by_key(|a| a.inci);
        }
        doc
    }

    /// Scroll View's staff list (staff list 0): every staff, in score
    /// order. Staves listed but not defined are left out.
    pub fn score_staves(&self) -> Vec<u32> {
        let mut list: Vec<(u32, u32)> = self
            .others
            .inst_used
            .iter()
            .filter(|i| i.cmper == 0)
            .filter_map(|i| Some((i.inci, i.inst?)))
            .collect();
        list.sort_unstable();
        let mut staves: Vec<u32> = list
            .into_iter()
            .map(|(_, staff)| staff)
            .filter(|staff| self.others.staff_specs.contains_key(staff))
            .collect();
        staves.dedup();
        staves
    }

    /// A text block's Enigma string.
    pub fn text_block(&self, block: u32) -> Option<&str> {
        let id = self.others.text_blocks.get(&block)?.text_id;
        self.texts.block_texts.get(&id).map(String::as_str)
    }

    /// A `fileInfo` text's Enigma string (`title`, `composer`, ...).
    pub fn file_info(&self, kind: &str) -> Option<&str> {
        self.texts.file_info.get(kind).map(String::as_str)
    }

    /// A text expression's Enigma string, by `textExprDef` id.
    pub fn text_expression(&self, def: u32) -> Option<&str> {
        let key = self.others.text_expr_defs.get(&def)?.text_id_key;
        self.texts.expressions.get(&key).map(String::as_str)
    }

    /// A font family by `fontName` id.
    pub fn font_name(&self, id: u32) -> Option<&str> {
        self.others.font_names.get(&id)?.name.as_deref()
    }

    /// The first clef of a clef list.
    pub fn clef_list(&self, id: u32) -> Option<&ClefList> {
        self.others.clef_lists.get(&(id, 0))
    }

    /// A frame's entries in time order, walked through their `next` links
    /// (entry numbers aren't in time order once a score has been edited).
    pub fn frame_entries(&self, frame: u32) -> Vec<&Entry> {
        // Files from old Finale versions keep the range in inci 1, with
        // inci 0 holding a pickup's start offset.
        let Some(spec) = (0..2)
            .filter_map(|inci| self.others.frame_specs.get(&(frame, inci)))
            .find(|f| f.start_entry.is_some())
        else {
            return Vec::new();
        };
        let end = spec.end_entry;
        let mut id = spec.start_entry.unwrap_or(0);
        let mut entries = Vec::new();
        while let Some(entry) = self.entries.get(&id) {
            entries.push(entry);
            // The length check guards against a corrupt cyclic chain.
            if id == end || entries.len() > self.entries.len() {
                break;
            }
            id = entry.next;
        }
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<finale version="27.4" xmlns="http://www.makemusic.com/2012/finale">
  <others>
    <frameSpec cmper="1" inci="0"><startEntry>1</startEntry><endEntry>3</endEntry></frameSpec>
    <instUsed cmper="0" inci="1"><inst>2</inst></instUsed>
    <instUsed cmper="0" inci="0"><inst>1</inst></instUsed>
    <instUsed cmper="0" inci="2"><inst>9</inst></instUsed>
    <staffSpec cmper="1"><fullName>4</fullName><defaultClef>3</defaultClef></staffSpec>
    <staffSpec cmper="2"/>
    <measSpec cmper="2"><beats>3</beats><divbeat>1024</divbeat></measSpec>
    <measSpec cmper="1"><beats>4</beats><divbeat>1024</divbeat><keySig><key>253</key></keySig></measSpec>
    <textBlock cmper="4"><textID>7</textID></textBlock>
  </others>
  <details>
    <gfhold cmper1="1" cmper2="1"><clefID>3</clefID><frame1>1</frame1></gfhold>
    <gfhold cmper1="1" cmper2="1" part="1" shared="false"><clefID>0</clefID></gfhold>
  </details>
  <entries>
    <entry entnum="3" prev="1" next="0"><dura>1024</dura><isValid/></entry>
    <entry entnum="1" prev="0" next="3"><dura>1024</dura><isValid/><isNote/>
      <note id="1"><harmLev>2</harmLev><tieStart/></note></entry>
  </entries>
  <texts>
    <blockText number="7">^fontid(1)Flute</blockText>
    <fileInfo type="title">Song</fileInfo>
  </texts>
</finale>"#;

    #[test]
    fn indexes_records() {
        let doc = Document::parse(XML).unwrap();
        assert_eq!(doc.score_staves(), vec![1, 2]);
        let measures: Vec<u32> = doc.others.meas_specs.iter().map(|m| m.cmper).collect();
        assert_eq!(measures, vec![1, 2]);
        assert_eq!(
            doc.others.meas_specs[0].key_sig.unwrap().linear(),
            Some((-3, false))
        );
        assert_eq!(doc.others.staff_specs[&1].default_clef, 3);
        // The score's gfhold, not linked part 1's.
        let hold = &doc.details.gfholds[&(1, 1)];
        assert_eq!(hold.clef_id, 3);
        assert_eq!(hold.frames, [1, 0, 0, 0]);
        let entries: Vec<u32> = doc.frame_entries(1).iter().map(|e| e.entnum).collect();
        assert_eq!(entries, vec![1, 3]);
        let note = doc.entries[&1].notes[0];
        assert_eq!((note.harm_lev, note.tie_start), (2, true));
        assert!(doc.entries[&3].is_rest());
        assert_eq!(doc.text_block(4), Some("^fontid(1)Flute"));
        assert_eq!(doc.file_info("title"), Some("Song"));
    }
}
