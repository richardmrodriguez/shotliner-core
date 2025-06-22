use std::{collections::HashMap, fmt::Error};

use screenplay_doc_parser_rs::screenplay_document;
use uuid::Uuid;

use crate::document::{TaggedElement};


pub mod production {
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
    pub struct Shot {
        pub id: Uuid,

        pub scene_id: Uuid, // scenes get a UUID because they can have alphanumeric scene number...
        pub shot_number: String,

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

    use crate::{document, production};
    
    /// A singular shot entry, which takes up a single row of the worksheet.
    pub struct ShotListEntry {
        completed: bool,
        
        
        shot_number: String,
        shot_type: production::ShotType,
        shot_subtype: production::ShotSubType,
        shot_setup: String,

        scene_number: String,
        scene_environment: String,
        scene_time: String,
        scene_location: String,
        scene_sublocation: String,


        group: String,
        characters: String,
        tags: String,
        props: String,

        estimated_setup_time: String,


        
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

pub fn get_shotlines_as_table(shotlines: &HashMap<Uuid, document::ShotLine>) -> Option<Vec<serializables::ShotListEntry>> {
    use serializables::ShotListEntry;
    let mut new_table = Vec::<ShotListEntry>::new();
    for (_, shotline) in shotlines {
        let shot = shotline.shot;
        new_table.push(
            ShotListEntry {
                completed: false,
                shot_number: shot.shot_number,
                shot_type: shot.shot_type,
                shot_subtype: shot.subtype,
                shot_setup: shot.setup,
                scene_number: shot.scene_number,
                scene_environment
            }
        );
    }
    
    None
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

    use super::*;

    fn _create_pdfword(text: String, element_indentation: f64, y_height_inches: Option<f64>) -> screenplay_doc_parser_rs::pdf_document::Word {
        use screenplay_doc_parser_rs::pdf_document;
        use screenplay_doc_parser_rs::pdf_document::TextPosition;
        let mut y_height_pts = 0.0;
        if let Some(inches) = y_height_inches {
            y_height_pts = 72.0 * inches;
        }
        else {
            y_height_pts = 3.0 * 72.0;
        }
        
        let new_word: pdf_document::Word = pdf_document::Word {
            text: text.clone(), 
            text_bbox_width: text.len() as f64 * 7.2 as f64, 
            position: TextPosition {
                x: element_indentation,
                y: y_height_pts
            }, 
            font_name: None, 
            font_size: 12.0, 
            font_character_width: 7.2 
        };
        new_word
    }

    // TODO: Test output
    // Create a Screenplay Doc with some content
    // Print "before" state of content
    // do operations (add Shotlines)
    // Print "after" state of content (with newly added ShotLines)



    #[test]
    fn it_works() {
        use screenplay_document;
        let mut doc = screenplay_document::ScreenplayDocument::default();

    }
}
