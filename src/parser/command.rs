use quick_xml::de::from_str;
use crate::parser::parser::Playlist;

pub fn parse(xml: &str) -> Result<Playlist, quick_xml::DeError> {
    from_str(xml)
}


