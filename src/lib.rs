use std::fmt::Error;

use screenplay_doc_parser_rs::screenplay_document;

pub mod shots {

}

pub mod document {
    use std::collections::HashMap;

    use screenplay_doc_parser_rs::screenplay_document::ScreenplayDocument;

    pub struct Scene {
        pub start: ScreenplayCoordinate,
        pub end: ScreenplayCoordinate, 
        pub story_location: String,
        pub story_sublocation: Option<String>,
        pub story_time_of_day: String, // DAY, NIGHT, etc.
        pub real_locations: Vec<String>,
        pub real_sublocations: Option<Vec<String>>,
        pub real_time_of_day: String,
    }

    pub struct ScreenplayCoordinate {
        pub page: u64,
        pub line: u64,
    }

    //TODO: this will be used later, when we implement merge-forward
    pub struct SmartScreenplayCoordinate {

    }

    pub enum ShotType {
        XWS,
        WS,
        MS,
        CU,
        XCU,
    }

    //TODO: Make these better, maybe split up more categories
    pub enum ShotSubType {
        //TwoShot, should just be a separate "number_of_subjects" in a higher struct
        Trucking,
        Moving,
        Dolly,
        WhipPan,
        Panning,
        Other,

    }

    pub struct CameraMetadata {
        lens_mm: u64,
        //Camera body, make, model, resolution, codec, etc.

    }

    // A Tag is a finite Screenplay Element or range of Elements, which correspond to one or more Departments.
    pub struct Tag {
        pub tag_str: String, // FIXME: TODO: We need to be able to search tags by ID as well as by production and/or screenplay element.
        // fuck.
        pub departments: Vec<ProductionDepartments>,
        pub elements: Vec<ScreenplayCoordinate>, // get each element by using the coordinates
        //probably could create some caching scheme but idgaf right now
    }


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

    // Camera Setup, represents a specific, discrete position to place the camera
    //  
    pub struct ShotSetup {
        index: u64, // simple numerical counter
        id: String, // uuid?
    } 

    pub struct Shot {
        // Shot Composition (angle, staging, movement, etc.)
        pub shot_type: ShotType,
        pub subtype: Option<ShotSubType>,
        pub setup: ShotSetup, 

        // Technical Metadata
        pub camera_metadata: Option<CameraMetadata>,

        pub tags: Vec<String>,
    }

    pub struct ShotLine {
        pub id: String, // TODO: use UUIDs probably...
        pub start: ScreenplayCoordinate,
        pub end: ScreenplayCoordinate,
        pub unfilmed_lines: Vec<ScreenplayCoordinate>,
        pub shot: Shot
    }

    pub struct AnnotationMap {
        pub shotlines: Vec<ShotLine>,
        pub tags: Vec<Tag>,

    }

    pub struct ShotlinerDoc {
        pub screenplay: ScreenplayDocument,
        pub annotation_map: AnnotationMap
    }
    
}

pub mod commands {
    use std::fmt::Error;
    
    pub struct CommandExecuteUndo<T: Fn()> {
        execute: T,
        undo: T,

    }

    pub trait Command {
        fn execute(&mut self, stack: &mut CommandStack) -> Result<(), Error>;
        fn undo(&mut self, stack: &mut CommandStack) -> Result<(), Error>;
    }
    pub struct CommandStack {
        stack: Vec<_>
    }
    impl CommandStack {
        fn undo() -> bool {
            false
        }
        fn redo() -> bool {
            false
        }
    }

    pub struct CommandIndex {
        index: u64
    }

    pub struct AddShotlineCommand {
        last_command_index: CommandIndex,
        shotline: crate::document::ShotLine,
    }
    


}


pub fn add_tagged_element(document: &mut document::ShotlinerDoc, tag: document::Tag, elements: Vec<document::ScreenplayCoordinate>) -> Result<(), Error> {
    Ok(())
}
pub fn add_tag(document: &mut document::ShotlinerDoc, tag: document::Tag) -> Result<(), Error> {
    Ok(())
}

pub fn get_shotline_from_id(document: &mut document::ShotlinerDoc, 
    id: String
) -> Result<&mut document::ShotLine, Error> {
    for shotline in &mut document.annotation_map.shotlines {
        if shotline.id == id {
            return Ok(shotline);
        }
    }

    Err(Error)
}

pub fn add_shotline(document: &mut document::ShotlinerDoc, shotline: document::ShotLine) -> Result<(), Error> {
    document.annotation_map.shotlines.push(shotline);
    Ok(())
}

pub fn modify_shotline(document: &mut document::ShotlinerDoc, 
    id: String, 
    new_shotline: document::ShotLine) -> Result<(), Error> {
    // find shotline from ID / coordinate(s???)

    if let Ok(shotline_ref) = get_shotline_from_id(document, id) {
        *shotline_ref = new_shotline;
        return Ok(());
    }

    Err(Error)
    // replace with new shotline (which is probably modified / cloned from original shotline)
}

pub fn remove_shotline(document: &mut document::ShotlinerDoc, id: String) -> Result<(), Error>  {
    let mut counter: usize = 0;
    let mut index_to_remove: Option<usize> = None;
    for shotline in &mut document.annotation_map.shotlines {
        if shotline.id == id {
            index_to_remove = Some(counter.clone());
            break;
        }
        counter = counter + 1;
    }

    if let Some(idx) = index_to_remove{
        document.annotation_map.shotlines.remove(idx);
        document.annotation_map.shotlines.shrink_to_fit();
        return Ok(());
    }


    Err(Error)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
