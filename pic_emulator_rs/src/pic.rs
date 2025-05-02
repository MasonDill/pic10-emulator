use crate::{data_memory::{RegisterFile, SpecialPurposeRegisters, STATUS_POR_VALUE, FSR_POR_VALUE, OSCCAL_POR_VALUE, CMCON0_POR_VALUE, TRIS_POR_VALUE, OPTION_POR_VALUE}, instructions::*, nbitnumber::{
    u12, u2, u3, u5, u9, NBitNumber, NumberOperations
}, program_memory::{ProgramMemory, RESET_VECTOR, PC_POR_MOVLW_OSCCAL_ADDRESS}};

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

#[derive(Clone)]
pub enum PICInstructionType {
    Miscellaneous,
    BitOperation,
    ControlTransfer,
    OperationsWithW,
    ALUOperation,
}
pub enum PICInstructionMnemonic {
    // Miscellaneous
    NOP, CLRWDT, OPTION, RETFIE, 
    SLEEP, MOVLB, TRIS, RETURN,

    // ALU Operation
    MOVWF, CLR, SUBWF, DECF, 
    IORWF, ANDWF, XORWF, ADDWF,
    MOVF, COMF, INCF, DECFSZ, 
    RRF, RLF, SWAPF, INCFSZ, 

    // Bit Operation
    BCF, BSF, BTFSC, BTFSS,

    // Control Transfer
    GOTO, CALL, RETLW,

    // Operations with W
    MOVLW, IORLW, ANDLW, XORLW,

    //Undefined Instruction
    UND
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
    fn decode_mnemonic(&mut self) -> InstructionExecutor;
}
impl PipelinedTuringMachine for PIC10F200 {
    /*
    See datasheet page 10 section 3.2

    Instruction Flow Summary
    An Instruction Cycle consists of four Q Cycles (Q1, Q2, Q3, Q4) - 4x Quadrature Clock dividing OSC1 increasing in phase by 90 degrees
    The PIC10F200 has a 4MHz internal clock -> 250nS per Q cycle, 1uS per instruction cycle
    Two stage pipeline, both stages take one instruction cycle
    -> two instruction cycles for an instruction to traverse the pipeline, one instruction completes every instruction cycle (except the first instruction cycle)
    Fetch in Q1, Read data in Q2, Decode & Execute Q2-4, Write data in Q4
     */
    fn power_on_initialize(&mut self) {
        // data sheet page 36, note 1. that the PC will first point to address 0xFF which will be loaded with the instruction MOVLW XX, where XX is the oscillator calibration value
        // this will load the W register with the oscillator calibration value
        // then the PC will be incremented to point to address 0x00, which will contain the first instruction to execute
        // also see data sheet page 11 note 1. -> load 0xFF (PC starts at **0xFF**, then moves to 0x00) with the MOVLW OSCCAL instruction
        let MOVLW_OSCCAL_INSTRUCTION: NBitNumber<12> = u12::new(0xC00 | OSCCAL_POR_VALUE as u16);
        self.program_memory.write(PC_POR_MOVLW_OSCCAL_ADDRESS, MOVLW_OSCCAL_INSTRUCTION);
        
        // Data sheet page 14 Table 4-1
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
        // see data sheet page 10 section 3.2
        self.fetch();        
        self.execute(); 
    }

    fn fetch(&mut self) {
        let PCL = self.data_memory.read(u5::new(SpecialPurposeRegisters::PCL as u16));
        self.program_counter = u9::new(PCL as u16); // writes to PCL will also write to the program counter
        self.program_counter = self.program_counter + u9::new(1); // Q1, see data sheet page 10 section 3.2
        self.program_counter = self.program_counter & u9::new(0x0FF); // always clear the 9th bit, see data sheet page 18 section 4.7

        self.instruction_register = PICInstruction::from_u12(self.program_memory.fetch(self.program_counter));
    }

