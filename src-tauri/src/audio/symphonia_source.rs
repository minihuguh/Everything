use std::error::Error;
use std::fs::File;
use std::num::{NonZero, NonZeroU16, NonZeroU32};
use std::path::Path;
use std::time::Duration;

use serde::Serialize;
use rodio::Source;
use symphonia::core::audio::sample::Sample;
use symphonia::core::codecs::audio::{AudioDecoder, AudioDecoderOptions};
use symphonia::core::codecs::registry::CodecRegistry;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, Tag};
use symphonia::core::meta::StandardTag;
use symphonia_adapter_libopus::OpusDecoder;

fn build_codec_registry() -> CodecRegistry {
    let mut registry = CodecRegistry::new();

    // 1. Registrar todos los códecs por defecto de Symphonia
    symphonia::default::register_enabled_codecs(&mut registry);

    // 2. Registrar el adaptador Libopus para decodificación de audio Opus
    registry.register_audio_decoder::<OpusDecoder>();
    registry
}

#[derive(Debug, Clone, Serialize)]
pub struct SimpleMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub duration: Option<Duration>,
}

pub struct SymphoniaSource {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn AudioDecoder>,
    track_id: u32,
    channels: u16,
    sample_rate: u32,
    total_duration: Option<Duration>,
    sample_buffer: Option<Vec<f32>>,
    buffer_pos: usize,
    metadata: SimpleMetadata,
}

impl SymphoniaSource {
    fn calculate_duration(track: &symphonia::core::formats::Track, sample_rate: u32) -> Option<Duration> {
        // ÚNICA fuente confiable en Symphonia 0.6
        track.num_frames
            .map(|n| Duration::from_secs_f64(n as f64 / sample_rate as f64))
    }
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let path = path.as_ref();

        let file_name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Desconocido")
            .to_string();

        let file = Box::new(File::open(path)?);

        let mss = MediaSourceStream::new(file, Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        // Probe::format() renombrado a Probe::probe() y devuelve Box<dyn FormatReader> directamente
        let mut format = symphonia::default::get_probe().probe(
            &hint,
            mss,
            FormatOptions::default(),
            MetadataOptions::default(),
        )?;

        let mut metadata = Self::extract_simple_metadata(&mut format);

        if metadata.title.is_none() {
            metadata.title = Some(file_name.clone());
        }
        if metadata.artist.is_none() {
            metadata.artist = Some("Desconocido".to_string());
        }

        // Opción A: obtener la pista de audio por defecto (recomendado en 0.6)
        let track = format.default_track(TrackType::Audio)
            .ok_or("No se encontró ninguna pista de audio soportada")?;

        // Opción B: filtrar manualmente como antes
        // let track = format
        //     .tracks()
        //     .iter()
        //     .find(|t| matches!(t.codec_params, Some(CodecParameters::Audio(_))))
        //     .ok_or("No se encontró ninguna pista de audio soportada")?;

        let track_id = track.id;


        // En 0.6, codec_params es Option<CodecParameters> (ahora un enum por tipo de medio)
        let audio_params = track
            .codec_params
            .as_ref()
            .and_then(|p| p.audio())
            .ok_or("No se encontraron parámetros de audio válidos")?;

        let sample_rate = audio_params
            .sample_rate
            .ok_or("No se pudo determinar sample rate")?;

        let channels = audio_params
            .channels
            .as_ref()
            .map(|c| c.count() as u16)
            .unwrap_or(2);

        // En 0.6, n_frames se movió de CodecParameters a Track directamente
        let mut total_duration = track.num_frames
            .map(|n| Duration::from_secs_f64(n as f64 / sample_rate as f64));

        if total_duration.is_none() {
            total_duration = Some(Duration::from_secs_f64(0.0));
        }

        let metadata = SimpleMetadata {
            duration: total_duration.or(metadata.duration),
            ..metadata
        };


        let registry = build_codec_registry();
        let decoder = registry.make_audio_decoder(
            audio_params,
            &AudioDecoderOptions::default(),
        )?;

        Ok(Self {
            format,
            decoder,
            track_id,
            channels,
            sample_rate,
            total_duration,
            sample_buffer: None,
            buffer_pos: 0,
            metadata,
        })
    }

