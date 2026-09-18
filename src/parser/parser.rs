use serde::{Serialize, Deserialize};

//Header
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Playlist {
    #[serde(rename = "@version", default = "default_version")]
    pub version: String,

    #[serde(rename = "@xmlns", skip_serializing_if = "Option::is_none")]
    pub xmlns: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation: Option<String>,

    #[serde(rename = "trackList")]
    pub track_list: TrackList,
}

fn default_version() -> String { "1".to_string() }

//TrackLists
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TrackList {
    #[serde(default, rename = "track")]
    pub tracks: Vec<Track>,
}

//Track data
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Track {
    #[serde(skip_serializing_if = "Option::is_none")]

    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]

    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]

    pub creator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]

    pub album: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]

    pub duration: Option<u64>,

    #[serde(default, rename = "meta", skip_serializing_if = "Vec::is_empty")]
    pub metas: Vec<Meta>,
}

//Track metadata
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Meta {
    #[serde(rename = "@rel")]
    pub rel: String,

    #[serde(rename = "$text")]
    pub value: String,
}

//Implementations of functionality
impl Track {
    pub fn new(location: &str, title: &str) -> Self {
        Self {
            location: Some(location.to_string()),
            title: Some(title.to_string()),
            ..Default::default()
        }
    }

    pub fn with_creator(mut self, creator: &str) -> Self {
        self.creator = Some(creator.to_string());
        self
    }

    pub fn with_album(mut self, album: &str) -> Self {
        self.album = Some(album.to_string());
        self
    }

    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration = Some(ms);
        self
    }

    pub fn add_meta(&mut self, rel: &str, value: &str) {
        // Elimina meta existente con misma clave
        self.metas.retain(|m| m.rel != rel);
        self.metas.push(Meta {
            rel: rel.to_string(),
            value: value.to_string(),
        });
    }

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

//Helpers
impl Playlist {
    pub fn new(title: &str) -> Self {
        Self {
            version: "1".to_string(),
            xmlns: Some("http://xspf.org/ns/0/".to_string()),
            title: Some(title.to_string()),
            date: Some(chrono::Local::now().to_rfc3339()),
            track_list: TrackList::default(),
            ..Default::default()
        }
    }

    pub fn add_track(&mut self, track: Track) {
        self.track_list.tracks.push(track);
    }

    pub fn remove_track(&mut self, index: usize) -> Option<Track> {
        if index < self.track_list.tracks.len() {
            Some(self.track_list.tracks.remove(index))
        } else {
            None
        }
    }

    pub fn move_track(&mut self, from: usize, to: usize) {
        if from < self.track_list.tracks.len() && to < self.track_list.tracks.len() {
            let track = self.track_list.tracks.remove(from);
            self.track_list.tracks.insert(to, track);
        }
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = Some(title.to_string());
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let xml = quick_xml::se::to_string(&self)?;
        std::fs::write(path, xml)?;
        Ok(())
    }
}