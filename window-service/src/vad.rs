use std::error::Error;
use std::fmt;

// VAD Configuration Constants
const WINDOW_SIZE_SAMPLES: usize = 1024; // ~64ms at 16kHz
const MIN_SPEECH_DURATION_MS: u64 = 250; // Minimum speech duration to consider valid
const MIN_SILENCE_DURATION_MS: u64 = 100; // Minimum silence to end speech
const VAD_THRESHOLD: f32 = 0.5; // Speech probability threshold
const MAX_SPEECH_DURATION_MS: u64 = 30000; // Maximum single speech segment (30s)
const DEFAULT_SAMPLE_RATE: f32 = 16000.0;

#[derive(Debug)]
pub enum VadError {
    ProcessingError(String),
    AudioFormatError(String),
}

impl fmt::Display for VadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VadError::ProcessingError(msg) => write!(f, "VAD processing error: {}", msg),
            VadError::AudioFormatError(msg) => write!(f, "Audio format error: {}", msg),
        }
    }
}

impl Error for VadError {}

#[derive(Clone, Debug)]
pub struct SpeechSegment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub confidence: f32,
    pub audio_data: Vec<f32>,
}

pub struct VadState {
    in_speech: bool,
    speech_start_ms: u64,
    current_time_ms: u64,
    silence_start_ms: u64,
    sample_rate: f32,
    samples_processed: usize,
}

impl VadState {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            in_speech: false,
            speech_start_ms: 0,
            current_time_ms: 0,
            silence_start_ms: 0,
            sample_rate,
            samples_processed: 0,
        }
    }

    pub fn reset(&mut self) {
        self.in_speech = false;
        self.speech_start_ms = 0;
        self.current_time_ms = 0;
        self.silence_start_ms = 0;
        self.samples_processed = 0;
    }

    fn samples_to_ms(&self, samples: usize) -> u64 {
        (samples as f32 / self.sample_rate * 1000.0) as u64
    }

    fn update_time(&mut self, samples: usize) {
        self.samples_processed += samples;
        self.current_time_ms = self.samples_to_ms(self.samples_processed);
    }
}

/// Simple energy-based VAD
pub struct EnergyVad {
    threshold: f32,
    window_size: usize,
    state: VadState,
}

impl EnergyVad {
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold,
            window_size: WINDOW_SIZE_SAMPLES,
            state: VadState::new(DEFAULT_SAMPLE_RATE),
        }
    }

    pub fn with_sample_rate(threshold: f32, sample_rate: f32) -> Self {
        Self {
            threshold,
            window_size: WINDOW_SIZE_SAMPLES,
            state: VadState::new(sample_rate),
        }
    }

    pub fn has_speech(&self, audio_samples: &[f32]) -> bool {
        if audio_samples.is_empty() {
            return false;
        }

        // Calculate RMS energy
        let rms = (audio_samples.iter()
            .map(|&x| x * x)
            .sum::<f32>() / audio_samples.len() as f32)
            .sqrt();

        rms > self.threshold
    }

    pub fn process_pcm_chunk(&self, pcm_data: &[u8]) -> Result<bool, VadError> {
        if pcm_data.len() % 2 != 0 {
            return Err(VadError::AudioFormatError("PCM data length must be even".to_string()));
        }

        let samples: Vec<f32> = pcm_data
            .chunks_exact(2)
            .map(|chunk| {
                let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                sample as f32 / 32768.0
            })
            .collect();

        Ok(self.has_speech(&samples))
    }

    /// Process audio stream and return detected speech segments
    pub fn process_audio_stream(&mut self, pcm_data: &[u8]) -> Result<Option<SpeechSegment>, VadError> {
        if pcm_data.len() % 2 != 0 {
            return Err(VadError::AudioFormatError("PCM data length must be even".to_string()));
        }

        let samples: Vec<f32> = pcm_data
            .chunks_exact(2)
            .map(|chunk| {
                let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                sample as f32 / 32768.0
            })
            .collect();

        // Update timing
        self.state.update_time(samples.len());

        // Calculate energy-based speech probability
        let has_speech = self.has_speech(&samples);
        let speech_confidence = if has_speech {
            // Simple confidence based on RMS energy
            let rms = (samples.iter()
                .map(|&x| x * x)
                .sum::<f32>() / samples.len() as f32)
                .sqrt();
            (rms / self.threshold).min(1.0)
        } else {
            0.0
        };

        // State machine for speech detection
        self.update_speech_state(has_speech, speech_confidence)
    }

    fn update_speech_state(&mut self, is_speech: bool, confidence: f32) -> Result<Option<SpeechSegment>, VadError> {
        match (self.state.in_speech, is_speech) {
            // Start of speech detection
            (false, true) => {
                self.state.in_speech = true;
                self.state.speech_start_ms = self.state.current_time_ms;
                self.state.silence_start_ms = 0; // Reset silence tracker
                Ok(None)
            },
            
            // Continuing speech
            (true, true) => {
                // Reset silence tracker since we detected speech again
                self.state.silence_start_ms = 0;
                
                // Check for maximum speech duration
                let speech_duration = self.state.current_time_ms - self.state.speech_start_ms;
                if speech_duration > MAX_SPEECH_DURATION_MS {
                    return self.end_speech_segment(confidence);
                }
                Ok(None)
            },
            
            // Start of silence during speech
            (true, false) => {
                if self.state.silence_start_ms == 0 {
                    self.state.silence_start_ms = self.state.current_time_ms;
                }
                
                let silence_duration = self.state.current_time_ms - self.state.silence_start_ms;
                if silence_duration >= MIN_SILENCE_DURATION_MS {
                    return self.end_speech_segment(confidence);
                }
                Ok(None)
            },
            
            // Continuing silence
            (false, false) => {
                Ok(None)
            },
        }
    }

    fn end_speech_segment(&mut self, confidence: f32) -> Result<Option<SpeechSegment>, VadError> {
        let speech_duration = self.state.current_time_ms - self.state.speech_start_ms;
        
        if speech_duration >= MIN_SPEECH_DURATION_MS {
            let segment = SpeechSegment {
                start_ms: self.state.speech_start_ms,
                end_ms: self.state.current_time_ms,
                confidence,
                audio_data: Vec::new(), // Could store the actual audio here if needed
            };
            
            // Reset speech state
            self.state.in_speech = false;
            self.state.silence_start_ms = 0;
            
            Ok(Some(segment))
        } else {
            // Speech was too short, ignore it
            self.state.in_speech = false;
            self.state.silence_start_ms = 0;
            Ok(None)
        }
    }

    pub fn reset(&mut self) {
        self.state.reset();
    }

    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold;
    }

    pub fn get_threshold(&self) -> f32 {
        self.threshold
    }

    pub fn is_in_speech(&self) -> bool {
        self.state.in_speech
    }

    pub fn get_current_time_ms(&self) -> u64 {
        self.state.current_time_ms
    }
}

