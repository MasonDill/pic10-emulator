#[cfg(test)]
mod test {
    // Import the module we want to test
    use crate::pic::PIC10F200;
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
            w_register: 0x00,
        };

        pic.power_on_initialize();
        
        // let clock:bool= false;

        while true() {
            pic.tick();
        }
    }
}