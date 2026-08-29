#[derive(Clone, Debug, Default, PartialEq)]
pub struct TrackMetadata {
    pub title: String,
    pub artist: String,
    pub duration_secs: f64,
    pub path: String,
    pub image: String,
}