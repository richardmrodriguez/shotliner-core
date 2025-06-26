use std::{collections::HashMap, fmt::Error};
use screenplay_doc_parser_rs::screenplay_document::{self, ScreenplayDocument};
use uuid::Uuid;

use crate::document::{TaggedElement};


pub mod production {
    use std::ops::{Deref, DerefMut};

    use uuid::Uuid;

    
    #[derive(Clone)]
    pub enum ProductionDepartments {
        Production,
        Art,
        Costumes,
        Makeup,
        Camera,
        Props,
        SpecialFX,
        Stunts,
        Animals,
        Vehicles,

    }
    
    #[derive(Clone)]
    pub enum ShotType {
        XWS,
        WS,
        MS,
        CU,
        XCU,
    }

    //TODO: Make these better, maybe split up more categories    
    #[derive(Clone)]
    pub enum ShotSubType {
        //TwoShot, should just be a separate "number_of_subjects" in a higher struct
        Trucking,
        Moving,
        Dolly,
        WhipPan,
        Panning,
        Other,

    }
    
    
    ///Represents a specific, discrete position to place the camera.    
    #[derive(Clone)]
    pub struct ShotSetup {
        index: u64, // simple numerical counter
        id: String, // uuid?
    } 
    
    #[derive(Clone)]
    pub struct ShotNumber(pub String);


    #[derive(Clone, PartialEq)]
    pub struct ShotID(Uuid);
    impl Deref for ShotID {
        type Target = Uuid;
        fn deref(&self) -> &Self::Target {
            &self.0
        }

    }
    impl DerefMut for ShotID {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    #[derive(Clone)]
    pub struct Shot {
        pub id: ShotID,

        pub scene_id: screenplay_doc_parser_rs::screenplay_document::SceneID, // scenes get a UUID because they can have alphanumeric scene number...
        pub shot_number: ShotNumber,

        // Shot Composition (angle, staging, movement, etc.)
        pub shot_type: ShotType,
        pub subtype: Option<ShotSubType>,
        pub setup: ShotSetup, 
        
        // Technical Metadata
        pub camera_metadata: Option<CameraMetadata>,
        
        pub tags: Vec<String>,
        pub media: Vec<crate::multimedia::MediaLink>
    }
    
    #[derive(Clone)]
    pub struct CameraMetadata {
        lens_mm: u64,
        //Camera body, make, model, resolution, codec, etc.
    
    }
}

/// Structs for export types, like Shot Lists or Storyboard Templates
pub mod serializables {

    use crate::{production::{ShotNumber, ShotSetup, ShotSubType, ShotType}};
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
        pub scene_location: screenplay_document::Location,
        pub scene_sublocation: Option<screenplay_document::Location>,


        pub group: String,
        pub characters: String,
        pub tags: String,
        pub props: String,

        pub estimated_setup_time: String,


        
    }

}

/// Structs and Methods for storing types of media that may be linked or embedded into the ShotLiner document.
pub mod multimedia {

    #[derive(Clone)]
    pub enum MediaType {
        Image,
        // could add other media types down the road...
    }

    #[derive(Clone)]
    pub struct MediaLink {
        filepath: String,
        media_type: MediaType,
    }


}

pub mod document {
    use std::{collections::HashMap, fmt::Error};

    use screenplay_doc_parser_rs::screenplay_document::{ScreenplayDocument, TextElement, ScreenplayCoordinate};
    use uuid::Uuid;

    use crate::{add_shotline, commands, multimedia::{MediaLink, MediaType}, remove_shotline};
    use crate::production;

    //TODO: this will be used later, when we implement merge-forward    
    #[derive(Clone)]
    pub struct SmartScreenplayCoordinate {

    }
    
    
    /// A Tag is a finite Screenplay Element or range of Elements, which correspond to one or more Departments.    
    #[derive(Clone)]    
    pub struct Tag {
        pub tag_str: String, // FIXME: TODO: We need to be able to search tags by ID as well as by production and/or screenplay element.
        // fuck.
        pub departments: Vec<production::ProductionDepartments>,
        //pub other_metadata: idk
    }    
    #[derive(Clone)]
    pub struct TaggedElement {
        origin: ScreenplayCoordinate,
        endpoint: ScreenplayCoordinate, //inclusive
        tags: Vec<Uuid> // tags are found / stored lazily; find tags by referencing the Annotation Map; Don't duplicate tag structs, just IDs
        // NOTE: if a UUID doesn't exist when invoking a tag search, DELETE it from the TaggedElement Vec
    }