    //noinspection D
    fn extract_simple_metadata(format: &mut Box<dyn FormatReader>) -> SimpleMetadata {
        let mut title = None;
        let mut artist = None;

        let metadata = format.metadata();

        // Solo procesar la primera revisión disponible
        if let Some(rev) = metadata.current() {
            // Media tags
            for tag in &rev.media.tags {
                if let Some(std_tag) = &tag.std {
                    match std_tag {
                        StandardTag::TrackTitle(v) => title = Some(v.to_string()),
                        StandardTag::Artist(v) => artist = Some(v.to_string()),
                        _ => {}
                    }
                }
            }

            // Per-track tags (solo si falta algo)
            if title.is_none() || artist.is_none() {
                for per_track in &rev.per_track {
                    for tag in &per_track.metadata.tags {
                        if let Some(std_tag) = &tag.std {
                            match std_tag {
                                StandardTag::TrackTitle(v) if title.is_none() => {
                                    title = Some(v.to_string());
                                }
                                StandardTag::Artist(v) if artist.is_none() => {
                                    artist = Some(v.to_string());
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        SimpleMetadata {
            title,
            artist,
            duration: None,
        }
    }


    #[inline]
    fn update_from_tag(tag: &Tag, title: &mut Option<String>, artist: &mut Option<String>) {
        let Some(std_tag) = &tag.std else { return };

        match std_tag {
            StandardTag::TrackTitle(v) if title.is_none() => {
                *title = Some(v.to_string());
            }
            StandardTag::Artist(v) if artist.is_none() => {
                *artist = Some(v.to_string());
            }
            _ => {}
        }
    }

    pub fn metadata(&self) -> &SimpleMetadata {
        &self.metadata
    }

    fn refuel_buffer(&mut self) -> bool {
        loop {
            // En 0.6, next_packet() devuelve Result<Option<Packet>>.
            // Ok(None) indica EOF de forma normal.
            let packet = match self.format.next_packet() {
                Ok(Some(packet)) => packet,
                Ok(None) => return false,
                Err(_) => return false,
            };

            // En 0.6, los campos de Packet son públicos; los getters fueron eliminados
            if packet.track_id != self.track_id {
                continue;
            }

            match self.decoder.decode(&packet) {
                Ok(audio_buf) => {
                    // SampleBuffer fue eliminado en 0.6.
                    // Se usa un Vec<f32> y los métodos del trait Audio / GenericAudioBufferRef.
                    let n_samples = audio_buf.samples_interleaved();

                    if let Some(ref mut buf) = self.sample_buffer {
                        buf.resize(n_samples, f32::MID);
                    } else {
                        self.sample_buffer = Some(vec![f32::MID; n_samples]);
                    }

                    if let Some(ref mut samples) = self.sample_buffer {
                        audio_buf.copy_to_slice_interleaved(samples);
                        self.buffer_pos = 0;
                    }
                    return true;
                }
                Err(SymphoniaError::DecodeError(_)) => continue,
                Err(_) => return false,
            }
        }
    }
}

impl Iterator for SymphoniaSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref sbuf) = self.sample_buffer {
            if self.buffer_pos >= sbuf.len() {
                if !self.refuel_buffer() {
                    return None;
                }
            }
        } else {
            if !self.refuel_buffer() {
                return None;
            }
        }

        let sample = self.sample_buffer.as_ref()?[self.buffer_pos];
        self.buffer_pos += 1;
        Some(sample)
    }
}

impl Source for SymphoniaSource {
    fn current_span_len(&self) -> Option<usize> {
        self.sample_buffer
            .as_ref()
            .map(|sbuf| sbuf.len().saturating_sub(self.buffer_pos))
    }

    fn channels(&self) -> NonZero<u16> {
        NonZeroU16::new(self.channels).unwrap_or_else(|| NonZeroU16::new(2).unwrap())
    }

    fn sample_rate(&self) -> NonZero<u32> {
        NonZeroU32::new(self.sample_rate).unwrap_or_else(|| NonZeroU32::new(44100).unwrap())
    }

    fn total_duration(&self) -> Option<Duration> {
        self.total_duration
    }
}