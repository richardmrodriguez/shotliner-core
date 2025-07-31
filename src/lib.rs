use screenplay_doc_parser_rs::screenplay_document::{self, ScreenplayDocument};
use std::{collections::HashMap, fmt::Error};
use uuid::Uuid;

use crate::document::TaggedElement;

/// Structs to represent Production specific things,
/// such as Shots, ShotSetups, and Departments.
///
/// These are things NOT represented explicitly by the screenplay alone;
///
/// These require human decision makers to choose which screenplay elements need to become
/// specific Production elements.
pub mod production;

/// Structs for exportable data, like Shot Lists or Storyboard Templates
pub mod serializables;

/// Structs and Methods for storing types of media that may be linked or embedded into the ShotLiner document.
pub mod multimedia;

/// This module is responsible for retrieving filtered and sorted reports of AnnotationMap elements.
pub mod reports;

/// The ShotLiner document itself.
/// 
/// This contains the ScreenplayDocument and AnnotationMap.
/// 
/// TODO: Move the AnnotationMap and its contained elements to a separate annotation module...
pub mod document {
    use std::collections::HashSet;
    use std::ops::{Deref, DerefMut};
    use std::{collections::HashMap, fmt::Error, hash::Hash};
    use uuid::Uuid;

    use screenplay_doc_parser_rs::screenplay_document::{self, ScreenplayDocument};

    use crate::commands;
    use crate::production::{self, Composition};

    //TODO: this will be used later, when we implement merge-forward
    #[derive(Clone)]
    pub struct SmartScreenplayCoordinate {}

    /// A Tag is a finite Screenplay Element or range of Elements, which correspond to one or more Departments.    
    #[derive(Clone, Debug)]
    pub struct Tag {
        pub string: String, // FIXME: TODO: We need to be able to search tags by ID as well as by production and/or screenplay element.
        // fuck.
        pub departments: Vec<production::Department>,
        //pub other_metadata: idk
    }

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct TagID(Uuid);
    impl Deref for TagID {
        type Target = Uuid;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl TagID {
        pub fn new() -> Self {
            TagID(Uuid::new_v4())
        }
    }

    #[derive(Clone, Debug)]
    pub struct TaggedElement {
        pub ocurrances: HashSet<(screenplay_document::ScreenplayCoordinate, screenplay_document::ScreenplayCoordinate)>, //list of RANGES that correspond to this thing...
        pub origin: screenplay_document::ScreenplayCoordinate,
        pub endpoint: screenplay_document::ScreenplayCoordinate, //inclusive
        pub tags: Vec<TagID>, // tags are found / stored lazily; find tags by referencing the Annotation Map; Don't duplicate tag structs, just IDs
                              // NOTE: if a UUID doesn't exist when invoking a tag search, DELETE it from the TaggedElement Vec
    }

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct TaggedElementID(Uuid);
    impl Deref for TaggedElementID {
        type Target = Uuid;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl TaggedElementID {
        pub fn new() -> Self {
            TaggedElementID(Uuid::new_v4())
        }
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
        pub start: screenplay_document::ScreenplayCoordinate,
        pub end: screenplay_document::ScreenplayCoordinate,
        pub primary_composition: production::Composition,
        pub shot_number: Option<production::ShotNumber>,
        pub unfilmed_lines: Option<Vec<screenplay_document::ScreenplayCoordinate>>,
        pub sub_compositions: Option<HashMap<screenplay_document::ScreenplayCoordinate, production::Composition>>
    }
    impl ShotLine {
        pub fn new(
            start: screenplay_document::ScreenplayCoordinate,
            end: screenplay_document::ScreenplayCoordinate,
        ) -> Self {
            ShotLine {

                start: start,
                end: end,
                primary_composition: Composition::new(), // default is WIDE
                shot_number: None,
                unfilmed_lines: None,
                sub_compositions: None,
            }
        }
    }

    #[derive(Clone)]
    pub struct AnnotationMap {
        pub shotlines: HashMap<ShotLineID, ShotLine>,
        pub tags: HashMap<TagID, Tag>,
        pub tagged_elements: HashMap<TaggedElementID, TaggedElement>,
        pub shot_setups: HashMap<Uuid, production::ShotSetup>
    }
    impl AnnotationMap {
        pub fn new() -> Self {
            AnnotationMap {
                shotlines: HashMap::new(),
                tags: HashMap::new(),
                tagged_elements: HashMap::new(),
                shot_setups: HashMap::new(),
            }
        }
    }

