use super::symphonia_source::SymphoniaSource;
use rodio::DeviceSinkBuilder;
use std::sync::{Arc, Mutex};
use serde_json::{json, Value};

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

    pub fn play_file(&self, path: &str) -> Result<Value, String> {
        let source = SymphoniaSource::new(path).map_err(|e| format!("Error al cargar: {}", e))?;
        let metadata = source.metadata().clone();
        let player = rodio::Player::connect_new(&self.mixer);

        let vol = *self.volume.lock().unwrap();
        player.set_volume(vol);
        player.append(source);

        *self.current_player.lock().unwrap() = Some(player);

        Ok(json!({
            "success": true,
            "metadata": {
                "title": metadata.title,
                "artist": metadata.artist,
                "duration": metadata.duration.map(|d| d.as_secs_f64()),
            }
        }))
    }

    pub fn get_time(&self) -> u64 {
        let x = self.current_player.lock().unwrap().as_mut().unwrap().get_pos();
        x.as_secs()
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
