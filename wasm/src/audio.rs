use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, OscillatorNode, OscillatorType};

use web_sys::console;

use chip8_core::AudioPlayer;

const BEEP_FREQUENCY: f32 = 440.0;

#[wasm_bindgen]
pub struct Audio {
    ctx: AudioContext,
    oscillator: OscillatorNode,
}

#[wasm_bindgen]
impl Audio {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Self, JsValue> {
        // Audio
        let ctx = AudioContext::new()?;

        let oscillator = OscillatorNode::new(&ctx)?;
        oscillator.set_type(OscillatorType::Sine);
        oscillator
            .frequency()
            .set_value_at_time(BEEP_FREQUENCY, ctx.current_time())?;

        oscillator.connect_with_audio_node(&ctx.destination())?;

        Ok(Audio { ctx, oscillator })
    }

    // No biggie if either of these fail
    #[wasm_bindgen]
    pub fn play(&self) {
        console::log(&JsValue::from_str("Hello hey").into());
        self.oscillator.start().ok();
    }

    #[wasm_bindgen]
    pub fn pause(&self) {
        self.oscillator.stop().ok();
    }

    #[wasm_bindgen]
    pub fn resume(&self) {
        self.ctx.resume().ok();
    }
}

impl AudioPlayer for Audio {
    fn play(&self) {
        self.oscillator.start().ok();
    }

    fn pause(&self) {
        self.oscillator.stop().ok();
    }
}
