use quick_xml::de::from_str;
pub mod parser;
pub use parser::{Playlist, Track, TrackList, Meta};

pub fn parse(xml: &str) -> Result<Playlist, String> {
    from_str(xml).map_err(|e| e.to_string())
}