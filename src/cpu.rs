// My PC is little endian, the CHIP-8 architecture is big endian, that doesn't matter here?
// The stack pointer points to the topmost level of the stack
// meaning that it points to empty space. If stack[0], stack[1], stack[2] have values inside of
// them, then stack_pointer is 3 (it points to stack[3])
// For now I don't check for overflow in certain places, If the necessity occurs, I will
// do it
// The last register V[0xF] seems to be for signifying overflow
// 8xy5 is subtracting with wrap-around, I don't know if that's correct
use crate::fonts::FONTSET;
use array2d::{Array2D, Error};
use concat_arrays::concat_arrays;
const START_PC: usize = 0x200;
const OPCODE_SIZE: usize = 2; // one opcode is 16 bits, that is 2 bytes
const NEXT_PC: usize = START_PC + OPCODE_SIZE;
const SKIPPED_PC: usize = START_PC + (2 * OPCODE_SIZE);
const CHIP_8_HEIGHT: usize = 32;
const CHIP_8_WIDTH: usize = 64;
#[derive(Debug)]
struct CpuError(String);
pub struct Cpu {
    ram: [u8; 4096],        // Four KB of memory
    v: [u8; 16],            // 16 8-bit general purpose registers
    i: u16,                 // index register - used for stepping through arrays
    stack: [u16; 16],       // stack - so far used for storing adresses
    sp: usize,              // stack pointer - points to the top of the stack
    pc: u16,     // program counter - stores the location of the instruction to be executed
    dt: u8,      // delay timer - decrements every cycle if not zero
    st: u8,      // sound timer - decrements every cycle if not zero
    opcode: u16, // current opcode - might be not needed
    display: Array2D<bool>, // store screen pixels - black and white
}

