use crate::audio_platform_cpal::AudioPlatformCpal;
use crate::sequencer::{Sequencer, SequencerConfig, MidiEvent};
use coremidi::{Client, Destination, PacketBuffer, OutputPort};
use cpal::Stream;
use rusty_link::{AblLink, SessionState};
use std::{
    sync::{mpsc::Receiver, Arc, Mutex},
    time::Duration,
    mem::MaybeUninit
};
use wmidi::MidiMessage;
use mach2::mach_time::{mach_absolute_time, mach_timebase_info};

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

// Using CoreMIDI objects with `lazy_static` in this example, because the `cpal` audio callback requires all variables
// to be moved into the callback, or to have a 'static lifetime. This is just one possible design solution.
lazy_static! {
    static ref DESTINATION: Destination = Destination::from_index(0).unwrap();
    static ref CLIENT: Client = Client::new("sequencer-rs").unwrap();
    static ref OUTPUT_PORT: OutputPort = CLIENT.output_port("sequencer-rs-midiout").unwrap();
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
        let config = SequencerConfig::new(120., 44100, 512.0);
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
            let now = unsafe { mach_absolute_time() };
            let info = timebase_info();
            let ms_to_host_ticks: u64 = (1.0 / ((info.numer as f64 / info.denom as f64) * 1.0e-6)) as u64;

            let mut midi: Vec<MidiEvent> = Vec::with_capacity(10);
            sequencer.render_timeline(now, beat_position, &mut midi);

            let output_latency_in_ms = output_latency.as_millis();
            //let output_latency_host_ticks = output_latency_in_ms as u64 * ms_to_host_ticks;

            for event in midi.iter() {
                let offset_in_ms = event.offset() / 44.1;
                let offset_in_host_ticks = offset_in_ms as u64 * ms_to_host_ticks;
                //println!("ms to host ticks              {}", ms_to_host_ticks);
                //println!("offset in samples             {}", event.offset());
                //println!("offset in ms                  {}", offset_in_ms);
                //println!("offset in host ticks          {}", offset_in_host_ticks);
                // TODO: account for output latency
                //let timestamp = now + offset_in_host_ticks as u64 - output_latency_host_ticks as u64;
                //println!("output latency in host ticks  {}", output_latency_host_ticks);
                let timestamp = now + offset_in_host_ticks as u64;

                match event.message() {
                    MidiMessage::NoteOn(_, note, velocity) => {
                        let data = [NOTE_ON, u8::from(*note), u8::from(*velocity)];
                        let p = PacketBuffer::new(timestamp as u64, &data);
                        OUTPUT_PORT.send(&DESTINATION, &p).unwrap();                   
                        //println!("note on at time   {}", timestamp);
                    },
                    MidiMessage::NoteOff(_, note, _) => {
                        let data = [NOTE_OFF, u8::from(*note), 0];
                        let p = PacketBuffer::new(timestamp as u64, &data);
                        OUTPUT_PORT.send(&DESTINATION, &p).unwrap();                   
                        //println!("note off at time  {}", timestamp);
                    },
                    _ => println!("unknown item"),
                }
                //println!("---------------------------");
            }

            buffer
        };

        // Build audio stream and start playback
        let stream = audio_cpal.build_stream::<f32>(callback);

        Self { stream }
    }
}

fn timebase_info() -> mach_timebase_info {
    let mut info = MaybeUninit::<mach_timebase_info>::uninit();
    unsafe { mach_timebase_info(info.as_mut_ptr()) };
    unsafe { info.assume_init() }
}