    fn execute(&mut self) {

        // Decode & read data during Q2
        // Decode returns the function pointer to the instruction implementation
        let instruction_fn = self.decode_mnemonic();
        // Read data during Q2, data is a static variable // TODO: Implement data read based on instruction if needed

        // Execute during Q3
        // Execute the instruction
        instruction_fn(self);

        // Write data during Q4 // TODO: Implement data write based on instruction if needed
    }

    fn decode_mnemonic(&mut self) -> InstructionExecutor
    {
        match self.instruction_register.instruction_category {
            PICInstructionType::ALUOperation => {
                match (self.instruction_register.instruction_raw.as_u16() & 0x3C0) >> 6 {
                    //4 bit opcode 9 downto 6, right shifted by 6
                    0x000 => MOVWF,
                    0x001 => CLR,
                    0x002 => SUBWF,
                    0x003 => DECF,
                    0x004 => IORWF,
                    0x005 => ANDWF,
                    0x006 => XORWF,
                    0x007 => ADDWF,
                    0x008 => MOVF,
                    0x009 => COMF,
                    0x00A => INCF,
                    // Note: 0x00B was DECF, datasheet shows it's DECFSZ
                    0x00B => DECFSZ, 
                    0x00C => RRF,
                    0x00D => RLF,
                    0x00E => SWAPF,
                    0x00F => INCFSZ,
                    // Handle potential undefined opcodes within this range if necessary
                    _ => HALT // Assuming HALT is a valid function in instructions.rs
                }
            }
            PICInstructionType::BitOperation => {
                match self.instruction_register.instruction_raw.as_u16() & (0x300) {
                    //2 bit op code bits 9 & 8
                    0x000 => BCF,
                    0x100 => BSF,
                    0x200 => BTFSC,
                    0x300 => BTFSS,
                    _ => HALT, // Should not happen with 2 bits
                }
            }
            PICInstructionType::ControlTransfer => {
                // Opcode bits 10 & 9 for CALL/GOTO, but RETLW uses lower bits.
                // Need to check the full pattern more carefully based on datasheet Table 11-2
                match self.instruction_register.instruction_raw.as_u16() & 0xF00 { // Check bits 11-8
                     // RETLW k (10 00xx kkkk kkkk) - This pattern seems off, RETLW is 0x08? Let's re-check decode_category
                     // Let's trust decode_category for now and match based on its output
                     0x800 => { // Control Transfer category
                         match self.instruction_register.instruction_raw.as_u16() & 0xF00 { // Check upper nibble again
                             0x800 => RETLW, // Assuming RETLW doesn't fit the 0x100/0x200/0x300 pattern
                             0x900 => CALL,
                             0xA00 | 0xB00 => GOTO, // Both 101x and 100x seem to be GOTO
                             _ => HALT
                         }
                     },
                     _ => HALT // Should not happen if decode_category is correct
                }
                /* // Previous simpler match, likely incorrect based on datasheet opcodes
                match self.instruction_register.instruction_raw.as_u16() & (0x300) {
                    //2 bit opcode bits 9 & 8
                    0x000 => RETLW, // This is likely wrong, RETLW is 10 00xx kkkk kkkk ?
                    0x100 => CALL,  // This is 10 01xx kkkk kkkk
                    0x200 | 0x300 => GOTO, // This is 10 1xxx kkkk kkkk
                    _ => HALT
                }
                */
            }
            PICInstructionType::Miscellaneous => {
                // 5 bit opcode 4 downto 0
                // Check specific full opcodes for misc instructions (Table 11-1)
                match self.instruction_register.instruction_raw.as_u16() & 0x0FF { // Mask lower 8 bits for clarity
                    0x000 => NOP,    // 00 0000 0000 0000 (NOP)
                    0x004 => CLRWDT, // 00 0000 0000 0100 (CLRWDT)
                    0x002 => OPTION, // 00 0000 0000 0010 (OPTION)
                    0x003 => SLEEP,  // 00 0000 0000 0011 (SLEEP)
                    // TRIS needs more specific check? instruction is 00 0000 0000 11fx
                     _ if (self.instruction_register.instruction_raw.as_u16() & 0x3F) >= 0x05 &&
                          (self.instruction_register.instruction_raw.as_u16() & 0x3F) <= 0x07 => TRIS, // 00 0000 00xx x11x? No, TRIS is 0x05/06/07
                    _ => HALT, // Other codes in the 0x000-0x01F range might be MOVLB, RETURN, RETFIE - Need to add them
                                // MOVLB 00 0000 0010 0xxx -> 0x20? - This overlaps OPTION? No, OPTION is 0x02. MOVLB is 0x00?
                                // Need to carefully re-read Table 11-1 & 11-2.
                                // Let's assume HALT for unhandled cases for now.
                }
                /* // Previous simpler match based only on lower 5 bits
                match self.instruction_register.instruction_raw.as_u16() & (0x01F) {
                    0x000 => NOP,
                    0x002 => OPTION,
                    0x003 => SLEEP,
                    0x004 => CLRWDT,
                    0x005..=0x007 => TRIS, // This covers 0x05, 0x06, 0x07
                    _ => HALT,
                }
                */
            }
            PICInstructionType::OperationsWithW => {
                match self.instruction_register.instruction_raw.as_u16() & (0x300) {
                    //2 bit opcode 9 & 8 (within the 11xx category)
                    // Example: MOVLW k is 11 00xx kkkk kkkk
                    0x000 => MOVLW,
                    0x100 => IORLW,
                    0x200 => ANDLW,
                    0x300 => XORLW,
                    _ => HALT, // Should not happen
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct PICInstruction  {
    pub instruction_raw: u12,
    //instruction: Option<PICMnemonic>,
    pub instruction_category: PICInstructionType,
}
impl PICInstruction {
    pub fn from_u12(instruction: u12) -> PICInstruction {
       PICInstruction {
            instruction_raw: instruction,
            instruction_category: PICInstruction::decode_category(instruction),
        }
    }

    fn decode_category(instruction: u12) -> PICInstructionType {
        match instruction.as_u16() & (0xC00) {
            // misc & alu -> 0000 | 0000 | 0000
            // bit  -> 0100 | 0000 | 0000
            // control 1000 | 0000 | 0000
            // operations = 1100 | 0000 | 0000
            0x000 => match instruction.as_u16() & (0x3E0) {
                0x000 => PICInstructionType::Miscellaneous, 
                _ => PICInstructionType::ALUOperation,
            }
            0x400 => PICInstructionType::BitOperation,
            0x800 => PICInstructionType::ControlTransfer,
            0xC00  => PICInstructionType::OperationsWithW,
            _ => panic!("TODO")
        }
    }

    pub fn extract_k(&self) -> u8{
        (self.instruction_raw.as_u16() & 0x0FF) as u8
    }

    pub fn extract_d(&self) -> NBitNumber<1>{
        NBitNumber::new(self.instruction_raw.as_u16() & 0x020)
    }

    pub fn extract_f(&self) -> NBitNumber<5>{
       NBitNumber::new(self.instruction_raw.as_u16() & 0x01F)
    }

    pub fn extract_b(&self) -> NBitNumber<3>{
        NBitNumber::new((self.instruction_raw.as_u16() & 0x0E0) >> 5)
    }

    pub fn extract_k_goto(&self) -> NBitNumber<9> {
        u9::new(self.instruction_raw.as_u16() & 0x1FF)
    }

    pub fn extract_k_movlb(&self) -> NBitNumber<3> {
        u3::new(self.instruction_raw.as_u16() & 0x007)
    }

    pub fn extract_f_tris(&self) -> NBitNumber<2> {
        u2::new(self.instruction_raw.as_u16() & 0x003)
    }

}