impl Cpu {
    fn new() -> Self {
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
            display: Array2D::filled_with(false, CHIP_8_HEIGHT, CHIP_8_WIDTH), // set all pixels to black
        }
    }
    fn run_opcode(&mut self, opcode: u16) -> Result<(), CpuError> {
        // Decode and Execute opcode
        match opcode & 0xF000 {
            // Look at first nibble to determine opcode
            0xA000 => {
                // Annn - LD I, addr
                self.i = opcode & 0x0FFF;
                self.pc += 2;
                Ok(())
            }
            0x1000 => {
                // 1nnn - JP addr
                self.pc = opcode & 0x0FFF;
                Ok(())
            }
            0x2000 => {
                // 2nnn - CALL addr
                self.sp += 1;
                self.stack[self.sp - 1] = self.pc;
                self.pc = opcode & 0x0FFF;
                Ok(())
            }
            0x3000 => {
                // 3xkk - SE Vx, byte
                let idx: usize = ((opcode >> 8) & 0x0F) as usize;
                if idx > 16 {
                    return Err(CpuError(format!(
                        "Attempt to access a 'V[{}]' register which doesn't exist",
                        idx
                    )));
                }
                let val: u16 = opcode & 0x00FF;
                if u16::from(self.v[idx]) == val {
                    self.skip_ins();
                }
                Ok(())
            }
            0x4000 => {
                // 4xkk - SNE Vx, byte
                let idx: usize = ((opcode >> 8) & 0x0F) as usize;
                if idx > 16 {
                    return Err(CpuError(format!(
                        "Attempt to access a 'V[{}]' register which doesn't exist",
                        idx
                    )));
                }
                let val: u16 = opcode & 0x00FF;
                if u16::from(self.v[idx]) != val {
                    self.skip_ins();
                }
                Ok(())
            }
            0x5000 => {
                // 5xy0 - SE Vx, Vy
                let x: usize = ((opcode >> 8) & 0x0F) as usize;
                let y: usize = ((opcode >> 4) & 0x00F) as usize;
                if self.v[x] == self.v[y] {
                    self.skip_ins();
                }
                Ok(())
            }
            0x6000 => {
                // 6xkk - LD Vx, byte
                let x: usize = ((opcode >> 8) & 0x0F) as usize;
                let kk: u8 = (opcode & 0x00FF) as u8;
                println!("{x}, {kk}");
                self.v[x] = kk;
                Ok(())
            }
            0x7000 => {
                // 7xkk - ADD Vx, byte
                let x: usize = ((opcode >> 8) & 0x0F) as usize;
                let kk: u8 = (opcode & 0x00FF) as u8;
                let result = self.v[x] + kk;
                self.v[x] = result;
                Ok(())
            }
            0x8000 => {
                // Check last nibble to determine instruction
                let x: usize = ((opcode >> 8) & 0x0F) as usize;
                let y: usize = ((opcode >> 4) & 0x00F) as usize;
                match opcode & 0x000F {
                    0x0000 => {
                        // 8xy0 - LD Vx, Vy
                        self.v[x] = self.v[y];
                        Ok(())
                    }
                    0x0001 => {
                        // 8xy1 - OR Vx, Vy
                        let result: u8 = self.v[x] | self.v[y];
                        self.v[x] = result;
                        Ok(())
                    }
                    0x0002 => {
                        // 8xy2 - AND Vx, Vy
                        let result: u8 = self.v[x] & self.v[y];
                        self.v[x] = result;
                        Ok(())
                    }
                    0x0003 => {
                        // 8xy3 - XOR Vx, Vy
                        let result: u8 = self.v[x] ^ self.v[y];
                        self.v[x] = result;
                        Ok(())
                    }
                    0x0004 => {
                        // 8xy4 - ADD Vx, Vy
                        let (result, overflow) = self.v[x].overflowing_add(self.v[y]);
                        if overflow {
                            self.v[0xF] = 1;
                        }
                        else {
                            self.v[0xF] = 0;
                        }
                        self.v[x] = result;
                        Ok(())
                    }
                    0x0005 => {
                        // 8xy5 - SUB Vx, Vy
                        let (result, overflow) = self.v[x].overflowing_sub(self.v[y]);
                        if !overflow {
                            self.v[0xF] = 1;
                        }
                        else {
                            self.v[0xF] = 0;
                        }
                        self.v[x] = result;
                        Ok(())
                    }
                    0x0006 => { 
                        // 8xy6 - SHR Vx {, Vy}
                        if self.v[x] & 0x0001 == 1 {
                            self.v[0xF] = 1;
                        }
                        else {
                            self.v[0xF] = 0;
                        }
                        self.v[x] = self.v[x] >> 1;
                        Ok(())
                    }
                    _ => Err(CpuError(format!("Unknown OpCode: {:#06X}", opcode))),
                }
            }

            0x0000 => {
                // Check nibbles beside the first to decode opcode
                match opcode & 0x000F {
                    0x0000 => {
                        // 00E0 - CLS
                        self.display = Array2D::filled_with(false, CHIP_8_HEIGHT, CHIP_8_WIDTH);
                        self.pc += OPCODE_SIZE as u16; // Increment program counter, so that it points to the next OPCODE
                        Ok(())
                    }
                    0x000E => {
                        // 00EE - RET
                        self.pc = self.stack[self.sp - 1];
                        self.sp -= 1;
                        Ok(())
                    }
                    _ => Err(CpuError(format!("Unknown OpCode: {:#06X}", opcode))),
                }
            }
            _ => Err(CpuError(format!("Unknown OpCode: {:#06X}", opcode))),
        }
    }

    fn skip_ins(&mut self) {
        // Skips an instruction
        self.pc += OPCODE_SIZE as u16 * 2;
    }
    fn emulate_cycle(&mut self) -> Result<(), CpuError> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    fn init_cpu() -> Cpu {
        let mut cpu = Cpu::new();
        cpu.v = [0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7];
        cpu
    }
    #[test]
    fn test_initial_state() {
        let cpu = Cpu::new();
        assert_eq!(cpu.pc, 0x200);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.stack, [0; 16]);
        // First char in font: 0
        assert_eq!(cpu.ram[0..5], [0xF0, 0x90, 0x90, 0x90, 0xF0]);
        // Last char in font: F
        assert_eq!(
            cpu.ram[FONTSET.len() - 5..FONTSET.len()],
            [0xF0, 0x80, 0xF0, 0x80, 0x80]
        );
    }
    #[test]
    fn test_opcode_annn() {
        let mut cpu = Cpu::new();
        // Add opcode 0xA2F0 to ram
        cpu.ram[0x200] = 0xA2;
        cpu.ram[0x201] = 0xF0;
        let initial_pc = cpu.pc;
        // Check if the operation was successful
        assert!(cpu.emulate_cycle().is_ok());
        // Check if the 'I' register has a correct value
        assert_eq!(cpu.i, 0x02F0);
        // Check if the program counter was properly incremented
        assert_eq!(cpu.pc, initial_pc + 2);
    }
    #[test]
    fn test_opcode_annn1() {
        let mut cpu = init_cpu();
        cpu.run_opcode(0xA2F0).unwrap();
        assert_eq!(cpu.i, 0x02F0);
        assert_eq!(cpu.pc, NEXT_PC as u16);
    }

    // CLS
    #[test]
    fn test_op_00e0() {
        let mut cpu = init_cpu();
        cpu.run_opcode(0x00e0).unwrap();

        for x in 0..CHIP_8_WIDTH {
            for y in 0..CHIP_8_HEIGHT {
                assert_eq!(cpu.display[(y, x)], false);
            }
        }
        assert_eq!(cpu.pc, NEXT_PC as u16);
    }
    // RET
    #[test]
    fn test_op_00ee() {
        let mut cpu = init_cpu();
        cpu.sp = 6;
        cpu.stack[5] = 0xaaaa;
        cpu.run_opcode(0x00ee).unwrap();
        assert_eq!(cpu.sp, 5);
        assert_eq!(cpu.pc, 0xaaaa);
    }
    // JP addr
    #[test]
    fn test_op_1nnn() {
        let mut cpu = init_cpu();
        cpu.run_opcode(0x1abc).unwrap();
        assert_eq!(cpu.pc, 0x0abc);
    }
    // CALL addr
    #[test]
    fn test_op_2nnn() {
        let mut cpu = init_cpu();
        cpu.run_opcode(0x2abc).unwrap();
        assert_eq!(cpu.sp, 1);
        assert_eq!(cpu.stack[0], START_PC as u16);
        assert_eq!(cpu.pc, 0x0abc);
    }
    // SE Vx, byte
    #[test]
    fn test_op_3xkk_with_skip() {
        let mut cpu = init_cpu();
        cpu.v[5] = 0x00ab;
        cpu.run_opcode(0x35ab).unwrap();
        assert_eq!(cpu.pc, SKIPPED_PC as u16);
    }
    #[test]
    fn test_op_3xkk_without_skip() {
        let mut cpu = init_cpu();
        cpu.v[5] = 0x00ab;
        cpu.run_opcode(0x35ac).unwrap();
        assert_eq!(cpu.pc, START_PC as u16);
    }
    // 4xkk - SNE Vx, byte
    #[test]
    fn test_op_4xkk_without_skip() {
        let mut cpu = init_cpu();
        cpu.v[10] = 0x00cd;
        cpu.run_opcode(0x4acd).unwrap();
        assert_eq!(cpu.pc, START_PC as u16);
    }
    #[test]
    fn test_op_4xkk_with_skip() {
        let mut cpu = init_cpu();
        cpu.v[10] = 0x00cd;
        cpu.run_opcode(0x4aca).unwrap();
        assert_eq!(cpu.pc, SKIPPED_PC as u16);
    }
    // SE Vx, Vy
    #[test]
    fn test_op_5xy0_without_skip() {
        let mut cpu = init_cpu();
        cpu.v[5] = 0x10;
        cpu.v[6] = 0x11;
        cpu.run_opcode(0x5560).unwrap();
        assert_eq!(cpu.pc, START_PC as u16);
    }
    #[test]
    fn test_op_5xy0_with_skip() {
        let mut cpu = init_cpu();
        cpu.v[5] = 0x10;
        cpu.v[6] = 0x10;
        cpu.run_opcode(0x5560).unwrap();
        assert_eq!(cpu.pc, SKIPPED_PC as u16);
    }
    // 6xkk - LD Vx, byte
    #[test]
    fn test_op_6xkk() {
        let mut cpu = init_cpu();
        cpu.run_opcode(0x6312).unwrap();
        assert_eq!(cpu.v[3], 0x12);
    }
    // 7xkk - ADD Vx, byte
    #[test]
    fn test_op_7xkk_without_overflow() {
        let mut cpu = init_cpu();
        cpu.v[1] = 0x11;
        cpu.run_opcode(0x7101).unwrap();
        assert_eq!(cpu.v[1], 0x12);
    }
    // 8xy0 - LD Vx, Vy
    #[test]
    fn test_op_8xy0() {
        let mut cpu = init_cpu();
        cpu.v[1] = 0xAA;
        cpu.run_opcode(0x8210).unwrap();
        assert_eq!(cpu.v[2], 0xAA);
    }
    //8xy1 - OR Vx, Vy
    #[test]
    fn test_op_8xy1() {
        let mut cpu = init_cpu();
        cpu.v[1] = 0xA1;
        cpu.v[2] = 0x1A;
        cpu.run_opcode(0x8211).unwrap();
        assert_eq!(cpu.v[2], 0xBB);
    }
    // 8xy2 - AND Vx, Vy
    #[test]
    fn test_op_8xy2() {
        let mut cpu = init_cpu();
        cpu.v[1] = 0x0B;
        cpu.v[2] = 0xAF;
        cpu.run_opcode(0x8212).unwrap();
        assert_eq!(cpu.v[2], 0x0B);
    }
    // 8xy3 - XOR Vx, Vy
    #[test]
    fn test_op_8xy3() {
        let mut cpu = init_cpu();
        cpu.v[1] = 0x0B;
        cpu.v[2] = 0xAF;
        cpu.run_opcode(0x8213).unwrap();
        assert_eq!(cpu.v[2], 0xA4);
    }
    // 8xy4 - ADD Vx, Vy
    #[test]
    fn test_op_8xy4_without_carry() {
        let mut cpu = init_cpu();
        cpu.v[1] = 0x0B;
        cpu.v[2] = 0xAF;
        cpu.run_opcode(0x8214).unwrap();
        assert_eq!(cpu.v[2], 0xBA);
        assert_eq!(cpu.v[0xF], 0);
    }
    #[test]
    fn test_op_8xy4_with_carry() {
        let mut cpu = init_cpu();
        cpu.v[1] = 0xFF;
        cpu.v[2] = 0xAF;
        cpu.run_opcode(0x8214).unwrap();
        assert_eq!(cpu.v[2], 0xAE);
        assert_eq!(cpu.v[0xF], 1);
    }
    // 8xy5 - SUB Vx, Vy
    #[test]
    fn test_op_8xy5_without_underflow() {
        let mut cpu = init_cpu();
        cpu.v[1] = 0xFF;
        cpu.v[2] = 0xAF;
        cpu.run_opcode(0x8125).unwrap();
        assert_eq!(cpu.v[1], 0x50);
        assert_eq!(cpu.v[0xF], 1);
    }
    #[test]
    fn test_op_8xy5_with_underflow() {
        let mut cpu = init_cpu();
        cpu.v[1] = 0x00;
        cpu.v[2] = 0x01;
        cpu.run_opcode(0x8125).unwrap();
        assert_eq!(cpu.v[1], 0xFF);
        assert_eq!(cpu.v[0xF], 0);
    }
    // 8xy6 - SHR Vx {, Vy}
    #[test]
    fn test_op_8xy6_with_lsb_one() {
        let mut cpu = init_cpu();
        cpu.v[1] = 0xAF;
        cpu.run_opcode(0x8126).unwrap();
        assert_eq!(cpu.v[1], 0x57);
        assert_eq!(cpu.v[0xF], 1);
    }
    #[test]
    fn test_op_8xy6_with_lsb_zero() {
        let mut cpu = init_cpu();
        cpu.v[1] = 0xAE;
        cpu.run_opcode(0x8126).unwrap();
        assert_eq!(cpu.v[1], 0x57);
        assert_eq!(cpu.v[0xF], 0);
    }
    

}
