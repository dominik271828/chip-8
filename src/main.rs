// My own modules
mod cpu;
mod fonts;
use cpu::{Cpu, CHIP_8_HEIGHT, CHIP_8_WIDTH};

extern crate glutin_window;
extern crate piston;
extern crate graphics;
extern crate opengl_graphics;
// Import for reading the command line argument
use std::env;
// Imports for reading files
use std::fs::File;
use std::io::prelude::*;
use piston::{RenderEvent, WindowSettings, ButtonEvent, UpdateEvent};
use glutin_window::GlutinWindow;
use piston::event_loop::{EventSettings, Events};
use opengl_graphics::{GlGraphics, OpenGL};


type Colour = [f32; 4];
const PIXEL_SIZE: f64 = 7.0;
const WHITE: Colour = [1.0; 4];
const BLACK: Colour = [0.0, 0.0, 0.0, 1.0];
const UPDATE_RATE: u64 = 500; // I need 500hz for the CPU
const FPS: u64 = 60; 
fn read_rom(path: &str) -> Vec<u8> {
    let mut f = File::open(path).expect("Failed to open rom!");
    let mut buffer: Vec<u8> = vec![];
    f.read_to_end(&mut buffer).expect("Failed to read rom!");
    buffer
}
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run /path/to/rom");
        return ();
    }
    let rom: Vec<u8> = read_rom(&args[1]);
    let mut cpu = Cpu::new();
    cpu.load_rom(rom);
    start_game(cpu);
}
fn start_game(mut cpu: cpu::Cpu) {
    // Initialize settings
    let w_width: f64 = CHIP_8_WIDTH as f64 * PIXEL_SIZE;
    let w_height: f64 = CHIP_8_HEIGHT as f64 * PIXEL_SIZE;
    let settings = WindowSettings::new("Chip 8 Emulator", (w_width, w_height)).exit_on_esc(true);
    // Create window
    let mut window: GlutinWindow = settings.build().expect("Could not create window");
    // Create the event
    let mut event_settings = EventSettings::new();
    event_settings.ups = UPDATE_RATE; // 500
    event_settings.max_fps = FPS; // 60
    let mut events = Events::new(event_settings);
    // Initialize OpenGL
    let opengl = OpenGL::V3_2;
    let mut gl = GlGraphics::new(opengl);
    while let Some(e) = events.next(&mut window) {
        if let Some(_) = e.update_args() { // Every update equals one cpu cycle
            cpu.emulate_cycle().unwrap();
        }
        // Capture a keypress and send it to the CPU
        if let Some(b) = e.button_args() {
            if b.state == piston::ButtonState::Release {
                if let Some(scancode) = b.scancode {
                    cpu.key_released(scancode);
                }
            }
            if b.state == piston::ButtonState::Press {
                if let Some(scancode) = b.scancode {
                    cpu.key_pressed(scancode);
                }
            }
        }
        if let Some(r) = e.render_args() {
            gl.draw(r.viewport(), |c, g| {
                graphics::clear(BLACK, g);
                // Draw all the pixels
                for x in 0..CHIP_8_WIDTH {
                    for y in 0..CHIP_8_HEIGHT {
                        let pos: [f64; 4] = [
                            PIXEL_SIZE * x as f64,
                            PIXEL_SIZE * y as f64,
                            PIXEL_SIZE * (x + 1) as f64,
                            PIXEL_SIZE * (y + 1) as f64,
                        ];
                        let colour;
                        if cpu.display[(x, y)] {
                            colour = BLACK;
                        } else {
                            colour = WHITE;
                        }
                        graphics::Rectangle::new(colour).draw(pos, &c.draw_state, c.transform, g);
                    }
                }
            });
        }
    }
}
