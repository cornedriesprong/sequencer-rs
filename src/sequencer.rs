use float_extras::f64::modf;
use wmidi::{Channel, MidiMessage, Note, U7};

const SEQUENCE_COUNT: usize = 8;
const MAX_EVENT_COUNT: usize = 2048;
const PPQ: i32 = 96; // pulses per quarter note
const NOTE_ON: u8 = 0x90;
const NOTE_OFF: u8 = 0x80;

pub struct SequencerConfig {
    tempo: f64,
    sample_rate: u64,
    buffer_size: usize,
}

impl SequencerConfig {
    pub fn new(tempo: f64, sample_rate: u64, buffer_size: usize) -> Self {
        Self {
            tempo,
            sample_rate,
            buffer_size,
        }
    }
}

#[derive(Clone)]
pub struct SequencerEvent<'a> {
    timestamp: f64,
    message: MidiMessage<'a>,
}

#[derive(Clone)]
struct MIDISequence<'a> {
    length: f64,
    events: Vec<SequencerEvent<'a>>,
}

impl<'a> MIDISequence<'a> {
    pub fn new(length: f64) -> MIDISequence<'a> {
        MIDISequence {
            length,
            events: Vec::new(),
        }
    }

    pub fn add_event(&mut self, event: SequencerEvent<'a>) {
        self.events.push(event);
    }
}

pub struct Sequencer<'a> {
    config: SequencerConfig,
    sequences: Vec<MIDISequence<'a>>,
}

impl<'a> Sequencer<'a> {
    pub fn new(config: SequencerConfig) -> Self {
        let mut sequences = Vec::new();
        let mut sequence = MIDISequence::new(4.);

        let note_on = MidiMessage::NoteOn(Channel::Ch1, Note::C4, U7::from_u8_lossy(100));
        let event1 = SequencerEvent {
            timestamp: 0.5,
            message: note_on,
        };
        sequence.add_event(event1);

        let note_off = MidiMessage::NoteOff(Channel::Ch1, Note::C4, U7::from_u8_lossy(0));
        let event2 = SequencerEvent {
            timestamp: 1.0,
            message: note_off,
        };
        sequence.add_event(event2);

        sequences.push(sequence);

        Self { config, sequences }
    }

    pub fn render_timeline(&self, beat_position: f64) {
        for sequence in &self.sequences {
            let buffer_start_time = Self::mod_position(self, beat_position, sequence.length);
            let length_in_samples = Self::beat_to_samples(self, sequence.length);
            let buffer_end_time =
                (buffer_start_time + self.config.buffer_size as f64) % length_in_samples;

            for event in &sequence.events {
                // offset in samples from beginning of buffer
                let event_offset = Self::beat_to_samples(self, event.timestamp);
                // determine if the event should occur in the current buffer
                let is_in_buffer =
                    event_offset >= buffer_start_time && event_offset <= buffer_end_time;

                if is_in_buffer {
                    match event.message {
                        MidiMessage::NoteOn(_, note, velocity) => {
                            println!("note on {} with velocity {:?}", note, velocity)
                        }
                        MidiMessage::NoteOff(_, note, velocity) => {
                            println!("note off {} with velocity {:?}", note, velocity)
                        }
                        _ => println!("unknown item"),
                    }
                }
            }
        }
    }

    fn beat_to_samples(&self, beat: f64) -> f64 {
        beat / &self.config.tempo * 60. * &(self.config.sample_rate as f64)
    }

    fn mod_position(&self, beat: f64, length: f64) -> f64 {
        let position_in_samples = Self::beat_to_samples(self, beat);
        let length_in_samples = Self::beat_to_samples(self, length);
        position_in_samples % length_in_samples
    }

    fn samples_per_beat(sample_rate: u64, tempo: f64) -> f64 {
        sample_rate as f64 * 60. / tempo
    }

    fn samples_per_subtick(sample_rate: u64, tempo: f64) -> f64 {
        Self::samples_per_beat(sample_rate, tempo) / PPQ as f64
    }

    fn subtick_position(beat_position: f64) -> i64 {
        let (_, fractional) = modf(beat_position);
        (PPQ as f64 * fractional).floor() as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beat_to_samples_zero() {
        let config = SequencerConfig {
            tempo: 120.,
            sample_rate: 44100,
            buffer_size: 512,
        };
        let sequencer = Sequencer::new(config);
        let result = sequencer.beat_to_samples(0.);
        assert_eq!(result, 0.);
    }

    #[test]
    fn beat_to_samples_one() {
        let config = SequencerConfig {
            tempo: 120.,
            sample_rate: 44100,
            buffer_size: 512,
        };
        let sequencer = Sequencer::new(config);
        let result = sequencer.beat_to_samples(1.);
        assert_eq!(result, 22050.);
    }

    #[test]
    fn mod_position_zero() {
        let config = SequencerConfig {
            tempo: 120.,
            sample_rate: 44100,
            buffer_size: 1024,
        };
        let sequencer = Sequencer::new(config);
        let result = sequencer.mod_position(0., 1.);
        assert_eq!(result, 0.);
    }

    #[test]
    fn mod_position_one() {
        let config = SequencerConfig {
            tempo: 120.,
            sample_rate: 44100,
            buffer_size: 1024,
        };
        let sequencer = Sequencer::new(config);
        let result = sequencer.mod_position(0., 1.);
        assert_eq!(result, 0.);
    }

    #[test]
    fn mod_position_longer() {
        let config = SequencerConfig {
            tempo: 120.,
            sample_rate: 44100,
            buffer_size: 1024,
        };
        let sequencer = Sequencer::new(config);
        let result = sequencer.mod_position(1., 2.);
        assert_eq!(result, 22050.);
    }

    #[test]
    fn samples_per_beat_test() {
        let result = Sequencer::samples_per_beat(44100, 120.);
        assert_eq!(result, 22050.);
    }

    #[test]
    fn samples_per_subtick_test() {
        let result = Sequencer::samples_per_subtick(44100, 120.);
        assert_eq!(result, 229.6875);
    }

    #[test]
    fn subtick_position_test() {
        let result = Sequencer::subtick_position(0.5);
        assert_eq!(result, 48);
    }
}
