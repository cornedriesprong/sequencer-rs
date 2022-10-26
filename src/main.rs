use crate::{
    audio_engine::{ AudioEngine, UpdateSessionState }, audio_platform_cpal::AudioPlatformCpal
};

use rusty_link::{AblLink, SessionState};

use std::sync::{
    mpsc, Arc, Mutex,
};

mod audio_engine;
mod audio_platform_cpal;
mod sequencer;

pub struct State {
    pub link: AblLink,
    pub session_state: SessionState,
    pub running: bool,
    pub quantum: f64,
}

impl State {
    pub fn new() -> Self {
        Self {
            link: AblLink::new(120.),
            session_state: SessionState::new(),
            running: true,
            quantum: 4.,
        }
    }

    pub fn capture_app_state(&mut self) {
        self.link.capture_app_session_state(&mut self.session_state);
    }

    pub fn commit_app_state(&mut self) {
        self.link.commit_app_session_state(&self.session_state);
    }
}

fn print_state(state: &mut State) {
    //let time = state.link.clock_micros();
    state.link.set_tempo_callback(|tempo| println!("tempo: {}", tempo));
}

#[macro_use]
extern crate lazy_static;
// Using AblLink with `lazy_static` in this example, because the `cpal` audio callback requires all variables
// to be moved into the callback, or to have a 'static lifetime. This is just one possible design solution.
lazy_static! {
    static ref ABL_LINK: AblLink = AblLink::new(120.);
}

fn main() {
    // init Audio Device and print device info
    let audio_platform = AudioPlatformCpal::new();
    let (_input_tx, input_rx) = mpsc::channel::<UpdateSessionState>();
    let quantum = Arc::new(Mutex::new(4.));
    let quantum_clone2 = Arc::clone(&quantum);
    let _audio_engine = AudioEngine::new(&ABL_LINK, audio_platform, input_rx, quantum_clone2);

    // init Link state
    let mut state = State::new();

    ABL_LINK.set_tempo_callback(|tempo| println!("tempo: {}", tempo));

    println!("link enabled!");
    state.link.enable(true);

    '_main_loop: while state.running {
        print_state(&mut state);
    }
}

