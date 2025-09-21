use rodio::{OutputStream, Sink, source::SineWave};
use std::error::Error;

use chip8_core::AudioPlayer;

const BEEP_FREQUENCY: f32 = 440.0;

pub struct Audio {
    sink: Sink,
    _stream: OutputStream, // Keep it here so it doesn't get dropped
}

impl Audio {
    pub fn new() -> Result<Audio, Box<dyn Error>> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        sink.append(SineWave::new(BEEP_FREQUENCY));
        sink.pause();

        Ok(Audio {
            sink,
            _stream: stream,
        })
    }
}

impl AudioPlayer for Audio {
    fn play(&self) {
        self.sink.play();
    }

    fn pause(&self) {
        self.sink.pause();
    }
}
