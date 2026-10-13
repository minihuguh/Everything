use std::collections::HashMap;
use super::symphonia_source::SymphoniaSource;
use crate::state::TrackMetadata;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use rodio::source::SeekError;
use rodio::DeviceSinkBuilder;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use log::{debug, error, info};

pub struct AudioPlayer {
    device_sink: rodio::MixerDeviceSink,
    mixer: rodio::mixer::Mixer,
    current_player: Arc<Mutex<Option<rodio::Player>>>,
    volume: Arc<Mutex<f32>>,
    metadata_cache: Arc<Mutex<HashMap<String, TrackMetadata>>>,
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
            metadata_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn extract_metadata(&self, path: &str) -> Result<TrackMetadata, String> {
        {
            info!("Extracting metadata from {}", path);
            let cache = self.metadata_cache.lock().unwrap();
            if let Some(meta) = cache.get(path) {
                info!("Using cache");
                return Ok(meta.clone());
            }
        }

        let source = SymphoniaSource::new(path).map_err(|e| format!("Error al cargar: {}", e))?;
        let meta = source.metadata();
        let mut image = String::new();
        if let Some(bytes) = &meta.image {
            image = STANDARD.encode(bytes);
        }

        let track_meta = TrackMetadata {
            title: meta.title.clone().unwrap_or_default(),
            artist: meta.artist.clone().unwrap_or_default(),
            duration: meta.duration.map(|d| d.as_secs_f64()).unwrap_or(0.0),
            path: path.to_string(),
            image,
        };

        let mut cache = self.metadata_cache.lock().unwrap();
        cache.insert(path.to_string(), track_meta.clone());

        Ok(track_meta)
    }

    pub fn play_file(&self, path: &str) -> Result<Value, String> {
        let track = self.extract_metadata(path)?;
        self.play_track(&track)?;
        Ok(json!({
            "success": true,
            "metadata": {
                "title": track.title,
                "artist": track.artist,
                "duration": track.duration,
                "image": track.image,
            }
        }))
    }

    pub fn play_track(&self, track: &TrackMetadata) -> Result<(), String> {
        let source =
            SymphoniaSource::new(&track.path).map_err(|e| format!("Error al cargar: {}", e))?;
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