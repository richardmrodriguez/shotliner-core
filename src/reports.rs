use std::collections::HashSet;
use std::ops::Range;

use crate::shotliner_document::{self, ShotlinerDoc};
use crate::{production, serializables};
use screenplay_doc_parser_rs::screenplay_document;

// FOR REFERENCE...
pub struct ShotListEntry<'a> {
    pub shot: &'a production::Shot,
    pub scene: &'a screenplay_document::Scene,

    pub group: shotliner_document::Group,
    pub characters: String,
    pub tags: Vec<&'a shotliner_document::Tag>,
    pub props: Vec<&'a production::Prop>,

    pub estimated_setup_time: String,
    pub completed: bool,
}

pub struct ShotList(Vec<production::Shot>);


pub struct SceneStrip<'a> {
    pub scene: &'a screenplay_document::Scene, // gives us scene number, location, environment, and time of day
    pub story_day: Option<u32>,
    pub page_span: Range<usize>,
    pub pages_eigths: (u32, u32),
    pub cast_in_scene: HashSet<screenplay_document::Character>,
    pub production_locations: HashSet<production::ProductionLocation>,
    pub estimated_duration: chrono::Duration,
    pub completed: bool,
}

pub enum StripBoardEntry<'a> {
    Scene(SceneStrip<'a>),
    Banner(String),
    DayBreak(chrono::NaiveDate), // TODO: how to account for time zones??
    CompanyMove(chrono::Duration), // TODO: company move likely needs other things...
    Other,
}

pub struct StripBoard<'a> {
    pub entries: Vec<StripBoardEntry<'a>>,
}


pub fn get_shotlist(shotliner_doc: &ShotlinerDoc) -> Option<ShotList> {
    None

}

pub fn get_stripboard_for_document(shotliner_doc: &ShotlinerDoc) -> Option<StripBoard> {
    None
}