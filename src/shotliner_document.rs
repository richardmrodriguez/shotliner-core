use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::{collections::HashMap, fmt::Error, hash::Hash};
use uuid::Uuid;

use screenplay_doc_parser_rs::screenplay_document::{self, ScreenplayDocument};

use crate::production::{self, ShotComposition};
use crate::{commands, shotliner_document};

//TODO: this will be used later, when we implement merge-forward for new drafts of the screenplay
#[derive(Clone)]
pub struct SmartScreenplayCoordinate {}

/// A Tag is a finite Screenplay Element or range of Elements, which correspond to one or more Departments.    
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Tag {
    pub string: String,
    // FIXME: TODO: We need to be able to search tags by ID as well as by production and/or screenplay element.
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
    pub ocurrances: HashSet<(
        screenplay_document::ScreenplayCoordinate,
        screenplay_document::ScreenplayCoordinate,
    )>, //list of RANGES that correspond to this thing...
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

#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    string: String,
    tags: HashSet<Tag>

}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct GroupID(Uuid);
impl GroupID {
    pub fn new() -> Self{
        GroupID(Uuid::new_v4())
    }
}
impl Deref for GroupID {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct ShotLine {
    pub start: screenplay_document::ScreenplayCoordinate,
    pub end: screenplay_document::ScreenplayCoordinate,
    pub unfilmed_lines: Option<HashSet<screenplay_document::ScreenplayDocument>>,
}
impl ShotLine {
    pub fn new(
        start: screenplay_document::ScreenplayCoordinate,
        end: screenplay_document::ScreenplayCoordinate,
    ) -> ShotLine {
        ShotLine {
            start: start,
            end: end,
            unfilmed_lines: None,
        }
    }
}

#[derive(Clone)]
pub struct AnnotationMap {
    pub shotlines: HashMap<production::ShotID, production::Shot>,
    pub tags: HashMap<TagID, Tag>,
    pub groups: HashMap<GroupID, Group>,
    pub tagged_elements: HashMap<TaggedElementID, TaggedElement>,
    pub shot_setups: HashMap<Uuid, production::ShotSetup>,
}
impl AnnotationMap {
    pub fn new() -> Self {
        AnnotationMap {
            shotlines: HashMap::new(),
            tags: HashMap::new(),
            groups: HashMap::new(),
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
    pub fn add_shotline(
        &mut self,
        shotline: production::Shot,
        id: production::ShotID,
    ) -> Result<(), Error> {
        if let None = self.annotation_map.shotlines.insert(id, shotline) {
            return Ok(());
        }
        Err(Error) // Tried to add a shotline that already has the UUID in the map!
    }
    pub fn modify_shotline(
        &mut self,
        id: &production::ShotID,
        new_shotline: production::Shot,
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
    pub fn remove_shotline(&mut self, id: &production::ShotID) -> Result<(), Error> {
        if let Some(_) = self.annotation_map.shotlines.remove(id) {
            return Ok(());
        }

        Err(Error)
    }
}
