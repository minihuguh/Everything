use std::error::Error;
use std::fs::File;
use std::num::{NonZero, NonZeroU16, NonZeroU32};
use std::path::Path;
use std::time::Duration;

use rodio::Source;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{CodecRegistry, Decoder, DecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia_adapter_libopus::OpusDecoder;

fn build_codec_registry() -> CodecRegistry {
    let mut registry = CodecRegistry::new();

    // 1. Registrar todos los códecs por defecto de Symphonia (MP3, FLAC, WAV, AAC, etc.)
    symphonia::default::register_enabled_codecs(&mut registry);

    // 2. Registrar el adaptador Libopus para decodificación de audio Opus (.opus / .ogg)
    registry.register_all::<OpusDecoder>();

    registry
}

pub struct SymphoniaSource {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    channels: u16,
    sample_rate: u32,
    total_duration: Option<Duration>,
    sample_buffer: Option<SampleBuffer<f32>>,
    buffer_pos: usize,
}

impl SymphoniaSource {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let path = path.as_ref();
        let file = Box::new(File::open(path)?);

        let mss = MediaSourceStream::new(file, Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let probed = symphonia::default::get_probe().format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )?;

        let format = probed.format;

        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
            .ok_or("No se encontró ninguna pista de audio soportada")?;

        let track_id = track.id;

        let sample_rate = track
            .codec_params
            .sample_rate
            .ok_or("No se pudo determinar sample rate")?;

        let channels = track
            .codec_params
            .channels
            .map(|c| c.count() as u16)
            .unwrap_or(2);

        let total_duration = track.codec_params.n_frames.map(|n_frames| {
            Duration::from_secs_f64(n_frames as f64 / sample_rate as f64)
        });

        let registry = build_codec_registry();
        let decoder = registry.make(&track.codec_params, &DecoderOptions::default())?;

        Ok(Self {
            format,
            decoder,
            track_id,
            channels,
            sample_rate,
            total_duration,
            sample_buffer: None,
            buffer_pos: 0,
        })
    }

    fn refuel_buffer(&mut self) -> bool {
        loop {
            let packet = match self.format.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    return false;
                }
                Err(_) => return false,
            };

            if packet.track_id() != self.track_id {
                continue;
            }

            match self.decoder.decode(&packet) {
                Ok(audio_buf) => {
                    let capacity = audio_buf.capacity() as u64;
                    let spec = *audio_buf.spec();

                    if self.sample_buffer.is_none() || self.sample_buffer.as_ref().unwrap().capacity() < capacity as usize {
                        self.sample_buffer = Some(SampleBuffer::new(capacity, spec));
                    }

                    if let Some(ref mut sample_buffer) = self.sample_buffer {
                        sample_buffer.copy_interleaved_ref(audio_buf);
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
            if self.buffer_pos >= sbuf.samples().len() {
                if !self.refuel_buffer() {
                    return None;
                }
            }
        } else {
            if !self.refuel_buffer() {
                return None;
            }
        }

        let sample = self.sample_buffer.as_ref()?.samples()[self.buffer_pos];
        self.buffer_pos += 1;
        Some(sample)
    }
}

impl Source for SymphoniaSource {
    fn current_span_len(&self) -> Option<usize> {
        self.sample_buffer
            .as_ref()
            .map(|sbuf| sbuf.samples().len().saturating_sub(self.buffer_pos))
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