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
use array2d::{Array2D, Error};
use concat_arrays::concat_arrays;
const START_PC: usize = 0x200;
const OPCODE_SIZE: usize = 2; // one opcode is 16 bits, that is 2 bytes
const NEXT_PC: usize = START_PC + OPCODE_SIZE;
const SKIPPED_PC: usize = START_PC + (2 * OPCODE_SIZE);
pub const CHIP_8_HEIGHT: usize = 32;
pub const CHIP_8_WIDTH: usize = 64;
#[derive(Debug)]
pub struct CpuError(String);
pub struct Cpu {
    ram: [u8; 4096],            // Four KB of memory
    v: [u8; 16],                // 16 8-bit general purpose registers
    i: usize,                   // index register - used for stepping through arrays
    stack: [u16; 16],           // stack - so far used for storing adresses
    sp: usize,                  // stack pointer - points to the top of the stack
    pc: u16,     // program counter - stores the location of the instruction to be executed
    dt: u8,      // delay timer - decrements every cycle if not zero
    st: u8,      // sound timer - decrements every cycle if not zero
    opcode: u16, // current opcode - might be not needed
    pub display: Array2D<bool>, // store screen pixels - black and white
    vf: bool,    // register that is used as a flag by some programs
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
            vf: false,
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
                self.v[x] = self.v[x] + nn;
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
            0x0000 => {
                //Handle CLS and RET
                match nn {
                    0xE0 => {
                        //CLS
                        // TODO: check if really the x-coordinate is lateral
                        self.display = Array2D::filled_with(false, CHIP_8_WIDTH, CHIP_8_HEIGHT); // set all pixels to black
                        Ok(())
                    }
                    0xEE => {
                        //RET
                        //TODO: Implement
                        Ok(())
                    }
                    _ => {
                        // Return error
                        Err(CpuError(format!("Unknown OpCode: {:#06X}", opcode)))
                    }
                }
            }

            _ => Err(CpuError(format!("Unknown OpCode: {:#06X}", opcode))),
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
        // Here is implemented the Fetch-Decode-Execute cycle
        // allows pc to be used as an index
        let pc: usize = self.pc as usize;
        // Fetch opcode
        let upper: u16 = self.ram[pc].into();
        let lower: u16 = self.ram[pc + 1].into();
        let opcode: u16 = (upper << 8) | lower; // merge two bytes
        self.run_opcode(opcode)?;
        Ok(())
        // TODO: update timers
    }
    pub fn load_rom(&mut self, rom: Vec<u8>) {
        for i in 0..rom.len() {
            self.ram[0x200 + i] = rom[i];
        }
    }
}
