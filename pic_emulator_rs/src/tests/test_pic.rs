#[cfg(test)]
mod test {
    // Import the module we want to test
    use crate::pic::{self, PIC10F200, Programmable, PipelinedTuringMachine};
    use crate::data_memory::RegisterFile;
    use crate::program_memory::ProgramMemory;
    use crate::nbitnumber::{u9, u12};
    use crate::pic::PICInstruction;
    use crate::pic::PICInstructionMnemonic;

    #[test]
    fn test_pic10f200() {
        let mut pic = PIC10F200 {
            data_memory: RegisterFile::new(),
            program_memory: ProgramMemory::new(),
            program_counter: u9::new(0),
            instruction_register: PICInstruction::from_u12(u12::new(0x000)),
            w_register: 0,
            io_pins: [false; 3],
        };

        // Example program: Add 0x08 and 0x05, store in 0x00
        let instructions = [
            u12::new(0xC08), // MOVLW 0x08
            u12::new(0x005), // ADDWF 0x05, 0
            u12::new(0x000), // MOVWF 0x00
            u12::new(0x000), // NOP
        ];
        program_instructions(&mut pic, &instructions);
        
        pic.power_on_initialize();

        // cycle forever
        loop {
            pic.tick();
        }
    }

    // Helper to program a slice of instructions starting at address 0
    fn program_instructions(pic: &mut PIC10F200, instructions: &[u12]) {
        // Load the instructions into an empty program buffer
        let mut program_buffer = [u12::new(0x000); 0x200];
        let num_to_copy = std::cmp::min(instructions.len(), program_buffer.len());
        program_buffer[..num_to_copy].copy_from_slice(&instructions[..num_to_copy]);

        // Program the chip with the buffer
        pic.program_chip(program_buffer);
    }
}