    #[derive(Clone)]
    pub struct ShotLine {
        pub start: ScreenplayCoordinate,
        pub end: ScreenplayCoordinate,
        pub unfilmed_lines: Vec<ScreenplayCoordinate>,
        pub shot: production::Shot,
    }

    #[derive(Clone)]
    pub struct AnnotationMap {
        pub shotlines: HashMap<Uuid, ShotLine>,
        pub tags: HashMap<Uuid, Tag>,
        pub tagged_elements: HashMap<Uuid, TaggedElement>,

    }
    
    //#[derive(Clone)]
    pub struct ShotlinerDoc {
        pub command_history: crate::commands::CommandHistory,
        pub screenplay: ScreenplayDocument,
        pub annotation_map: AnnotationMap
    }
    impl ShotlinerDoc {
        fn command_exec(&mut self, cmd: &commands::Command) -> Result<(), Error> {
            use commands::Command::*;
            match cmd {
                AddShotline(id,sl) => {
                    return add_shotline(self, sl.clone(), id.clone());
                }
                ModifyShotline(id,sl_opt) => {
                    return Ok(());
                }
                _ => {
                    return Err(Error);
                }
            }
            
            Err(Error)
        } 

        fn command_undo(&mut self, cmd: &commands::Command<>) -> Result<(), Error>{
            match cmd {

                commands::Command::AddShotline(id,sl) => {
                    let id = id.clone();
                    if let Ok(_) = remove_shotline(self, id) {
                        return Ok(());

                    }
                    return Err(Error);


                }
                _ => {
                    return Err(Error);
                }
            }
        }
    }

    
}

pub mod commands {
    use std::{collections::VecDeque, fmt::Error};

    use uuid::Uuid;

    use crate::document::{ShotLine, Tag};
    pub enum CommandHistoryStatus {
        ExecuteSuccess,
        UndoSuccess,
        UndoLimitReached,
        RedoLimitReached,
    }

    pub struct CommandHistory {
        pub index: u64,
        pub history: VecDeque<Command>,
        pub max_history_size: u64,
        pub current_history_size: u64,
    }
    impl CommandHistory {
        fn execute(&mut self) -> Result<CommandHistoryStatus, Error> {
            if self.index < self.max_history_size {
                self.index = self.index + 1;
                return Ok(CommandHistoryStatus::ExecuteSuccess);
            }
            else {
                return Err(Error);
            }
        }
        fn undo(&mut self) {
            if self.index > 0 {
                self.index = self.index -1;
            }
            else {
                // TODO:     
            }
        }
        fn redo(&mut self) {
            if self.index + 1 <= self.current_history_size {
                self.index = self.index+1;
            }
        }
    }
    
    pub enum Command {
        AddShotline(Uuid, ShotLine),
        ModifyShotline(Uuid, Option<ShotLine>), // takes ID, old shotline
        RemoveShotline(Uuid, Option<ShotLine>), // takes ID, old shotline

        AddTag(Uuid, Tag),
        ModifyTag(Uuid, Option<Tag>),
        RemoveTag(Uuid, Option<Tag>),
    }

}

