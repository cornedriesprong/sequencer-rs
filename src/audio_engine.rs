use crate::audio_platform_cpal::AudioPlatformCpal ;
use cpal::Stream;
use rusty_link::{ AblLink, HostTimeFilter, SessionState };
use std::{
    sync::{mpsc::Receiver, Arc, Mutex},
    time::Duration,
};

pub struct AudioEngine {
    pub stream: Stream,
}

pub enum UpdateSessionState {
    TempoPlus,
    TempoMinus,
    TogglePlaying,
}

impl AudioEngine {
    pub fn new(
        link: &'static AblLink,
        audio_cpal: AudioPlatformCpal,
        input: Receiver<UpdateSessionState>,
        quantum: Arc<Mutex<f64>>,
    ) -> Self {

        let mut prev_beat: i32 = -1;
        let mut host_time_filter = HostTimeFilter::new();
        let mut audio_session_state = SessionState::new();
        link.capture_audio_session_state(&mut audio_session_state);

        // define audio callback
        let callback = move |buffer_size: usize,
                             sample_rate: u64,
                             output_latency: Duration,
                             sample_time: Duration,
                             sample_clock: u64| {

                                 let mut buffer: Vec<f32> = Vec::with_capacity(buffer_size);

                                 // fill up buffer with empty samples?
                                 for _ in 0..buffer_size {
                                     if !audio_session_state.is_playing() {
                                         buffer.push(0.0);
                                         continue;
                                     }

                                 }

                                 let beat = audio_session_state.beat_at_time(link.clock_micros(), 4.) as i32;

                                 if beat != prev_beat {
                                    println!("beat: {}", beat);
                                    prev_beat = beat;
                                 }

                                 // return buffer
                                 buffer
                             };

        // Build audio stream and start playback
        let stream = audio_cpal.build_stream::<f32>(callback);

        Self { stream }
    }
}
