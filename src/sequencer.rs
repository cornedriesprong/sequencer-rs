use float_extras::f64::modf;

const SEQUENCE_COUNT: usize = 8;
const MAX_EVENT_COUNT: usize = 2048;
const PPQ: i32 = 96; // pulses per quarter note

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

#[derive(Copy, Clone)]
struct MIDIEvent {
    time: f64,
    note: u8,
    velocity: u8,
}

#[derive(Clone)]
struct MIDISequence {
    length: f64,
    events: Vec<MIDIEvent>,
}

impl MIDISequence {
    pub fn new(length: f64) -> MIDISequence {
        MIDISequence {
            length,
            events: Vec::new(),
        }
    }
}

pub struct Sequencer {
    config: SequencerConfig,
    sequences: Vec<MIDISequence>,
}

impl Sequencer {
    pub fn new(config: SequencerConfig) -> Self {
        let mut sequences = Vec::new();
        for _ in 0..SEQUENCE_COUNT {
            sequences.push(MIDISequence::new(8.));
        }

        Self { config, sequences }
    }

    pub fn render_timeline(&self, beat_position: f64) {
        for sequence in &self.sequences {
            let buffer_start_time =
                Self::mod_position(beat_position, sequence.length, &self.config);
            println!("buffer_start_time: {}", buffer_start_time);
        }
    }

    fn beat_to_samples(beat: f64, tempo: f64, sample_rate: u64) -> f64 {
        beat / tempo * 60. * sample_rate as f64
    }

    fn mod_position(beat: f64, length: f64, config: &SequencerConfig) -> f64 {
        let position_in_samples = Self::beat_to_samples(beat, config.tempo, config.sample_rate);
        let length_in_samples = Self::beat_to_samples(length, config.tempo, config.sample_rate);
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
        let result = Sequencer::beat_to_samples(0., 120., 48000);
        assert_eq!(result, 0.);
    }

    #[test]
    fn beat_to_samples_one() {
        let result = Sequencer::beat_to_samples(1., 120., 44100);
        assert_eq!(result, 22050.);
    }

    #[test]
    fn mod_position_zero() {
        let config = SequencerConfig {
            tempo: 120.,
            sample_rate: 44100,
            buffer_size: 1024,
        };
        let result = Sequencer::mod_position(0., 1., &config);
        assert_eq!(result, 0.);
    }

    #[test]
    fn mod_position_one() {
        let config = SequencerConfig {
            tempo: 120.,
            sample_rate: 44100,
            buffer_size: 1024,
        };
        let result = Sequencer::mod_position(1., 1., &config);
        assert_eq!(result, 0.);
    }

    #[test]
    fn mod_position_longer() {
        let config = SequencerConfig {
            tempo: 120.,
            sample_rate: 44100,
            buffer_size: 1024,
        };
        let result = Sequencer::mod_position(1., 2., &config);
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
