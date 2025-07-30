#[derive(Clone, Debug)]
pub enum MediaType {
    Image,
    // could add other media types down the road...
}

#[derive(Clone, Debug)]
pub struct MediaLink {
    filepath: String,
    media_type: MediaType,
}