pub fn get_shotlines_as_table(shotlines: &HashMap<Uuid, document::ShotLine>, screenplay_doc: &ScreenplayDocument) -> Option<Vec<serializables::ShotListEntry>> {
    use serializables::ShotListEntry;
    let mut new_table = Vec::<ShotListEntry>::new();
    for (_, shotline) in shotlines {
        let shot = &shotline.shot;
        let scene = screenplay_doc.scenes.get(&shot.scene_id)?;
        new_table.push(
            ShotListEntry {
                completed: false,
                shot_number: shot.shot_number.clone(),
                shot_type: shot.shot_type.clone(),
                shot_subtype: shot.subtype.clone()?,
                shot_setup: shot.setup.clone(),
                scene_number: scene.number.clone()?,
                scene_environment: scene.environment.clone(),
                scene_time: scene.story_time_of_day.clone()?,
                scene_location: scene.story_location.clone(),
                scene_sublocation: scene.story_sublocation.clone(),
                group: "".into(),
                characters: "".into(), // TODO: add get_characters_for_shot function (will probably piggy back off functions like _get_pages_for_shot or get_lines_for_shot...)
                // the ScreenplayDocument should be responsible for basic gets like Scene and Page numbers/IDs,
                // or all scenes with a given location, all characters within a scene, all scenes with a character
                // BUT only SHOTLINER-CORE should be responsible for other advanced gets like all characters in a shotline,
                // or all tags or props, etc.
                // SL-CORE is responsible for assigning and finding PRODUCTION elements,
                // but ScreenplayDocument is responsible for SCREENPLAY elements, things inherent like speaking CHARACTERS and LOCATIONS and ALHPA-NUMBERED elements (pages and scenes) 
                tags: "".into(),
                props: "".into(),
                estimated_setup_time: "".into()
            }
        );
    }
    Some(new_table)
    
}

pub fn add_tagged_element(document: &mut document::ShotlinerDoc, id: Uuid, new_tagged_element: TaggedElement) -> Result<(), Error> {
    if let None = document.annotation_map.tagged_elements.insert(id, new_tagged_element) {
        return Ok(());
    }
    Err(Error)
}
pub fn modify_tagged_element(document: &mut document::ShotlinerDoc, id: Uuid, new_tagged_element: TaggedElement) -> Result<(), Error> {
    if let Some(_) = document.annotation_map.tagged_elements.insert(id, new_tagged_element) {
        return Ok(());
    }
    Err(Error)
}
pub fn remove_tagged_element(document: &mut document::ShotlinerDoc, id: Uuid) -> Result<(), Error> {
    if let Some(_) = document.annotation_map.tagged_elements.remove(&id) {
        return Ok(());
    }
    Err(Error)
}

pub fn add_tag(document: &mut document::ShotlinerDoc, tag: document::Tag, id: Uuid) -> Result<(), Error> {
    if let None = document.annotation_map.tags.insert(id, tag) {
        return Ok(());
    }
    Err(Error)
}
pub fn modify_tag(document: &mut document::ShotlinerDoc, new_tag: document::Tag, id: Uuid) -> Result<(), Error> {
    if document.annotation_map.tags.contains_key(&id) {
        document.annotation_map.tags.insert(id, new_tag);
        return Ok(());
    }
    Err(Error)
}
pub fn remove_tag(document: &mut document::ShotlinerDoc, id: Uuid) -> Result<(), Error> {
    if let Some(_) = document.annotation_map.tags.remove(&id) {
        return Ok(());
    }
    Err(Error)
}

pub fn add_shotline(document: &mut document::ShotlinerDoc, shotline: document::ShotLine, id: Uuid) -> Result<(), Error> {
    if let None = document.annotation_map.shotlines.insert(id, shotline) {
        return Ok(());
    }
    Err(Error) // Tried to add a shotline that already has the UUID in the map!
}
pub fn modify_shotline(document: &mut document::ShotlinerDoc, 
    id: Uuid, 
    new_shotline: document::ShotLine) -> Result<(), Error> {

    if let Some(_) = document.annotation_map.shotlines.insert(id, new_shotline){
        return  Ok(());
    }

    Err(Error) // Tried to modify a ShotLine that didn't exist!
}
pub fn remove_shotline(document: &mut document::ShotlinerDoc, id: Uuid) -> Result<(), Error>  {
    
    if let Some(_) = document.annotation_map.shotlines.remove(&id) {
        return Ok(());
    }

    Err(Error)
}


#[cfg(test)]
mod tests {
    
    #test
}
