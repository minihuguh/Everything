use super::symphonia_source::SymphoniaSource;
use crate::state::TrackMetadata;
use rodio::DeviceSinkBuilder;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use rodio::source::SeekError;
use serde_json::{json, Value};
use base64::{Engine as _, engine::general_purpose::STANDARD};

pub struct AudioPlayer {
    device_sink: rodio::MixerDeviceSink,
    mixer: rodio::mixer::Mixer,
    current_player: Arc<Mutex<Option<rodio::Player>>>,
    volume: Arc<Mutex<f32>>,
}

impl AudioPlayer {
    pub fn new() -> Result<Self, String> {
        let device_sink = DeviceSinkBuilder::open_default_sink().map_err(|e| e.to_string())?;
        let mixer = device_sink.mixer().clone();
        Ok(AudioPlayer {
            device_sink,
            mixer,
            current_player: Arc::new(Mutex::new(None)),
            volume: Arc::new(Mutex::new(0.5)),
        })
    }

    pub fn extract_metadata(&self, path: &str) -> Result<TrackMetadata, String> {
        let source = SymphoniaSource::new(path).map_err(|e| format!("Error al cargar: {}", e))?;
        let meta = source.metadata();
        let mut image = String::new();
        if let Some(bytes) = &meta.image {
            image = STANDARD.encode(bytes);
        }
        Ok(TrackMetadata {
            title: meta.title.clone().unwrap_or_default(),
            artist: meta.artist.clone().unwrap_or_default(),
            duration_secs: meta.duration.map(|d| d.as_secs_f64()).unwrap_or(0.0),
            path: path.to_string(),
            image,
        })
    }

    pub fn play_file(&self, path: &str) -> Result<Value, String> {
        let track = self.extract_metadata(path)?;
        self.play_track(&track)?;
        Ok(json!({
            "success": true,
            "metadata": {
                "title": track.title,
                "artist": track.artist,
                "duration": track.duration_secs,
                "image": track.image,
            }
        }))
    }

    pub fn play_track(&self, track: &TrackMetadata) -> Result<(), String> {
        let source = SymphoniaSource::new(&track.path).map_err(|e| format!("Error al cargar: {}", e))?;
        let player = rodio::Player::connect_new(&self.mixer);
        let vol = *self.volume.lock().unwrap();
        player.set_volume(vol);
        player.append(source);
        *self.current_player.lock().unwrap() = Some(player);
        Ok(())
    }

    pub fn get_time(&self) -> u64 {
        let guard = self.current_player.lock().unwrap();
        match guard.as_ref() {
            Some(player) => player.get_pos().as_secs(),
            None => 0,
        }
    }

    pub fn set_time(&self, time_pos: Duration) -> Result<(), SeekError> {
        let mut guard = self.current_player.lock().unwrap();
        match guard.as_mut() {
            Some(player) => player.try_seek(time_pos),
            None => Ok(()),
        }
    }

    pub fn pause(&self) {
        if let Some(ref player) = *self.current_player.lock().unwrap() {
            player.pause();
        }
    }

    pub fn resume(&self) {
        if let Some(ref player) = *self.current_player.lock().unwrap() {
            player.play();
        }
    }

    pub fn stop(&self) {
        if let Some(ref player) = *self.current_player.lock().unwrap() {
            player.stop();
        }
    }

    pub fn set_volume(&self, vol: f32) {
        *self.volume.lock().unwrap() = vol;
        if let Some(ref player) = *self.current_player.lock().unwrap() {
            player.set_volume(vol);
        }
    }

    pub fn is_playing(&self) -> bool {
        match *self.current_player.lock().unwrap() {
            Some(ref player) => !player.empty(),
            None => false,
        }
    }
}