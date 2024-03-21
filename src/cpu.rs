// My PC is little endian, the CHIP-8 architecture is big endian, that doesn't matter here?
// The stack pointer points to the topmost level of the stack
// meaning that it points to empty space. If stack[0], stack[1], stack[2] have values inside of
// them, then stack_pointer is 3 (it points to stack[3])
// For now I don't check for overflow in certain places, If the necessity occurs, I will
// do it
// The last register V[0xF] seems to be for signifying overflow
// 8xy5 is subtracting with wrap-around, I don't know if that's correct
// TODO: Delete all of those allow's and get rid of warnings
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
use crate::fonts::FONTSET;
use crate::UPDATE_RATE;
use array2d::{Array2D, Error};
use concat_arrays::concat_arrays;
const START_PC: usize = 0x200;
const OPCODE_SIZE: usize = 2; // one opcode is 16 bits, that is 2 bytes
const NEXT_PC: usize = START_PC + OPCODE_SIZE;
const SKIPPED_PC: usize = START_PC + (2 * OPCODE_SIZE);
pub const CHIP_8_HEIGHT: usize = 32;
pub const CHIP_8_WIDTH: usize = 64;
#[derive(PartialEq, Copy, Clone)]
enum KeyState {
    Up,
    Down,
}
#[derive(Debug)]
pub struct CpuError(String);
// TODO: make ram not pub
pub struct Cpu {
    pub ram: [u8; 4096],        // Four KB of memory
    v: [u8; 16],                // 16 8-bit general purpose registers
    i: usize,                   // index register - used for stepping through arrays
    stack: [u16; 16],           // stack - so far used for storing adresses
    sp: usize,                  // stack pointer - points to the top of the stack
    pc: u16,     // program counter - stores the location of the instruction to be executed
    dt: u8,      // delay timer - decrements every cycle if not zero
    st: u8,      // sound timer - decrements every cycle if not zero
    opcode: u16, // current opcode - might be not needed
    pub display: Array2D<bool>, // store screen pixels - black and white
    timer_counter: u16, // counter to update dt in the desired intervals
    keys: [KeyState; 16],
    halt: bool, // Field for the instruction Fx0A
    halt_idx: usize, // Field for the instruction Fx0A
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            ram: concat_arrays!(FONTSET, [0; 4016]), // add fonts, the rest is 0's
            v: [0; 16],                              // fill all registers with 0's
            i: 0,                                    // set index register to 0's
            stack: [0; 16],                          // fill the stack with 0's
            sp: 0,
            pc: START_PC as u16, // program counter starts at 0x200 (earlier is the interpreter code)
            dt: 0,               // not sure about this one
            st: 0,               // not sure about this one
            opcode: 0x000,       // set current opcode
            display: Array2D::filled_with(false, CHIP_8_WIDTH, CHIP_8_HEIGHT), // set all pixels to black
            timer_counter: 0,
            keys: [KeyState::Up; 16],
            halt: false, 
            halt_idx:  0, 
        }
    }
    fn run_opcode(&mut self, opcode: u16) -> Result<(), CpuError> {
        // Decode and Execute opcode
        let x: usize = ((opcode >> 8) & 0x0F).try_into().unwrap(); // index, so we cast
        let y: usize = ((opcode >> 4) & 0x00F).try_into().unwrap(); // index, so we cast
        let n: u8 = (opcode & 0x000F).try_into().unwrap(); // immediate - value hardcoded in the opcode
        let nn: u8 = (opcode & 0x00FF).try_into().unwrap();
        let nnn: u16 = opcode & 0x0FFF;
        let f: u16 = opcode & 0xF000; // The first nibble
        let vxlow: usize = (self.v[x] & 0x0F) as usize; // the lowest nibble of self.v[x]

        // Increment the PC after fetching the opcode and before decoding
        self.increment_pc();

        match f {
            0x1000 => {
                // 1nnn - JP addr
                self.pc = nnn;
                Ok(())
            }
            0x6000 => {
                // 6xnn - LD Vx, byte
                self.v[x] = nn;
                Ok(())
            }
            0x7000 => {
                // 7xnn - ADD Vx, byte
                self.v[x] = self.v[x].overflowing_add(nn).0;
                Ok(())
            }
            0xA000 => {
                // Annn - LD I, addr
                self.i = nnn.try_into().unwrap();
                Ok(())
            }
            0xD000 => {
                // Dxyn - DRW Vx, Vy, nibble
                // Ensure V[x], V[y] are in bounds
                let vx: usize = self.v[x] as usize % CHIP_8_WIDTH;
                let vy: usize = self.v[y] as usize % CHIP_8_HEIGHT;
                // Set VF to 0
                self.v[0xF] = 0;
                // the bounds of a range must have matching types
                for r in 0_usize..n.try_into().unwrap() {
                    // Find y index
                    let idx_y: usize = vy + r;
                    // If idx_y out of bounds, discard it
                    if idx_y >= CHIP_8_HEIGHT {
                        break;
                    }
                    let curr_byte: u8 = self.ram[self.i + r];
                    for z in 0..8_usize {
                        // Find x index
                        let idx_x: usize = vx + z;
                        // If idx_x out of bounds, discard it
                        if idx_x >= CHIP_8_WIDTH {
                            break;
                        }
                        if curr_byte & 1 << (7 - z) == 0 {
                            continue;
                        }
                        if self.display[(idx_x, idx_y)] == true {
                            self.v[0xF] = 1;
                        }
                        self.display[(idx_x, idx_y)] ^= true;
                    }
                }
                Ok(())
            }
            0x3000 => {
                // 3xkk - SE Vx, byte
                if self.v[x] == nn {
                    self.increment_pc();
                }
                Ok(())
            }
            0x4000 => {
                // 4xkk - SNE Vx, byte
                if self.v[x] != nn {
                    self.increment_pc();
                }
                Ok(())
            }

            0x8000 => {
                match n {
                    // 8xy4 - ADD Vx, Vy
                    0x4 => {
                        let (result, overflow) = self.v[x].overflowing_add(self.v[y]);
                        self.v[x] = result;
                        if overflow {
                            self.v[0xF] = 1;
                        } else {
                            self.v[0xF] = 0;
                        }
                        Ok(())
                    }
                    0x5 => {
                        // 8xy5 - SUB Vx, Vy
                        // What kind of subtraction are we after? I think overflowing
                        let (result, underflow) = self.v[x].overflowing_sub(self.v[y]);
                        self.v[x] = result;
                        if underflow {
                            // Cowgod says it should be >
                            self.v[0xF] = 0;
                        } else {
                            self.v[0xF] = 1;
                        }
                        Ok(())
                    }
                    0x0 => {
                        // 8xy0 - LD Vx, Vy
                        self.v[x] = self.v[y];
                        Ok(())
                    }
                    0x7 => {
                        // 8xy7 - SUB Vx, Vy
                        // What kind of subtraction are we after? I think overflowing
                        let (result, underflow) = self.v[y].overflowing_sub(self.v[x]);
                        self.v[x] = result;
                        if underflow {
                            // Cowgod says it should be >
                            self.v[0xF] = 0;
                        } else {
                            self.v[0xF] = 1;
                        }
                        Ok(())
                    }
                    0x1 => {
                        // 8xy1 - OR Vx, Vy
                        self.v[x] = self.v[x] | self.v[y];
                        Ok(())
                    }
                    0x6 => {
                        //  8xy6 - SHR Vx {, Vy}
                        let mut temp = 0;
                        if self.v[x] & 1 == 1 {
                            temp = 1;
                        }
                        self.v[x] >>= 1;
                        self.v[0xF] = temp;
                        Ok(())
                    }
                    0x2 => {
                        // 8xy2 - AND Vx, Vy
                        self.v[x] = self.v[x] & self.v[y];
                        Ok(())
                    }
                    0xE => {
                        // 8xyE - SHL Vx {, Vy}
                        let mut temp = 0;
                        if self.v[x] >> 7 & 1 == 1 {
                            temp = 1;
                        }
                        self.v[x] <<= 1;
                        self.v[0xF] = temp;
                        Ok(())
                    }
                    0x3 => {
                        // 8xy3 - XOR Vx, Vy
                        self.v[x] ^= self.v[y];
                        Ok(())
                    }

                    _ => {
                        // Return error
                        Err(CpuError(format!("Unknown OpCode: {:#06X}", opcode)))
                    }
                }
            }
            0xF000 => {
                match nn {
                    // Fx0A - LD Vx, K
                    0x0A => {
                        self.halt_idx = x;
                        self.halt = true;
                        Ok(())
                    }
                    // Fx07 - LD Vx, DT
                    0x07 => {
                        self.v[x] = self.dt;
                        Ok(())
                    }
                    // Fx15 - LD DT, Vx
                    0x15 => {
                        self.dt = self.v[x];
                        Ok(())
                    }
                    // Fx65 - LD Vx, [I]
                    0x65 => {
                        for idx in 0..x + 1 {
                            self.v[idx] = self.ram[self.i + idx];
                        }
                        Ok(())
                    }
                    // Fx55 - LD [I], Vx
                    0x55 => {
                        for idx in 0..x + 1 {
                            self.ram[self.i + idx] = self.v[idx];
                        }
                        Ok(())
                    }
                    // Fx33 - LD B, Vx
                    0x33 => {
                        let hundreds: u8 = self.v[x] / 100;
                        let tens: u8 = (self.v[x] % 100) / 10;
                        let ones: u8 = self.v[x] % 10;
                        self.ram[self.i] = hundreds;
                        self.ram[self.i + 1] = tens;
                        self.ram[self.i + 2] = ones;
                        Ok(())
                    }
                    0x1E => {
                        // Fx1E - ADD I, Vx
                        self.i = self.i.saturating_add(self.v[x].into());
                        Ok(())
                    }
                    _ => {
                        // Return error
                        Err(CpuError(format!("Unknown OpCode: {:#06X}", opcode)))
                    }
                }
            }
            0x2000 => {
                // 2nnn - CALL addr
                self.stack[self.sp] = self.pc;
                self.sp += 1;
                self.pc = nnn;
                Ok(())
            }

            0x0000 => {
                //Handle CLS and RET
                match nn {
                    0xE0 => {
                        //CLS
                        self.display = Array2D::filled_with(false, CHIP_8_WIDTH, CHIP_8_HEIGHT); // set all pixels to black
                        Ok(())
                    }
                    0xEE => {
                        //RET
                        // Subtract from the sp
                        self.sp -= 1;
                        // set the pc to the address at the top of the stack
                        self.pc = self.stack[self.sp];
                        Ok(())
                    }
                    _ => {
                        // Return error
                        Err(CpuError(format!("Unknown OpCode: {:#06X}", opcode)))
                    }
                }
            }
            0x5000 => {
                if self.v[x] == self.v[y] {
                    self.increment_pc();
                }
                Ok(())
            }
            0x9000 => {
                if self.v[x] != self.v[y] {
                    self.increment_pc();
                }
                Ok(())
            }
            0xE000 => match nn {
                0x9E => {
                    // Ex9E - SKP Vx
                    if self.keys[vxlow] == KeyState::Down {
                        self.skip_ins();
                    }
                    Ok(())
                }
                0xA1 => {
                    // ExA1 - SKNP Vx
                    if self.keys[vxlow] == KeyState::Up {
                        self.skip_ins();
                    }
                    Ok(())
                }
                _ => Err(CpuError(format!("Unknown OpCode: {:#06X}", opcode))),
            },
            _ => Err(CpuError(format!("Unknown OpCode: {:#06X}", opcode))),
        }
    }
    fn translate(key: i32) -> Option<u8> {
        // Translate the scancodes to 0-F values
        match key {
            2 => Some(0x1),
            3 => Some(0x2),
            4 => Some(0x3),
            5 => Some(0xC),
            16 => Some(0x4),
            17 => Some(0x5),
            18 => Some(0x6),
            19 => Some(0xD),
            30 => Some(0x7),
            31 => Some(0x8),
            32 => Some(0x9),
            33 => Some(0xE),
            44 => Some(0xA),
            45 => Some(0x0),
            46 => Some(0xB),
            47 => Some(0xF),
            _ => None,
        }
    }

    pub fn key_pressed(&mut self, key: i32) {
        if let Some(k) = Self::translate(key) {
            eprintln!("{:#} is Down", k);
            self.keys[k as usize] = KeyState::Down;
            // If we're halting (Fx0A has been called),
            // save the key value to self.v[x] negate self.halt
            if self.halt {
                self.v[self.halt_idx] = k;
            }
        }
    }
    pub fn key_released(&mut self, key: i32) {
        if let Some(k) = Self::translate(key) {
            eprintln!("{:#} is Up", k);
            self.keys[k as usize] = KeyState::Up;
            // When pressed key is released, stop halting
            if self.halt {
                self.halt = false;
            }
        }
    }
    fn increment_pc(&mut self) {
        // Increments the PC
        self.pc += OPCODE_SIZE as u16;
    }

    fn skip_ins(&mut self) {
        // Skips an instruction
        self.pc += OPCODE_SIZE as u16 * 2;
    }
    pub fn emulate_cycle(&mut self) -> Result<(), CpuError> {
        // update timers 60 times per second
        self.timer_counter += 1;
        if self.timer_counter as u64 >= UPDATE_RATE / 60 {
            if self.dt > 0 {
                self.dt -= 1;
            }
            if self.st > 0 {
                self.st -= 1;
            }
            self.timer_counter = 0;
        }
        // For the purpose of the Fx0A instruction
        // all execution stops, but the timers are still ticking down
        if self.halt {
            return Ok(());
        }
        // Here is implemented the Fetch-Decode-Execute cycle
        // allows pc to be used as an index
        let pc: usize = self.pc as usize;
        // Fetch opcode
        let upper: u16 = self.ram[pc].into();
        let lower: u16 = self.ram[pc + 1].into();
        let opcode: u16 = (upper << 8) | lower; // merge two bytes
        self.run_opcode(opcode)?;
        Ok(())
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        for i in 0..rom.len() {
            self.ram[0x200 + i] = rom[i];
        }
    }
}
