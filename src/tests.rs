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
