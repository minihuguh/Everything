use dioxus::prelude::*;
use crate::ipc::log;
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TrackMetadata {
    pub title: String,
    pub artist: String,
    pub duration_secs: f64,
    pub path: String,
    pub image: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum RepeatMode {
    #[default]
    Off,
    One,
    All,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum QueueSource {
    #[default]
    Playlist,
    Queue,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HistoryEntry {
    pub source: QueueSource,
    pub playlist_index: Option<usize>,
    pub queue_index: Option<usize>,
    pub track: TrackMetadata,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlayerState {
    pub current_view: &'static str,
    pub is_playing: bool,
    pub current_time: f64,
    pub duration: f64,
    pub volume: f64,
    pub metadata: Option<TrackMetadata>,
    pub is_loading: bool,
    pub is_dragging_prog: bool,
    pub active_playlist: Vec<TrackMetadata>,
    pub active_playlist_name: String,
    pub user_queue: Vec<TrackMetadata>,
    pub history: Vec<HistoryEntry>,
    pub current_playlist_index: Option<usize>,
    pub current_queue_index: Option<usize>,
    pub current_source: QueueSource,
    pub is_shuffled: bool,
    pub repeat_mode: RepeatMode,
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
        Self::format_secs(self.current_time)
    }

    pub fn format_duration(&self) -> String {
        Self::format_secs(self.duration)
    }

    pub fn current_track(&self) -> Option<&TrackMetadata> {
        let from_index = match self.current_source {
            QueueSource::Playlist => {
                self.current_playlist_index.and_then(|i| self.active_playlist.get(i))
            }
            QueueSource::Queue => {
                self.current_queue_index.and_then(|i| self.user_queue.get(i))
            }
        };
        from_index.or(self.metadata.as_ref())
    }

    pub fn next_track_exists(&self) -> bool {
        match self.current_source {
            QueueSource::Playlist => {
                if !self.user_queue.is_empty() {
                    return true;
                }
                if let Some(idx) = self.current_playlist_index {
                    if idx + 1 < self.active_playlist.len() {
                        return true;
                    }
                }
                self.repeat_mode == RepeatMode::All && !self.active_playlist.is_empty()
            }
            QueueSource::Queue => {
                if let Some(idx) = self.current_queue_index {
                    if idx + 1 < self.user_queue.len() {
                        return true;
                    }
                }
                if let Some(idx) = self.current_playlist_index {
                    if idx + 1 < self.active_playlist.len() {
                        return true;
                    }
                }
                !self.active_playlist.is_empty() ||
                    (self.repeat_mode == RepeatMode::All && !self.user_queue.is_empty())
            }
        }
    }

    pub fn prev_track_exists(&self) -> bool {
        if self.current_time > 3.0 {
            return true;
        }

        match self.current_source {
            QueueSource::Queue => {
                if let Some(idx) = self.current_queue_index {
                    idx > 0
                } else {
                    false
                }
            }
            QueueSource::Playlist => {
                if let Some(idx) = self.current_playlist_index {
                    idx > 0
                } else {
                    false
                }
            }
        }
    }

    pub fn play_playlist(&mut self, tracks: Vec<TrackMetadata>, name: String, start_index: usize) {
        self.active_playlist = tracks;
        self.active_playlist_name = name;
        self.current_source = QueueSource::Playlist;
        self.current_playlist_index = Some(start_index.min(self.active_playlist.len().saturating_sub(1)));
        self.current_queue_index = None;
        self.history.clear();
        self.current_time = 0.0;
        self.is_playing = true;
        self.metadata = self.current_track().cloned();
        if let Some(meta) = &self.metadata {
            self.duration = meta.duration_secs;
        }
    }

    pub fn play_user_queue(&mut self, tracks: Vec<TrackMetadata>, start_index: usize) {
        self.user_queue = tracks;
        self.current_source = QueueSource::Queue;
        self.current_queue_index = Some(start_index.min(self.user_queue.len().saturating_sub(1)));
        self.current_playlist_index = None;
        self.history.clear();
        self.current_time = 0.0;
        self.is_playing = true;
        self.metadata = self.current_track().cloned();
        if let Some(meta) = &self.metadata {
            self.duration = meta.duration_secs;
        }
    }

    pub fn add_to_queue(&mut self, track: TrackMetadata) {
        self.user_queue.push(track);
        if self.current_track().is_none() && self.user_queue.len() == 1 {
            self.current_source = QueueSource::Queue;
            self.current_queue_index = Some(0);
            self.current_playlist_index = None;
            self.metadata = self.current_track().cloned();
            if let Some(meta) = &self.metadata {
                self.duration = meta.duration_secs;
            }
            self.is_playing = true;
        }
    }

    pub fn clear_user_queue(&mut self) {
        if self.current_source == QueueSource::Queue {
            if let Some(idx) = self.current_queue_index {
                if idx < self.user_queue.len() {
                    let current = self.user_queue[idx].clone();
                    self.user_queue.clear();
                    self.user_queue.push(current);
                    self.current_queue_index = Some(0);
                    self.history.retain(|e| e.source != QueueSource::Queue);
                    return;
                }
            }
        }
        self.user_queue.clear();
        self.current_queue_index = None;
        self.history.retain(|e| e.source != QueueSource::Queue);
    }

    pub fn remove_from_queue(&mut self, index: usize) {
        if index >= self.user_queue.len() {
            return;
        }
        if self.current_source == QueueSource::Queue {
            if let Some(current_idx) = self.current_queue_index {
                if index == current_idx {
                    return;
                }
                if index < current_idx {
                    self.current_queue_index = Some(current_idx - 1);
                }
            }
        }

        self.history.retain_mut(| entry |{
            if entry.source == QueueSource::Queue {
                if let Some(queue_index) = entry.queue_index {
                    if queue_index == index {
                        return false;
                    } else if queue_index < index {
                        if index < queue_index {
                            entry.queue_index = Some(queue_index - 1);
                        }
                    }
                }
            }
            true
        });

        self.user_queue.remove(index);
        if self.user_queue.is_empty() {
            self.current_queue_index = None;
        }
    }

    fn avoid_duplicate_code (&mut self) {
        if let Some(current) = self.current_track().cloned() {
            self.history.push(HistoryEntry {
                source: self.current_source,
                playlist_index: self.current_playlist_index,
                queue_index: self.current_queue_index,
                track: current,
            });
        }
    }

    pub fn advance(&mut self) {
        if self.repeat_mode == RepeatMode::One {
            self.current_time = 0.0;
            self.is_playing = true;
            return;
        }

        self.avoid_duplicate_code();

        match self.current_source {
            QueueSource::Playlist => {
                if !self.user_queue.is_empty() {
                    self.current_source = QueueSource::Queue;
                    self.current_queue_index = Some(0);
                    // self.current_playlist_index = None;
                } else if let Some(idx) = self.current_playlist_index {
                    if idx + 1 < self.active_playlist.len() {
                        self.current_playlist_index = Some(idx + 1);
                    } else if self.repeat_mode == RepeatMode::All && !self.active_playlist.is_empty() {
                        self.current_playlist_index = Some(0);
                        self.history.clear();
                    } else {
                        self.is_playing = false;
                        return;
                    }
                } else {
                    self.is_playing = false;
                    return;
                }
            }
            QueueSource::Queue => {
                if let Some(idx) = self.current_queue_index {
                    if idx + 1 < self.user_queue.len() {
                        self.current_queue_index = Some(idx + 1);
                    } else if !self.active_playlist.is_empty() {
                        let next_idx = self.current_playlist_index.map(|i| i + 1).unwrap_or(0);
                        if next_idx < self.active_playlist.len() {
                            self.current_source = QueueSource::Playlist;
                            self.current_playlist_index = Some(next_idx);
                            self.current_queue_index = None;
                        } else if self.repeat_mode == RepeatMode::All {
                            self.current_source = QueueSource::Playlist;
                            self.current_playlist_index = Some(0);
                            self.current_queue_index = None;
                            self.history.clear();
                        } else {
                            self.is_playing = false;
                            return;
                        }
                    } else if self.repeat_mode == RepeatMode::All && !self.user_queue.is_empty() {
                        self.current_queue_index = Some(0);
                        self.history.clear();
                    } else {
                        self.is_playing = false;
                        return;
                    }
                } else {
                    self.is_playing = false;
                    return;
                }
            }
        }

        self.current_time = 0.0;
        self.metadata = self.current_track().cloned();
        if let Some(meta) = &self.metadata {
            self.duration = meta.duration_secs;
        }
    }
    pub fn go_back(&mut self) -> bool {
        if self.current_time > 3.0 {
            self.current_time = 0.0;
            if !self.is_playing {
                self.is_playing = true;
            }
            return true;
        }

        if self.current_source == QueueSource::Queue {
            if let Some(idx) = self.current_queue_index {
                if idx > 0 {
                    self.current_queue_index = Some(idx - 1);
                    self.current_time = 0.0;
                    self.metadata = self.current_track().cloned();
                    if let Some(meta) = &self.metadata {
                        self.duration = meta.duration_secs;
                    }
                    self.is_playing = true;
                    return true;
                }
            }
            self.current_time = 0.0;
            return false;
        }

        if self.current_source == QueueSource::Playlist {
            if let Some(idx) = self.current_playlist_index {
                if idx > 0 {
                    self.current_playlist_index = Some(idx - 1);
                    self.current_time = 0.0;
                    self.metadata = self.current_track().cloned();
                    if let Some(meta) = &self.metadata {
                        self.duration = meta.duration_secs;
                    }
                    self.is_playing = true;
                    return true;
                }
            }
            self.current_time = 0.0;
            return false;
        }

        // if !self.is_playing {
        //     self.is_playing = true;
        // }

        self.current_time = 0.0;
        false
    }
    pub fn toggle_repeat(&mut self) {
        self.repeat_mode = match self.repeat_mode {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        };
    }

    pub fn toggle_shuffle(&mut self) {
        self.is_shuffled = !self.is_shuffled;
    }

    pub fn jump_to_playlist(&mut self, index: usize) {
        if index >= self.active_playlist.len() {
            return;
        }
        self.avoid_duplicate_code();
        self.current_source = QueueSource::Playlist;
        self.current_playlist_index = Some(index);
        self.current_queue_index = None;
        self.current_time = 0.0;
        self.metadata = self.current_track().cloned();
        if let Some(meta) = &self.metadata {
            self.duration = meta.duration_secs;
        }
        self.is_playing = true;
    }

    pub fn jump_to_queue(&mut self, index: usize) {
        if index >= self.user_queue.len() {
            return;
        }
        if self.current_source == QueueSource::Queue && self.current_queue_index == Some(index) {
            return;
        }

        self.avoid_duplicate_code();

        self.current_source = QueueSource::Queue;
        self.current_queue_index = Some(index);
        self.current_playlist_index = None;
        self.current_time = 0.0;
        self.metadata = self.current_track().cloned();
        if let Some(meta) = &self.metadata {
            self.duration = meta.duration_secs;
        }
        self.is_playing = true;
    }

    pub fn user_queue_len(&self) -> usize {
        self.user_queue.len()
    }

    pub fn playlist_len(&self) -> usize {
        self.active_playlist.len()
    }

    fn format_secs(secs: f64) -> String {
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
        volume: 10.0,
        metadata: None,
        is_loading: false,
        is_dragging_prog: false,
        active_playlist: Vec::new(),
        active_playlist_name: String::new(),
        user_queue: Vec::new(),
        history: Vec::new(),
        current_playlist_index: None,
        current_queue_index: None,
        current_source: QueueSource::Playlist,
        is_shuffled: false,
        repeat_mode: RepeatMode::Off,
    }));
}

pub fn use_current_view() -> Memo<&'static str> {
    let player = use_player_state();
    use_memo(move || player().current_view)
}

pub fn set_current_view(view: &'static str) {
    use_player_state().write().current_view = view;
}