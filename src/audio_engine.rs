use crate::audio_platform_cpal::AudioPlatformCpal;
use crate::sequencer::{Sequencer, SequencerConfig, SequencerEvent};
use coremidi::{Client, Destination, Destinations, EventBuffer, PacketBuffer, Protocol};
use cpal::Stream;
use rusty_link::{AblLink, HostTimeFilter, SessionState};
use std::thread::sleep;
use std::{
    sync::{mpsc::Receiver, Arc, Mutex},
    time::Duration,
};
use wmidi::MidiMessage;

const NOTE_ON: u8 = 0x90;
const NOTE_OFF: u8 = 0x80;

pub enum UpdateSessionState {
    TempoPlus,
    TempoMinus,
    TogglePlaying,
}

pub struct AudioEngine {
    pub stream: Stream,
}

impl AudioEngine {
    pub fn new(
        link: &'static AblLink,
        audio_cpal: AudioPlatformCpal,
        input: Receiver<UpdateSessionState>,
        quantum: Arc<Mutex<f64>>,
    ) -> Self {
        //let prev_beat: i32 = -1;
        //let host_time_filter = HostTimeFilter::new();
        let mut audio_session_state = SessionState::new();
        link.capture_audio_session_state(&mut audio_session_state);

        // TODO: get actual buffer size and sample time from cpal, and sync tempo with Link
        let config = SequencerConfig::new(200., 44100, 512);
        let sequencer = Sequencer::new(config);

        // define audio callback
        let callback = move |buffer_size: usize,
                             sample_rate: u64,
                             output_latency: Duration,
                             sample_time: Duration,
                             sample_clock: u64| {
            let mut buffer: Vec<f32> = Vec::with_capacity(buffer_size);

            // fill up buffer with silence
            for _ in 0..buffer_size {
                if !audio_session_state.is_playing() {
                    buffer.push(0.);
                    continue;
                }
            }

            let beat_position = audio_session_state.beat_at_time(link.clock_micros(), 4.);
            // TODO: make sure we don't exceed capacity
            let mut midi: Vec<SequencerEvent> = Vec::with_capacity(10);
            sequencer.render_timeline(beat_position, &mut midi);

            let destination_index = 0;
            let destination = Destination::from_index(destination_index).unwrap();
            let client = Client::new("sequencer-rs").unwrap();
            let output_port = client.output_port("sequencer-rs-midiout").unwrap();
            let timestamp = 0;

            for event in midi.iter() {
                let message = event.message();
                match message {
                    MidiMessage::NoteOn(_, note, velocity) => {
                        let data = [NOTE_ON, u8::from(*note), u8::from(*velocity)];
                        let p = PacketBuffer::new(timestamp, &data);
                        output_port.send(&destination, &p).unwrap();                   
                    },
                    MidiMessage::NoteOff(_, note, _) => {
                        let data = [NOTE_OFF, u8::from(*note), 0];
                        let p = PacketBuffer::new(timestamp, &data);
                        output_port.send(&destination, &p).unwrap();                   
                    },
                    _ => println!("unknown item"),
                }
            }

            buffer
        };

        // Build audio stream and start playback
        let stream = audio_cpal.build_stream::<f32>(callback);

        Self { stream }
    }
}

