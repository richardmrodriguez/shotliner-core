use screenplay_doc_parser_rs::screenplay_document::{self, ScreenplayDocument};
use std::{collections::HashMap, fmt::Error};
use uuid::Uuid;

use crate::shotliner_document::TaggedElement;

/// Structs to represent Production specific things,
/// such as Shots, ShotSetups, and Departments.
///
/// These are things NOT represented explicitly by the screenplay alone;
///
/// These require human decision makers to choose which screenplay elements need to become
/// specific Production elements.
pub mod production;

/// Structs for exportable data, like Shot Lists, Strip Boards or Storyboard Templates
pub mod serializables;

/// Structs and Methods for storing types of media that may be linked or embedded into the ShotLiner document.
pub mod multimedia;

/// This module is responsible for retrieving filtered and sorted reports of AnnotationMap elements.
/// 
/// This is where you go to generate Shot Lists and Strip Boards, among other things...
pub mod reports;

/// The ShotLiner document itself.
/// 
/// This contains the ScreenplayDocument and AnnotationMap.
/// 
/// TODO: Move the AnnotationMap and its contained elements to a separate annotation module...
pub mod shotliner_document;

pub mod commands;

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
        production::{self, ShotComposition, ShotNumber}, shotliner_document::{
            AnnotationMap, ShotLine,  ShotlinerDoc, Tag, TagID, TaggedElement, TaggedElementID
        }
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
        let mut new_composition = ShotComposition::new();
        let Ok(_) = new_composition.add_tag(&new_tag_id.clone()) else {
            panic!("Failed to add tag to shot.")
        };
        let shotline = ShotLine {
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
            unfilmed_lines: None
        };

        let shot = production::Shot {
            
            shot_number: Some(ShotNumber("1A".to_string())),
            primary_composition: new_composition,
            sub_compositions: None,
            shotline: Some(shotline)
        };
        let Ok(_) = new_shotliner_doc.add_shotline(shot, production::ShotID::new()) else {
            panic!("Failed to add Shot.")
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

        println!("\nADDED SHOTS:");
        for (id, sl) in &new_shotliner_doc.annotation_map.shotlines {
            let Some(shotline) = &sl.shotline else {
                
                continue;
            };
            println!(
                "ID: {:?} | SL_START: {:>3?}, SL_END: {:>3?}, UNFILMED_LINES: {:?} | \nPRIMARY COMP:\n {:#?},",
                id,
                (shotline.start.page, shotline.end.line),
                (shotline.end.page, shotline.end.line),
                shotline.unfilmed_lines,
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
