use crate::production::*;
use crate::shotliner_document;
use screenplay_doc_parser_rs::{self, screenplay_document};

/// A singular shot entry, which takes up a single row of the worksheet.
pub struct ShotListEntry {
    pub(crate) completed: bool,
    
    pub shot_number: ShotNumber,
    pub shot_type: ShotType,
    pub shot_subtype: ShotSubType,
    pub shot_setup: ShotSetup,

    pub scene_number: screenplay_document::SceneNumber,
    pub scene_environment: screenplay_document::Environment,
    pub scene_time: screenplay_document::TimeOfDay,
    pub scene_location: screenplay_document::LocationID,
    pub scene_sublocation: Option<screenplay_document::LocationID>,


    pub group: String,
    pub characters: String,
    pub tags: String,
    pub props: String,

    pub estimated_setup_time: String,


    
}
