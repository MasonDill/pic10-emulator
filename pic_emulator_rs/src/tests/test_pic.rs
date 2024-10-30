#[cfg(test)]
mod test {
    // Import the module we want to test
    use crate::pic::{self, PIC10F200};
    use crate::data_memory::RegisterFile;
    use crate::program_memory::ProgramMemory;
    use crate::nbitnumber::u9;
    use crate::pic::PICInstruction;
    
    #[test]
    fn test_pic10f200() {
        let pic = PIC10F200 {
            data_memory: RegisterFile::new(),
            program_memory: ProgramMemory::new(),
            program_counter: u9::new(0),
            instruction_register: PICInstruction::new(),
            w_register,
            io_pins: todo!(),
        };

        let program = [0x000; 0x200];
        pic.program_chip(program);
        
        pic.power_on_initialize();
    }

    // program the PIC with a program
    fn test_program_chip(pic: &mut PIC10F200) {

        
    }
}