/// Advanced energy-based VAD with additional features
pub struct AdvancedEnergyVad {
    base_vad: EnergyVad,
    energy_history: Vec<f32>,
    history_size: usize,
    adaptive_threshold: bool,
    base_threshold: f32,
}

impl AdvancedEnergyVad {
    pub fn new(threshold: f32) -> Self {
        Self {
            base_vad: EnergyVad::new(threshold),
            energy_history: Vec::new(),
            history_size: 20, // smaller window to avoid long-term drift
            adaptive_threshold: true,
            base_threshold: threshold,
        }
    }

    pub fn with_adaptive_threshold(mut self, adaptive: bool) -> Self {
        self.adaptive_threshold = adaptive;
        self
    }

    pub fn process_pcm_chunk(&mut self, pcm_data: &[u8]) -> Result<bool, VadError> {
        if pcm_data.len() % 2 != 0 {
            return Err(VadError::AudioFormatError("PCM data length must be even".to_string()));
        }

        let samples: Vec<f32> = pcm_data
            .chunks_exact(2)
            .map(|chunk| {
                let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                sample as f32 / 32768.0
            })
            .collect();

        // Calculate RMS energy
        let rms = (samples.iter()
            .map(|&x| x * x)
            .sum::<f32>() / samples.len() as f32)
            .sqrt();

        // Update energy history
        self.energy_history.push(rms);
        if self.energy_history.len() > self.history_size {
            self.energy_history.remove(0);
        }

        // Adaptive thresholding
        if self.adaptive_threshold && self.energy_history.len() > 8 {
            let avg_energy: f32 = self.energy_history.iter().sum::<f32>() / self.energy_history.len() as f32;
            // Clamp adaptive threshold within a band around base to prevent overshoot
            let mut adaptive_threshold = avg_energy * 1.5; // less aggressive multiplier
            let min_t = (self.base_threshold * 0.5).max(0.001);
            let max_t = self.base_threshold * 2.0;
            if adaptive_threshold < min_t { adaptive_threshold = min_t; }
            if adaptive_threshold > max_t { adaptive_threshold = max_t; }
            self.base_vad.set_threshold(adaptive_threshold);
        }

        Ok(rms > self.base_vad.get_threshold())
    }

    pub fn process_audio_stream(&mut self, pcm_data: &[u8]) -> Result<Option<SpeechSegment>, VadError> {
        self.base_vad.process_audio_stream(pcm_data)
    }

    pub fn reset(&mut self) {
        self.base_vad.reset();
        self.energy_history.clear();
    }

    pub fn get_average_energy(&self) -> f32 {
        if self.energy_history.is_empty() {
            0.0
        } else {
            self.energy_history.iter().sum::<f32>() / self.energy_history.len() as f32
        }
    }
}

pub enum VadMode { Energy, Adaptive }

pub trait VadDecider: Send {
    fn process_chunk(&mut self, pcm: &[u8]) -> bool;
}

impl VadDecider for EnergyVad {
    fn process_chunk(&mut self, pcm: &[u8]) -> bool {
        self.process_pcm_chunk(pcm).unwrap_or(true)
    }
}

impl VadDecider for AdvancedEnergyVad {
    fn process_chunk(&mut self, pcm: &[u8]) -> bool {
        self.process_pcm_chunk(pcm).unwrap_or(true)
    }
}

pub fn make_vad(mode: VadMode, threshold: f32) -> Box<dyn VadDecider> {
    match mode {
        VadMode::Energy => Box::new(EnergyVad::new(threshold)),
        VadMode::Adaptive => Box::new(AdvancedEnergyVad::new(threshold).with_adaptive_threshold(true)),
    }
}

pub fn mode_from_env() -> VadMode {
    match std::env::var("VAD_MODE").unwrap_or_else(|_| "energy".to_string()).to_lowercase().as_str() {
        "adaptive" => VadMode::Adaptive,
        _ => VadMode::Energy,
    }
}

pub fn threshold_from_env(default: f32) -> f32 {
    match std::env::var("VAD_THRESHOLD").ok().and_then(|s| s.parse::<f32>().ok()) {
        Some(v) => v,
        None => default,
    }
}
