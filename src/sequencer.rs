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
    buffer_size: f64,
}

impl SequencerConfig {
    pub fn new(tempo: f64, sample_rate: u64, buffer_size: f64) -> Self {
        Self {
            tempo,
            sample_rate,
            buffer_size,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SequencerEvent<'a> {
    timestamp: f64,
    message: MidiMessage<'a>,
}

#[derive(Clone, Debug)]
pub struct MidiEvent<'a> {
    offset: f64,
    message: MidiMessage<'a>,
}

impl<'a> MidiEvent<'a> {

    pub fn offset(&self) -> f64 {
        self.offset
    }

    pub fn message(&self) -> &MidiMessage {
        &self.message
    }
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
        let mut sequence = MIDISequence::new(1.);

        let fr = 1.0 / 16.;

        let event1 = SequencerEvent {
            timestamp: 0.,
            message: MidiMessage::NoteOn(Channel::Ch1, Note::C4, U7::from_u8_lossy(100)),
        };
        sequence.add_event(event1);

        let event2 = SequencerEvent {
            timestamp: fr,
            message: MidiMessage::NoteOff(Channel::Ch1, Note::C4, U7::from_u8_lossy(0)),
        };
        sequence.add_event(event2);

        let event3 = SequencerEvent {
            timestamp: fr * 2.,
            message: MidiMessage::NoteOn(Channel::Ch1, Note::C4, U7::from_u8_lossy(100)),
        };
        sequence.add_event(event3);

        let event4 = SequencerEvent {
            timestamp: fr * 3.,
            message: MidiMessage::NoteOff(Channel::Ch1, Note::C4, U7::from_u8_lossy(0)),
        };
        sequence.add_event(event4);

        let event5 = SequencerEvent {
            timestamp: fr * 4.,
            message: MidiMessage::NoteOn(Channel::Ch1, Note::C4, U7::from_u8_lossy(100)),
        };
        sequence.add_event(event5);

        let event6 = SequencerEvent {
            timestamp: fr * 5.,
            message: MidiMessage::NoteOff(Channel::Ch1, Note::C4, U7::from_u8_lossy(0)),
        };
        sequence.add_event(event6);

        let event7 = SequencerEvent {
            timestamp: fr * 6.,
            message: MidiMessage::NoteOn(Channel::Ch1, Note::C4, U7::from_u8_lossy(100)),
        };
        sequence.add_event(event7);

        let event8 = SequencerEvent {
            timestamp: fr * 7.,
            message: MidiMessage::NoteOff(Channel::Ch1, Note::C4, U7::from_u8_lossy(0)),
        };
        sequence.add_event(event8);

        let event9 = SequencerEvent {
            timestamp: fr * 8.,
            message: MidiMessage::NoteOn(Channel::Ch1, Note::C4, U7::from_u8_lossy(100)),
        };
        sequence.add_event(event9);

        let event10 = SequencerEvent {
            timestamp: fr * 9.,
            message: MidiMessage::NoteOff(Channel::Ch1, Note::C4, U7::from_u8_lossy(0)),
        };
        sequence.add_event(event10);

        let event11 = SequencerEvent {
            timestamp: fr * 10.,
            message: MidiMessage::NoteOn(Channel::Ch1, Note::C4, U7::from_u8_lossy(100)),
        };
        sequence.add_event(event11);

        let event12 = SequencerEvent {
            timestamp: fr * 11.,
            message: MidiMessage::NoteOff(Channel::Ch1, Note::C4, U7::from_u8_lossy(0)),
        };
        sequence.add_event(event12);

        let event13 = SequencerEvent {
            timestamp: fr * 12.,
            message: MidiMessage::NoteOn(Channel::Ch1, Note::C4, U7::from_u8_lossy(100)),
        };
        sequence.add_event(event13);

        let event14 = SequencerEvent {
            timestamp: fr * 13.,
            message: MidiMessage::NoteOff(Channel::Ch1, Note::C4, U7::from_u8_lossy(0)),
        };
        sequence.add_event(event14);

        let event15 = SequencerEvent {
            timestamp: fr * 14.,
            message: MidiMessage::NoteOn(Channel::Ch1, Note::C4, U7::from_u8_lossy(100)),
        };
        sequence.add_event(event15);

        let event16 = SequencerEvent {
            timestamp: fr * 15.,
            message: MidiMessage::NoteOff(Channel::Ch1, Note::C4, U7::from_u8_lossy(0)),
        };
        sequence.add_event(event16);

        //let event17 = SequencerEvent {
            //timestamp: fr * 16.,
            //message: MidiMessage::NoteOff(Channel::Ch1, Note::C4, U7::from_u8_lossy(0)),
        //};
        //sequence.add_event(event17);

        sequences.push(sequence);

        Self { config, sequences }
    }

    pub fn render_timeline(&self, now: u64, beat_position: f64, midi: &mut Vec<MidiEvent<'a>>) {
        for sequence in &self.sequences {
            let buffer_start_time = Self::mod_position(self, beat_position, sequence.length);
            let loop_length_in_samples = Self::beat_to_samples(self, sequence.length);
            let buffer_end_time =
                (buffer_start_time + self.config.buffer_size as f64) % loop_length_in_samples;

            //println!("beat position: {}", beat_position);
            //println!("buffer start:  {}", buffer_start_time);
            //println!("buffer end:    {}", buffer_end_time);
            //println!("---------------");
            for event in &sequence.events {
                // offset in samples since beginning of buffer
                let event_offset = Self::beat_to_samples(self, event.timestamp);
                // check whether the event should occur in the current buffer
                let is_in_buffer =
                    event_offset >= buffer_start_time && event_offset < buffer_end_time;
                let loops_around = (buffer_start_time + self.config.buffer_size > loop_length_in_samples) 
                    && event_offset <= buffer_end_time;
                // TODO: add loop around logic

                let mut offset;
                offset = if event_offset == 0. { buffer_start_time } else {};
                if loops_around {
                    println!("LOOP!!!!!!!!!!!!!!!!!");
                    let loop_restart_in_buffer = loop_length_in_samples - buffer_start_time;
                    println!("{}", loop_length_in_samples);
                    offset += loop_restart_in_buffer;
                } 
                if is_in_buffer || loops_around {
                    println!("buffer start  {}", buffer_start_time);
                    println!("timestamp     {}", event.timestamp);
                    println!("event time    {}", event_offset);
                    println!("offset        {}", offset);
                    println!("-----------------------");

                    let midi_event = MidiEvent {
                        offset: event_offset - buffer_start_time,
                        message: event.message.clone(),
                    };
                    midi.push(midi_event);

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
