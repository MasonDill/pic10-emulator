use crate::{data_memory::{RegisterFile, SpecialPurposeRegisters, STATUS_POR_VALUE, FSR_POR_VALUE, OSCCAL_POR_VALUE, CMCON0_POR_VALUE, TRIS_POR_VALUE, OPTION_POR_VALUE},
logic::*, program_memory::{ProgramMemory, RESET_VECTOR, PC_POR_MOVLW_OSCCAL_ADDRESS}};
use common::{instructions::{PICInstructionMnemonic, PICInstructionType, PICInstruction}, nbitnumber::{NBitNumber, u9, u12, u5, NumberOperations}};

// Define the type alias for the instruction executor function pointer
// Moved before the trait definition to be in scope.
type InstructionExecutor = fn(&mut PIC10F200);

#[derive(Clone)]

//Highest level wrapper of the MCU
pub struct PIC10F200 {
    pub data_memory : RegisterFile,
    pub program_memory : ProgramMemory,
    pub program_counter : u9,
    pub instruction_register : PICInstruction,

    // these registers are not part of the data memory file register (not addressable)
    pub w_register : u8,
    pub io_pins : [bool; 3]
}

pub enum PIC10F2Types {
    PIC10F200,
    PIC10F202,
    PIC10F204,
    PIC10F206,
}

pub trait Programmable {
    fn program_chip(&mut self, new_program: [u12; 0x200]);
}
impl Programmable for PIC10F200 {
    fn program_chip(&mut self, new_program: [u12; 0x200]) {
        self.program_memory.flash(new_program);
        self.data_memory.flash();
    }
}

// TODO: make tick() the only public method
pub trait PipelinedTuringMachine {
    fn power_on_initialize(&mut self);
    fn fetch(&mut self);
    fn execute(&mut self);
    fn tick(&mut self);
    fn decode_mnemonic(instruction : PICInstructionMnemonic) -> InstructionExecutor;
}
impl PipelinedTuringMachine for PIC10F200 {
    fn power_on_initialize(&mut self) {
        // data sheet page 36, note 1. that the PC will first point to address 0xFF which will be loaded with the instruction MOVLW XX, where XX is the oscillator calibration value
        // this will load the W register with the oscillator calibration value
        // then the PC will be incremented to point to address 0x00, which will contain the first instruction to execute
        // also see data sheet page 11 note 1. -> load 0xFF (PC starts at **0xFF**, then moves to 0x00) with the MOVLW OSCCAL instruction
        // (should this be the duty of the programmer or the hardware? datasheet is not clear, just says 'don't overwrite it')
        let MOVLW_OSCCAL_INSTRUCTION: NBitNumber<12> = u12::new(0xC00 | OSCCAL_POR_VALUE as u16);
        self.program_memory.write(PC_POR_MOVLW_OSCCAL_ADDRESS, MOVLW_OSCCAL_INSTRUCTION);
        
        // Data sheet page 14 Table 4-1 details power on reset values for special function registers
        // INDF POR is undefined at power on
        // TMR0 POR is undefined at power on
        self.data_memory.write(u5::new(SpecialPurposeRegisters::PCL as u16),    PC_POR_MOVLW_OSCCAL_ADDRESS.as_u16() as u8); // data sheet page 14 Table 4-1
        self.data_memory.write(u5::new(SpecialPurposeRegisters::STATUS as u16), STATUS_POR_VALUE);
        self.data_memory.write(u5::new(SpecialPurposeRegisters::FSR as u16),    FSR_POR_VALUE);
        self.data_memory.write(u5::new(SpecialPurposeRegisters::OSCCAL as u16), OSCCAL_POR_VALUE);
        // GPIO POR is undefined at power on
        self.data_memory.write(u5::new(SpecialPurposeRegisters::CMCON0 as u16), CMCON0_POR_VALUE);
        self.data_memory.tris_register = TRIS_POR_VALUE; // not addressable
        self.data_memory.option_register = OPTION_POR_VALUE; // not addressable
    }

    fn tick(&mut self) {
        // see data sheet page 10 section 3.2 for pipeline details
        self.fetch();        
        self.execute(); 
    }

    fn fetch(&mut self) {
        let PCL = self.data_memory.read(u5::new(SpecialPurposeRegisters::PCL as u16));
        self.program_counter = u9::new(PCL as u16); // writes to PCL will also write to the program counter
        self.program_counter = self.program_counter + u9::new(1); // Q1, see data sheet page 10 section 3.2
        self.program_counter = self.program_counter & u9::new(0x0FF); // always clear the 9th bit, see data sheet page 18 section 4.7

        self.instruction_register = PICInstruction::from_u12(self.program_memory.fetch(self.program_counter)); // will decode the instruction
    }

    fn execute(&mut self) {
        let instruction_fn = PIC10F200::decode_mnemonic(self.instruction_register.instruction_mnemonic);
        instruction_fn(self);
    }

    // Maps a mnemonic to its corresponding execution function
    fn decode_mnemonic(mnemonic: PICInstructionMnemonic) -> InstructionExecutor {
        match mnemonic {
            // Miscellaneous
            PICInstructionMnemonic::NOP => NOP,
            PICInstructionMnemonic::CLRWDT => CLRWDT,
            PICInstructionMnemonic::OPTION => OPTION,
            PICInstructionMnemonic::SLEEP => SLEEP,
            PICInstructionMnemonic::TRIS => TRIS,

            // ALU Operation
            PICInstructionMnemonic::MOVWF => MOVWF,
            PICInstructionMnemonic::CLR => CLR,
            PICInstructionMnemonic::SUBWF => SUBWF,
            PICInstructionMnemonic::DECF => DECF,
            PICInstructionMnemonic::IORWF => IORWF,
            PICInstructionMnemonic::ANDWF => ANDWF,
            PICInstructionMnemonic::XORWF => XORWF,
            PICInstructionMnemonic::ADDWF => ADDWF,
            PICInstructionMnemonic::MOVF => MOVF,
            PICInstructionMnemonic::COMF => COMF,
            PICInstructionMnemonic::INCF => INCF,
            PICInstructionMnemonic::DECFSZ => DECFSZ,
            PICInstructionMnemonic::RRF => RRF,
            PICInstructionMnemonic::RLF => RLF,
            PICInstructionMnemonic::SWAPF => SWAPF,
            PICInstructionMnemonic::INCFSZ => INCFSZ,

            // Bit Operation
            PICInstructionMnemonic::BCF => BCF,
            PICInstructionMnemonic::BSF => BSF,
            PICInstructionMnemonic::BTFSC => BTFSC,
            PICInstructionMnemonic::BTFSS => BTFSS,

            // Control Transfer
            PICInstructionMnemonic::GOTO => GOTO,
            PICInstructionMnemonic::CALL => CALL,
            PICInstructionMnemonic::RETLW => RETLW,

            // Operations with W
            PICInstructionMnemonic::MOVLW => MOVLW,
            PICInstructionMnemonic::IORLW => IORLW,
            PICInstructionMnemonic::ANDLW => ANDLW,
            PICInstructionMnemonic::XORLW => XORLW,

            PICInstructionMnemonic::UND => HALT, // Handle undefined instructions

            // remove these
            PICInstructionMnemonic::RETFIE => todo!(),
            PICInstructionMnemonic::MOVLB => todo!(),
            PICInstructionMnemonic::RETURN => todo!(),
        }
    }
}