    //#[derive(Clone)]
    pub struct ShotlinerDoc {
        pub screenplay: ScreenplayDocument,
        pub command_history: Option<crate::commands::CommandHistory>,
        pub annotation_map: AnnotationMap,
    }
    impl ShotlinerDoc {
        pub fn new(
            screenplay: screenplay_doc_parser_rs::screenplay_document::ScreenplayDocument,
        ) -> Self {
            ShotlinerDoc {
                screenplay: screenplay,
                command_history: None,
                annotation_map: AnnotationMap::new(),
            }
        }

        fn command_exec(&mut self, cmd: &commands::Command) -> Result<(), Error> {
            use commands::Command::*;
            match cmd {
                AddShotline(id, sl) => {
                    return self.add_shotline(sl.clone(), id.clone());
                }
                ModifyShotline(id, sl_opt) => {
                    return Ok(());
                }
                _ => {
                    return Err(Error);
                }
            }

            Err(Error)
        }

        fn command_undo(&mut self, cmd: &commands::Command) -> Result<(), Error> {
            match cmd {
                commands::Command::AddShotline(id, sl) => {
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

        pub fn add_tagged_element(
            &mut self,
            id: TaggedElementID,
            new_tagged_element: TaggedElement,
        ) -> Result<(), Error> {
            if let None = self
                .annotation_map
                .tagged_elements
                .insert(id, new_tagged_element)
            {
                return Ok(());
            }
            Err(Error)
        }
        pub fn modify_tagged_element(
            &mut self,
            id: TaggedElementID,
            new_tagged_element: TaggedElement,
        ) -> Result<(), Error> {
            if let Some(_) = self
                .annotation_map
                .tagged_elements
                .insert(id, new_tagged_element)
            {
                return Ok(());
            }
            Err(Error)
        }
        pub fn remove_tagged_element(&mut self, id: TaggedElementID) -> Result<(), Error> {
            if let Some(_) = self.annotation_map.tagged_elements.remove(&id) {
                return Ok(());
            }
            Err(Error)
        }

        pub fn add_tag(&mut self, tag: Tag, id: TagID) -> Result<(), Error> {
            if let None = self.annotation_map.tags.insert(id, tag) {
                return Ok(());
            }
            Err(Error)
        }
        pub fn modify_tag(&mut self, new_tag: Tag, id: TagID) -> Result<(), Error> {
            if self.annotation_map.tags.contains_key(&id) {
                self.annotation_map.tags.insert(id, new_tag);
                return Ok(());
            }
            Err(Error)
        }
        pub fn remove_tag(&mut self, id: TagID) -> Result<(), Error> {
            if let Some(_) = self.annotation_map.tags.remove(&id) {
                return Ok(());
            }
            Err(Error)
        }

        // TODO: Actually write this test...
        /// Adds a shotline to this ShotlinerDocument struct.
        /// ```
        /// use shotliner_corelib::document;
        /// use screenplay_doc_parser_rs::screenplay_document::ScreenplayDocument;
        /// let new_screenplay = ScreenplayDocument::default();
        /// let mut doc = document::ShotlinerDoc::new(new_screenplay);
        ///
        /// let new_shotline =
        ///
        /// doc.add_shotline(
        ///
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
        pub fn modify_shotline(
            &mut self,
            id: &ShotLineID,
            new_shotline: ShotLine,
        ) -> Result<(), Error> {
            if let Some(_) = self.annotation_map.shotlines.get(id) {
                if let Some(_) = self
                    .annotation_map
                    .shotlines
                    .insert(id.clone(), new_shotline)
                {
                    return Ok(());
                }
            }

            Err(Error) // Tried to modify a ShotLine that didn't exist!
        }
        pub fn remove_shotline(&mut self, id: &ShotLineID) -> Result<(), Error> {
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
            } else {
                return Err(Error);
            }
        }
        fn undo(&mut self) {
            if self.index > 0 {
                self.index = self.index - 1;
            } else {
                // TODO:
            }
        }
        fn redo(&mut self) {
            if self.index + 1 <= self.current_history_size {
                self.index = self.index + 1;
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
    use core::panic;
    use std::collections::{HashMap, HashSet};

    use screenplay_doc_parser_rs::{
        pdf_document::ElementIndentationsInches,
        screenplay_document::{self, SceneID, ScreenplayCoordinate},
    };
    use uuid::Uuid;

    use crate::{
        document::{
            AnnotationMap, ShotLineID, ShotlinerDoc, Tag, TagID, TaggedElement, TaggedElementID,
        },
        production::{Composition, ShotID, ShotNumber, ShotSetup},
    };

    // TODO: TEST MODIFY AND REMOVE
    // TODO: need a ScreenplayRange struct that holds two ScreenplayCoordinates
    #[test]
    fn test_add_stuff_to_annotation_map() {
        use screenplay_doc_parser_rs;

        let doc_result =
            screenplay_doc_parser_rs::mupdf_basic_parser::get_screenplay_doc_from_filepath(
                "test_data/VCR2L.pdf".to_string(),
                Some(ElementIndentationsInches::us_letter_default().dialogue(3.5)),
                None,
                None,
                None,
            );
        let Ok(doc) = doc_result else {
            panic!("Doc failed to load. {:#?}", doc_result);
        };
        let mut new_shotliner_doc = ShotlinerDoc {
            screenplay: doc,
            command_history: None,
            annotation_map: AnnotationMap::new(),
        };
        let new_tag_id = TagID::new();
        let new_tag = Tag {
            string: "Prop1".to_string(),
            departments: vec![crate::production::Department::Props],
        };
        let Ok(_) = new_shotliner_doc.add_tag(new_tag, new_tag_id.clone()) else {
            panic!("Failed to add tag.")
        };
        let mut new_composition = Composition::new();
        let Ok(_) = new_composition.add_tag(&new_tag_id.clone()) else {
            panic!("Failed to add tag to shot.")
        };
        let shotline = crate::document::ShotLine {
            start: ScreenplayCoordinate {
                page: 1,
                line: 2,
                element: None,
            },
            end: ScreenplayCoordinate {
                page: 1,
                line: 4,
                element: None,
            },
            shot_number: Some(ShotNumber("1A".to_string())),
            unfilmed_lines: None,

            primary_composition: new_composition,
            sub_compositions: None
        };
        let Ok(_) = new_shotliner_doc.add_shotline(shotline, ShotLineID::new()) else {
            panic!("Failed to add shotline.")
        };

        let new_tagged_region = (
            screenplay_document::ScreenplayCoordinate {
                page: 1,
                line: 1,
                element: Some(7),
            },
            screenplay_document::ScreenplayCoordinate {
                page: 1,
                line: 1,
                element: Some(8),
            },
        );

        let new_tagged_element_id = TaggedElementID::new();
        let new_tagged_element = TaggedElement {
            ocurrances: HashSet::new(),
            origin: new_tagged_region.0,
            endpoint: new_tagged_region.1,
            tags: vec![new_tag_id.clone()],
        };
        let Ok(_) =
            new_shotliner_doc.add_tagged_element(new_tagged_element_id.clone(), new_tagged_element)
        else {
            panic!("Failed to add new tagged element.");
        };

        println!("\nADDED TAGS:");
        for (tag_id, tag) in &new_shotliner_doc.annotation_map.tags {
            println!(
                "ID: {:?} | DEPs: {:<48?}| TAG_STRING: {:?}",
                tag_id, tag.departments, tag.string
            )
        }

        println!("\nADDED SHOTLINES:");
        for (id, sl) in &new_shotliner_doc.annotation_map.shotlines {
            println!(
                "ID: {:?} | SL_START: {:>3?}, SL_END: {:>3?}, UNFILMED_LINES: {:?} | \nPRIMARY COMP:\n {:#?},",
                id,
                (sl.start.page, sl.start.line),
                (sl.end.page, sl.end.line),
                sl.unfilmed_lines,
                sl.primary_composition,
            );
            for cmp in &sl.sub_compositions  {
                println!("SUBCOMP: {:#?}", cmp);
            }
        }

        println!("\nTAGGED ELEMENTS:");
        for (te_id, te) in &new_shotliner_doc.annotation_map.tagged_elements {
            println!("TE_ID: {:?} | TAGGED ELEMENT: {:#?}", te_id, te);

            println!("\n TEXT IN RANGE:");
            let Some(origin_page) = &new_shotliner_doc.screenplay.pages.get(te.origin.page) else {
                panic!("Couldn't get page.")
            };
            let Some(end_page) = &new_shotliner_doc.screenplay.pages.get(te.endpoint.page) else {
                panic!("Couldn't get end_page.");
            };
            let mut curr_page = te.origin.page;

            let Some(origin_line) = origin_page.lines.get(te.origin.line) else {
                panic!();
            };
            let Some(end_line) = end_page.lines.get(te.endpoint.line) else {
                panic!();
            };

            println!(
                "ORIGIN ELEMENT: {:?} | ENDPOINT ELEMENT: {:?}",
                origin_line.text_elements.get(te.origin.element.unwrap() as usize).unwrap().text,
                end_line.text_elements.get(te.endpoint.element.unwrap() as usize).unwrap().text
            );
        }
    }
}
