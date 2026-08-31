use serde::Serialize;

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct TrackMetadata {
    pub title: String,
    pub artist: String,
    pub duration: f64,
    pub path: String,
    pub image: String,
}