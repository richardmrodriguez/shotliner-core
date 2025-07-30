use std::{collections::HashMap, fmt::Error};
use screenplay_doc_parser_rs::screenplay_document::{self, ScreenplayDocument};
use uuid::Uuid;

use crate::document::{TaggedElement};


pub mod production;

/// Structs for export types, like Shot Lists or Storyboard Templates
pub mod serializables;

/// Structs and Methods for storing types of media that may be linked or embedded into the ShotLiner document.
pub mod multimedia;

pub mod document {
    use std::collections::btree_map::Range;
    use std::ops::{Deref, DerefMut};
    use std::{collections::HashMap, fmt::Error, hash::Hash};
    use screenplay_doc_parser_rs::pdf_document::Line;
    use uuid::Uuid;

    use screenplay_doc_parser_rs::screenplay_document::{SPType, Scene, SceneHeadingElement, SceneID, ScreenplayCoordinate, ScreenplayDocument, TextElement};

    use crate::commands;
    use crate::production::{self, Shot};

    //TODO: this will be used later, when we implement merge-forward    
    #[derive(Clone)]
    pub struct SmartScreenplayCoordinate {

    }
    
    
    /// A Tag is a finite Screenplay Element or range of Elements, which correspond to one or more Departments.    
    #[derive(Clone, Debug)]    
    pub struct Tag {
        pub tag_str: String, // FIXME: TODO: We need to be able to search tags by ID as well as by production and/or screenplay element.
        // fuck.
        pub departments: Vec<production::ProductionDepartments>,
        //pub other_metadata: idk
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct TagID(Uuid);


    #[derive(Clone, Debug)]
    pub struct TaggedElement {
        origin: ScreenplayCoordinate,
        endpoint: ScreenplayCoordinate, //inclusive
        tags: Vec<TagID> // tags are found / stored lazily; find tags by referencing the Annotation Map; Don't duplicate tag structs, just IDs
        // NOTE: if a UUID doesn't exist when invoking a tag search, DELETE it from the TaggedElement Vec
    }

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct ShotLineID(Uuid);
    impl Deref for ShotLineID {
        type Target = Uuid;
        fn deref(&self) -> &Self::Target {
            &self.0
        }

    }
    impl DerefMut for ShotLineID {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl ShotLineID {
        pub fn new() -> Self {
            ShotLineID(Uuid::new_v4())
        }
    }

    #[derive(Clone, Debug)]
    pub struct ShotLine {
        pub start: ScreenplayCoordinate,
        pub end: ScreenplayCoordinate,
        pub unfilmed_lines: Vec<ScreenplayCoordinate>,
        pub shot: production::Shot,
    }
    impl ShotLine {
        pub fn new(start: ScreenplayCoordinate, end: ScreenplayCoordinate, shot: Shot) -> Self {
            ShotLine { 
                start: start, 
                end: end, 
                unfilmed_lines: Vec::new(), 
                shot: shot 
            }
        }
    }

    #[derive(Clone)]
    pub struct AnnotationMap {
        pub shotlines: HashMap<ShotLineID, ShotLine>,
        pub tags: HashMap<Uuid, Tag>,
        pub tagged_elements: HashMap<Uuid, TaggedElement>,

    }
    impl AnnotationMap {
        pub fn new() -> Self {
            AnnotationMap {
                shotlines: HashMap::new(),
                tags: HashMap::new(),
                tagged_elements: HashMap::new()
            }
        }
    }
    
    //#[derive(Clone)]
    pub struct ShotlinerDoc {
        pub screenplay: ScreenplayDocument,
        pub command_history: Option<crate::commands::CommandHistory>,
        pub annotation_map: AnnotationMap
    }
    impl ShotlinerDoc {
        pub fn new(screenplay: screenplay_doc_parser_rs::screenplay_document::ScreenplayDocument) -> Self {
            ShotlinerDoc { 
                screenplay: screenplay, 
                command_history: None, 
                annotation_map: AnnotationMap::new()
            }
        }

        fn command_exec(&mut self, cmd: &commands::Command) -> Result<(), Error> {
            use commands::Command::*;
            match cmd {
                AddShotline(id,sl) => {
                    return self.add_shotline( sl.clone(), ShotLineID(id.clone()));
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
                    let other_id = id.clone();
                    if let Ok(_) = self.remove_shotline((&other_id)) {
                        return Ok(());

                    }
                    return Err(Error);


                }
                _ => {
                    return Err(Error);
                }
            }
        }

        pub fn get_scenes_for_shotline(&self, shotline_id: &ShotLineID) -> Vec<&SceneID> {
            let shotline = self.annotation_map.shotlines.get(shotline_id);
            let mut scenes = Vec::new();
            self.screenplay.scenes
        }
        
