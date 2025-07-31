use std::{fmt::Error, ops::{Deref, DerefMut}};

use uuid::Uuid;

use crate::{document::{Tag, TagID}, multimedia::MediaLink};


#[derive(Clone, Debug)]
pub enum Department {
    Production,
    Art,
    Wardrobe,
    HairMakeup,
    Camera,
    Sound,
    Electric,
    LightingGrip,
    Props,
    PracticalFX,
    VisualFX,
    Stunts,
    Animals,
    Vehicles,
    Dance,
    Choreography,
    Pyrotechnics,
    Armory,
    Intimacy,
    Craft,
    Transportation,
    Miscellaneous,

    Other(String),

}

#[derive(Clone, Debug)]
pub enum ShotType {
    ExtremeWide,
    Wide,
    Medium,
    CloseUp,
    ExtremeCloseUp,
    Insert,
    Other,
}

//TODO: Make these better, maybe split up more categories    
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
pub struct ShotSetup {
    pub index: u64, // simple numerical counter
    pub id: String, // uuid?
} 

#[derive(Clone, Debug)]
pub struct ShotNumber(pub String);


#[derive(Clone, PartialEq, Debug)]
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
impl ShotID {
    pub fn new() -> Self {
        ShotID(Uuid::new_v4())
    }
}

#[derive(Clone, Debug)]
pub struct Composition {

    // Shot Composition (angle, staging, movement, etc.)
    pub shot_type: ShotType,
    pub subtype: Option<ShotSubType>,
    pub setup: Option<ShotSetup>, 
    
    // Technical Metadata
    pub camera_metadata: Option<CameraMetadata>,
    
    pub tags: Vec<TagID>,
    //pub media: Vec<crate::multimedia::MediaLink>
}
impl Composition {
    pub fn new() -> Self {
        Composition { 
            shot_type: ShotType::Wide,
            subtype: None, 
            setup: None, 
            camera_metadata: None, 
            tags: Vec::new(), 
            //media: Vec::new() 
        }
        
    }
    pub fn shot_type(&mut self, shot_type: ShotType) {
        self.shot_type = shot_type;
    }
    pub fn subtype(&mut self, subtype: Option<ShotSubType>) {
        self.subtype = subtype;
    }
    pub fn setup(&mut self, setup: Option<ShotSetup>) {
        self.setup = setup;
    }
    pub fn camera_metadata(&mut self, camera_metadata: Option<CameraMetadata>) {
        self.camera_metadata = camera_metadata;
    }
    //pub fn add_media(&mut self, media_link: MediaLink)
    pub fn add_tag(&mut self, tag: &TagID) -> Result<(), Error>{
        if self.tags.contains(tag) {
            return Err(Error);
        }
        self.tags.push(tag.clone());
        return Ok(());
    }
    pub fn remove_tag(&mut self, tag: &TagID) -> Result<(), Error> {
        if self.tags.contains(tag) {
            self.tags.retain(|id| id != tag);
            return Ok(());
        }
        return Err(Error);
    }

}

#[derive(Clone, Debug)]
pub struct CameraMetadata {
    lens_mm: u64,
    //Camera body, make, model, resolution, codec, etc.

}
