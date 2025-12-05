use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;
use log::{debug, info, warn, trace};

use super::audio_analyzer::AudioAnalyzer;

/// Wrapper to tap into audio samples and convert stereo to mono i16 for FFT
struct TappedSource<I> {
    inner: I,
    tx: Sender<i16>,
    sample_rate: u32,
    left_sample: Option<f32>,  // Buffer for stereo->mono conversion
}

impl<I> Iterator for TappedSource<I>
where
    I: Iterator<Item = f32>,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.inner.next()?;

        // For stereo: average left and right channels for FFT
        if let Some(left) = self.left_sample.take() {
            // This is the right channel, average with left
            let mono = (left + sample) / 2.0;
            let sample_i16 = (mono * 32767.0).clamp(-32768.0, 32767.0) as i16;
            let _ = self.tx.send(sample_i16);
        } else {
            // This is the left channel, save it
            self.left_sample = Some(sample);
        }

        Some(sample)
    }
}

impl<I> rodio::Source for TappedSource<I>
where
    I: rodio::Source<Item = f32>,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.inner.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.inner.total_duration()
    }
}

/// Plays audio from a file and feeds it to FFT analysis using AudioAnalyzer
/// Returns Some(()) on success, None on error
pub fn start_file_audio(
    bass_energy: Arc<Mutex<f32>>,
    mid_energy: Arc<Mutex<f32>>,
    high_energy: Arc<Mutex<f32>>,
    file_path: &str,
) -> Option<()> {
    use rodio::{Decoder, OutputStream, Sink, Source};
    use std::fs::File;
    use std::io::BufReader;

    info!("Loading audio file: {}", file_path);

    // Open audio file for validation
    let file = match File::open(file_path) {
        Ok(f) => {
            debug!("Audio file opened successfully");
            f
        }
        Err(e) => {
            warn!("Failed to open audio file '{}': {}", file_path, e);
            return None;
        }
    };

    let source = match Decoder::new(BufReader::new(file)) {
        Ok(s) => {
            debug!("Audio decoder created successfully");
            s
        }
        Err(e) => {
            warn!("Failed to decode audio file: {}", e);
            return None;
        }
    };

    // Get sample rate and channels
    let sample_rate = source.sample_rate();
    let channels = source.channels();
    info!("Audio format: {} Hz, {} channels", sample_rate, channels);

    // Create channel for streaming samples to FFT thread (as i16)
    let (tx, rx): (Sender<i16>, Receiver<i16>) = mpsc::channel();

    // Spawn FFT analysis thread with AudioAnalyzer
    std::thread::spawn(move || {
        let mut analyzer = AudioAnalyzer::new(bass_energy, mid_energy, high_energy);
        let mut sample_buffer = Vec::with_capacity(4096);

        debug!("Audio FFT analyzer thread started");

        // Receive samples from playback thread
        while let Ok(sample) = rx.recv() {
            sample_buffer.push(sample);

            // Process in batches for efficiency
            if sample_buffer.len() >= 2048 {
                analyzer.process_samples(&sample_buffer);
                sample_buffer.clear();
            }
        }

        debug!("Audio FFT analyzer thread ended");
    });

    // Start playback and stream samples to FFT
    let file_path_owned = file_path.to_string();
    std::thread::spawn(move || {
        debug!("Audio playback thread started");

        let (_stream, stream_handle) = match OutputStream::try_default() {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to create output stream: {}", e);
                return;
            }
        };

        let sink = match Sink::try_new(&stream_handle) {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to create sink: {}", e);
                return;
            }
        };

        loop {
            let file = match File::open(&file_path_owned) {
                Ok(f) => f,
                Err(_) => break,
            };

            let source = match Decoder::new(BufReader::new(file)) {
                Ok(s) => s,
                Err(_) => break,
            };

            let sample_rate = source.sample_rate();

            // Convert to f32 and wrap with tapping source
            let tx_clone = tx.clone();
            let converted = source.convert_samples::<f32>();

            let tapped = TappedSource {
                inner: converted,
                tx: tx_clone,
                sample_rate,
                left_sample: None,
            };

            sink.append(tapped);
            sink.sleep_until_end();

            trace!("Audio playback loop completed, restarting");
        }

        debug!("Audio playback thread ended");
    });

    info!("Audio playback and FFT analysis started successfully");
    Some(())
}