        pub fn get_shotlines_as_table(&mut self, shotlines: &HashMap<Uuid, crate::document::ShotLine>, screenplay_doc: &ScreenplayDocument) -> Option<Vec<crate::serializables::ShotListEntry>> {
            use crate::serializables::ShotListEntry;

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

        pub fn add_tagged_element(&mut self, id: Uuid, new_tagged_element: TaggedElement) -> Result<(), Error> {
            if let None = self.annotation_map.tagged_elements.insert(id, new_tagged_element) {
                return Ok(());
            }
            Err(Error)
        }
        pub fn modify_tagged_element(&mut self, id: Uuid, new_tagged_element: TaggedElement) -> Result<(), Error> {
            if let Some(_) = self.annotation_map.tagged_elements.insert(id, new_tagged_element) {
                return Ok(());
            }
            Err(Error)
        }
        pub fn remove_tagged_element(&mut self, id: Uuid) -> Result<(), Error> {
            if let Some(_) = self.annotation_map.tagged_elements.remove(&id) {
                return Ok(());
            }
            Err(Error)
        }

        pub fn add_tag(&mut self, tag: Tag, id: Uuid) -> Result<(), Error> {
            if let None = self.annotation_map.tags.insert(id, tag) {
                return Ok(());
            }
            Err(Error)
        }
        pub fn modify_tag(&mut self, new_tag: Tag, id: Uuid) -> Result<(), Error> {
            if self.annotation_map.tags.contains_key(&id) {
                self.annotation_map.tags.insert(id, new_tag);
                return Ok(());
            }
            Err(Error)
        }
        pub fn remove_tag(&mut self, id: Uuid) -> Result<(), Error> {
            if let Some(_) = self.annotation_map.tags.remove(&id) {
                return Ok(());
            }
            Err(Error)
        }

        /// Adds a shotline to this ShotlinerDocument struct.
        /// ```
        /// use shotliner_corelib::document;
        /// use screenplay_doc_parser_rs::screenplay_document::ScreenplayDocument;
        /// let new_screenplay = ScreenplayDocument::default();
        /// let mut doc = document::ShotlinerDoc::new(new_screenplay);
        /// 
        /// doc.add_shotline(
        /// Shotline::
        /// );
        /// 
        /// ``` 
        /// This takes in an `id`, instead of "just generating its own UUID,"
        /// because this function is expected to be used in a higher-level command pattern.
        /// 
        /// So, it may be useful 
        pub fn add_shotline(&mut self, shotline: ShotLine, id: ShotLineID) -> Result<(), Error> {
            if let None = self.annotation_map.shotlines.insert(id, shotline) {
                return Ok(());
            }
            Err(Error) // Tried to add a shotline that already has the UUID in the map!
        }
        pub fn modify_shotline(&mut self, 
            id: &ShotLineID, 
            new_shotline: ShotLine) -> Result<(), Error> {
            if let Some(_) = self.annotation_map.shotlines.get(id) {
                if let Some(_) = self.annotation_map.shotlines.insert(id.clone(), new_shotline){
                    return  Ok(());
                }
            }

            Err(Error) // Tried to modify a ShotLine that didn't exist!
        }
        pub fn remove_shotline(&mut self, id: &ShotLineID) -> Result<(), Error>  {
            
            if let Some(_) = self.annotation_map.shotlines.remove(id) {
                return Ok(());
            }

            Err(Error)
        }

    }

    
}

pub mod commands {
    use std::{collections::VecDeque, fmt::Error};

    use uuid::Uuid;

    use crate::document::{ShotLine, ShotLineID, Tag, TagID};
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
        AddShotline(ShotLineID, ShotLine),
        ModifyShotline(ShotLineID, Option<ShotLine>), // takes ID, old shotline
        RemoveShotline(ShotLineID, Option<ShotLine>), // takes ID, old shotline

        AddTag(TagID, Tag),
        ModifyTag(TagID, Option<Tag>),
        RemoveTag(TagID, Option<Tag>),
    }

}


#[cfg(test)]
mod tests {
    use screenplay_doc_parser_rs::{pdf_document::ElementIndentationsInches, screenplay_document::{SceneID, ScreenplayCoordinate}};
    use uuid::Uuid;

    use crate::{document::{AnnotationMap, ShotLineID, ShotlinerDoc}, production::{Shot, ShotID, ShotNumber, ShotSetup}};

    
    #[test]
    fn load_document_from_filesystem() {
        use screenplay_doc_parser_rs;
        
        let doc_result = screenplay_doc_parser_rs::mupdf_basic_parser::get_screenplay_doc_from_filepath(
            "test_data/VCR.pdf".to_string(), 
            Some(ElementIndentationsInches::us_letter_default().dialogue(3.5)), 
            None, 
            None, 
            None
        );
        if let Ok(doc) = doc_result {
            let mut new_shotliner_doc = ShotlinerDoc {
                screenplay: doc,
                command_history: None,
                annotation_map: AnnotationMap::new()
            };

            let shotline = crate::document::ShotLine {
                start: ScreenplayCoordinate {
                    page: 1,
                    line: 2,
                    element: None
                },
                end: ScreenplayCoordinate { 
                    page: 1, 
                    line: 4, 
                    element: None 
                },
                unfilmed_lines: Vec::new(),
                
                shot: Shot::new(None),
            };
           new_shotliner_doc.add_shotline(shotline, ShotLineID::new());
            for (id, sl) in new_shotliner_doc.annotation_map.shotlines {
                println!("ID: {:} | {:#?}", *id, sl);
            }
            println!("yay");
        }
        else {
            println!("oh no");
            println!("{:#?}", doc_result);
        }
    }
}
