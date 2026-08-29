use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PlayResponse {
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Metadata {
    pub title: String,
    pub artist: String,
    #[serde(rename = "duration")]
    pub duration_secs: f64,
    pub image: String,
}
