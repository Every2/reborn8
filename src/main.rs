use std::env;
use std::fs::File;
use std::io::Read;

use reborn8::chip8::Chip8;
use reborn8::sdl::{draw, process_input};
use reborn8::{SCALE, SCREEN_HEIGHT, SCREEN_WIDTH, TICKS_PER_FRAME};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: path/to/rom");
        return;
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsytem = sdl_context.video().unwrap();
    let window = video_subsytem
        .window(
            "Reborn8",
            (SCREEN_WIDTH as u32) * SCALE,
            (SCREEN_HEIGHT as u32) * SCALE,
        )
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut chip8 = Chip8::new();

    let mut rom = File::open(&args[1]).expect("Unable to open file");
    let mut buffer = Vec::new();

    rom.read_to_end(&mut buffer).unwrap();
    chip8.load_rom(&buffer);

    'gameloop: loop {
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
                    if let Some(k) = process_input(key) {
                        chip8.is_key_pressed(k, true);
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = process_input(key) {
                        chip8.is_key_pressed(k, false);
                    }
                }
                _ => (),
            }
        }

        for _ in 0..TICKS_PER_FRAME {
            chip8.clock();
        }

        chip8.tick();
        draw(&chip8, &mut canvas);
    }
}
