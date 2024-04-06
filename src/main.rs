use std::env;
// Button import
use crate::piston::ButtonEvent;
// Imports for reading files
use std::fs::File;
use std::io;
use std::io::prelude::*;
// Import so I can play music
use music;
// Import so I can read the timestamp of the window
use crate::piston::GenericEvent;
// Imports for adding the window
extern crate piston; // What does this do?
use piston::{RenderEvent, WindowSettings};
extern crate glutin_window;
use glutin_window::GlutinWindow;
use piston::event_loop::{EventLoop, EventSettings, Events};
// Imports for adding the graphics
extern crate graphics;
extern crate opengl_graphics;
//Import for update handling
use crate::piston::UpdateEvent;

use array2d::{Array2D, Error};
use opengl_graphics::{GlGraphics, OpenGL};
mod cpu;
use cpu::Cpu;
mod fonts;
use cpu::{CHIP_8_HEIGHT, CHIP_8_WIDTH};
type Colour = [f32; 4];
const PIXEL_SIZE: f64 = 7.0;
const WHITE: Colour = [1.0; 4];
const BLACK: Colour = [0.0, 0.0, 0.0, 1.0];
const UPDATE_RATE: u64 = 500; // I need 500hz for the CPU
const FPS: u64 = 60; // How often I will refresh the display
fn read_rom(path: &str) -> Vec<u8> {
    let mut f = File::open(path).expect("Failed to open rom!");
    let mut buffer: Vec<u8> = vec![];
    f.read_to_end(&mut buffer).expect("Failed to read rom!");
    buffer
}
// Create an enum for sound?
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
enum Music {
    Piano, 
}
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
enum Sound {
    Ding, 
}

fn main() {
    let args: Vec<String> = env::args().collect();
    //let curr_rom: usize = 7;
    // Read the rom
    // TODO: Generate the romlist automatically
    // let rom_list: Vec<&str> = vec!["1-chip8-logo.ch8", "2-ibm-logo.ch8", "3-corax+.ch8", "4-flags.ch8", "5-quirks.ch8", "6-keypad.ch8", "7-beep.ch8", "breakout.ch8"];
    let rom: Vec<u8> = read_rom(&args[1]);
    let mut cpu = cpu::Cpu::new();
    cpu.load_rom(rom);
    // To skip the menu
    //cpu.ram[0x1FF] = 0x3;
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
    // for counting the frames
    let mut update_counter = 0;
    // Handle the music
    // TODO: Set only one channel
    music::start::<Music, Sound, _>(16, || {
        music::bind_sound_file(Sound::Ding, "./assets/beep_200ms.wav");
        music::set_volume(music::MAX_VOLUME);
        // music::play_sound(&Sound::Ding, music::Repeat::Times(1), music::MAX_VOLUME);
    
    // Probe the system for events
    let mut update: u64 = 0;
    while let Some(e) = events.next(&mut window) {
        if let Some(x) = e.update_args() { // Every update equals one cpu cycle
            cpu.emulate_cycle().unwrap();
            // n second check if self.beep = true if it is, play sound
            // the beep sound is 200ms
            /*
            update_counter += 1;
            if cpu.beep == true && update_counter >= 100 {
                 music::play_sound(&Sound::Ding, music::Repeat::Times(1), music::MAX_VOLUME);
                 update_counter = 0;
            }
            
            if cpu.beep == true {
                println!("Beep");
            }
            else {
                print!("{}[2J", 27 as char);
            }
            */
        }
        // Capture a keypress and send it to the CPU
        if let Some(b) = e.button_args() {
            // TODO: fix this naive implementation
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
    });
}
