use dioxus::prelude::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TrackMetadata {
    pub title: String,
    pub artist: String,
    pub duration_secs: f64,
    pub path: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlayerState {
    pub current_view: &'static str,
    pub is_playing: bool,
    pub current_time: f64,      // segundos actuales
    pub duration: f64,          // duración total en segundos
    pub volume: f64,            // 0.0 - 1.0
    pub metadata: Option<TrackMetadata>,
    pub is_loading: bool,
    pub is_dragging_prog: bool
}

impl PlayerState {
    pub fn progress_pct(&self) -> f64 {
        if self.duration > 0.0 {
            (self.current_time / self.duration * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        }
    }

    pub fn format_time(&self) -> String {
        let temp = Self::format_secs(self.current_time);
        // web_sys::console::log_1(&format!("[CHECK FROM STATE] {}", temp).into());
        temp
    }

    pub fn format_duration(&self) -> String {
        Self::format_secs(self.duration)
    }

    fn format_secs(secs: f64) -> String {
        // web_sys::console::log_1(&format!("[CHECK FROM STATE (format_secs)] {}", secs).into());
        let total: u64 = secs.round() as u64;
        let mins: u64 = total / 60;
        let secs: u64 = total % 60;
        format!("{mins:02}:{secs:02}")
    }
}

pub fn use_player_state() -> Signal<PlayerState> {
    use_context::<Signal<PlayerState>>()
}

pub fn provide_player_state() {
    use_context_provider(|| Signal::new(PlayerState {
        current_view: "home",
        is_playing: false,
        current_time: 0.0,
        duration: 0.0,
        volume: 100.0,
        metadata: None,
        is_loading: false,
        is_dragging_prog: false,
    }));
}

pub fn use_current_view() -> Memo<&'static str> {
    let player = use_player_state();
    use_memo(move || player().current_view)
}

/// Cambia la vista actual
pub fn set_current_view(view: &'static str) {
    use_player_state().write().current_view = view;
}