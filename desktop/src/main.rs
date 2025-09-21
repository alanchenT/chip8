mod audio;
use audio::Audio;

use chip8_core::*;

use sdl2::{
    self, event::Event, keyboard::Keycode, pixels::Color, rect::Rect, render::Canvas, video::Window,
};

use std::{
    env,
    fs::File,
    io::Read,
    process,
    time::{Duration, Instant},
};

const SCALE: u32 = 16;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE; // SDL2 requires u32
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

const TICKS_PER_FRAME: usize = 10;

const TICK_RATE: Duration = Duration::from_millis(16);

fn draw_screen(emulator: &Emulator, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::BLACK);
    canvas.clear(); // Black out canvas

    let emulator_screen = emulator.get_display();

    canvas.set_draw_color(Color::WHITE);

    for (idx, pixel_bit) in emulator_screen.iter().enumerate() {
        // Skip if pixel is black
        if !(*pixel_bit) {
            continue;
        }

        // Get 2D position from index
        let x_coord = (idx % SCREEN_WIDTH) as u32;
        let y_coord = (idx / SCREEN_WIDTH) as u32;

        // Draw white pixel
        let rect = Rect::new(
            (x_coord * SCALE) as i32,
            (y_coord * SCALE) as i32,
            SCALE,
            SCALE,
        );

        canvas.fill_rect(rect).unwrap();
    }

    canvas.present();
}

/*
    Keyboard                    Chip-8
    +---+---+---+---+           +---+---+---+---+
    | 1 | 2 | 3 | 4 |           | 1 | 2 | 3 | C |
    +---+---+---+---+           +---+---+---+---+
    | Q | W | E | R |           | 4 | 5 | 6 | D |
    +---+---+---+---+     =>    +---+---+---+---+
    | A | S | D | F |           | 7 | 8 | 9 | E |
    +---+---+---+---+           +---+---+---+---+
    | Z | X | C | V |           | A | 0 | B | F |
    +---+---+---+---+           +---+---+---+---+
*/
fn keycode_to_button(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: cargo run path/to/game");
        process::exit(1);
    }

    // Justification for .unwrap(): These ought to fail only in the most exceptional cases. Even then, it's not my fault. It's SDL's.
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("CHIP-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut chip8 = Emulator::new();

    // Load file into emulator
    let mut rom = File::open(&args[1]).expect("Unable to open file");
    let mut buffer = Vec::new();

    rom.read_to_end(&mut buffer).unwrap();
    chip8.load(&buffer);

    let audio_player = Audio::new().unwrap();
    chip8.set_audio_player(Some(Box::new(audio_player)));

    let mut next_tick_time = Instant::now(); // Loop runs at 60 Hz
    'gameloop: loop {
        if Instant::now() >= next_tick_time {
            next_tick_time += TICK_RATE;
        } else {
            continue;
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'gameloop;
                }

                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(input) = keycode_to_button(key) {
                        chip8.keypress(input, true);
                    }
                }

                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(input) = keycode_to_button(key) {
                        chip8.keypress(input, false);
                    }
                }

                _ => (),
            }
        }

        // Tick 10 times before drawing
        for _ in 0..TICKS_PER_FRAME {
            chip8.tick();
        }

        chip8.tick_timers();
        draw_screen(&chip8, &mut canvas);
    }
}
