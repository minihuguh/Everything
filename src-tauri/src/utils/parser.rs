use quick_xml::de::from_str;
use serde::{Serialize, Deserialize};

//Header
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Playlist {
    #[serde(rename = "@version")]
    pub version: String,

    #[serde(rename = "@xmlns")]
    pub xmlns: Option<String>,

    pub title: Option<String>,
    pub location: Option<String>,
    pub identifier: Option<String>,
    pub date: Option<String>,
    pub annotation: Option<String>,

    #[serde(rename = "trackList")]
    pub track_list: TrackList,
}

//TrackLists
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TrackList {
    #[serde(default, rename = "track")]
    pub tracks: Vec<Track>,
}

//Track data
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Track {
    pub location: Option<String>,
    pub title: Option<String>,
    pub creator: Option<String>,
    pub album: Option<String>,
    pub duration: Option<u64>,

    #[serde(default, rename = "meta")]
    pub metas: Vec<Meta>,
}

//Track metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Meta {
    #[serde(rename = "@rel")]
    pub rel: String,

    #[serde(rename = "$text")]
    pub value: String,
}

//Implementations of functionality
impl Track {
    pub fn meta(&self, key: &str) -> Option<&str> {
        self.metas.iter()
            .find(|m| m.rel == key)
            .map(|m| m.value.as_str())
    }

    pub fn bitrate(&self)    -> Option<&str> { self.meta("bitrate") }
    pub fn channels(&self)   -> Option<&str> { self.meta("channels") }
    pub fn format(&self)     -> Option<&str> { self.meta("format") }
    pub fn samplerate(&self) -> Option<&str> { self.meta("samplerate") }
    pub fn size(&self)       -> Option<&str> { self.meta("size") }

    pub fn duration_fmt(&self) -> String {
        match self.duration {
            Some(ms) => {
                let secs = ms / 1000;
                format!("{}:{:02}", secs / 60, secs % 60)
            }
            None => "--:--".to_string(),
        }
    }
}

//Parse command
pub fn parse(xml: &str) -> Result<Playlist, quick_xml::DeError> {
    from_str(xml)
}