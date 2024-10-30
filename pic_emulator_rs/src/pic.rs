use crate::{data_memory::{RegisterFile, SpecialPurposeRegisters}, instructions::*, nbitnumber::{
    u12, u2, u3, u5, u9, NBitNumber, NumberOperations
}, program_memory::{ProgramMemory, RESET_VECTOR}};

#[derive(Clone, Copy)]

//Highest level wrapper of the MCU
pub struct PIC10F200 {
    data_memory : RegisterFile,
    program_memory : ProgramMemory,
    program_counter : u9,
    instruction_register : PICInstruction,
    w_register : u8,
    io_pins : [bool; 3]
}

pub enum PIC10F2Types {
    PIC10F200,
    PIC10F202,
    PIC10F204,
    PIC10F206,
}
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

trait Programmable {
    fn program_chip(&mut self, new_program: [u12; 0x200]);
}
impl Programmable for PIC10F200 {
    fn program_chip(&mut self, new_program: [u12; 0x200]) {
        self.program_memory.flash(new_program);
        self.data_memory.flash();
    }
}

trait PipelinedTuringMachine {
    fn power_on_initialize(&mut self);
    fn fetch(&mut self);
    fn execute(&mut self);
    fn tick(&mut self);
    fn decode_mnemonic(&mut self);
}
impl PipelinedTuringMachine for PIC10F200 {
    /*
    Instruction Flow Summary

    An Instruction Cycle consists of four Q Cycles (Q1, Q2, Q3, Q4) - 4x Quadrature Clock dividing OSC1 increasing in phase by 90 degrees
    The PIC10F200 has a 4MHz internal clock -> 250nS per Q cycle, 1uS per instruction cycle
    Two stage pipeline, both stages take one instruction cycle
    -> two instruction cycles for an instruction to traverse the pipeline, one instruction completes every instruction cycle (except the first instruction cycle)
    Fetch in Q1, Read data in Q2, Decode & Execute Q2-4, Write data in Q4
     */
    fn power_on_initialize(&mut self) {
        //data sheet page 18
        self.data_memory.write(u5::new(SpecialPurposeRegisters::PCL as u16), 0xFF);
        self.data_memory.write(u5::new(SpecialPurposeRegisters::STATUS as u16), 0x18);
        self.data_memory.write(u5::new(SpecialPurposeRegisters::FSR as u16), 0x70);
        self.data_memory.write(u5::new(SpecialPurposeRegisters::OSCCAL as u16), 0xFE);
        self.data_memory.write(u5::new(SpecialPurposeRegisters::CMCON0 as u16), 0xFF);
    }

    fn tick(&mut self) {
        self.fetch();

        if (self.program_counter) == RESET_VECTOR {
            //the first cycle should skip execution stage, AKA when PCL == RESET_VECTOR
            return;
        }
        self.execute(); 
    }

    fn fetch(&mut self) {
        //The PC is incremented by 1 & the instruction is placed into the instruction register every Q1 cycle
        //if not Q1 cycle, then return

        let PCL = self.data_memory.read(u5::new(SpecialPurposeRegisters::PCL as u16));
        self.program_counter = u9::new(PCL as u16);

        //Fetch the instruction from the program memory
        self.instruction_register = PICInstruction::from_u12(self.program_memory.fetch(self.program_counter));
    }

    fn execute(&mut self) {
        // Switch on Q

        //Decode & read data during Q2
        self.decode_mnemonic(); // return function pointer to execute & data register to read from
        // Read data during Q2, data is a static variable

        //Execute during Q3 
        //Execute the instruction and get a tuple of the data to write to memory (register, value)
        
        //Write data during Q4
    }

    // decoded during Q2
    fn decode_mnemonic(&mut self)
    {
        match self.instruction_register.instruction_category {
            PICInstructionType::ALUOperation => {
                match (self.instruction_register.instruction_raw.as_u16() & 0x3C0) >> 6 {
                    //4 bit opcode 9 downto 6, right shifted by 6
                    0x000 => MOVWF(self),
                    0x001 => CLR(self),
                    0x002 => SUBWF(self),
                    0x003 => DECF(self),
                    0x004 => IORWF(self),
                    0x005 => ANDWF(self),
                    0x006 => XORWF(self),
                    0x007 => ADDWF(self),
                    0x008 => MOVF(self),
                    0x009 => COMF(self),
                    0x00A => INCF(self),
                    0x00B => DECF(self),
                    0x00C => RRF(self),
                    0x00D => RLF(self),
                    0x00E => SWAPF(self),
                    0x00F => INCFSZ(self),
                    _ => HALT(self) //There should not be any undefined ALU opearations
                }
            }
            PICInstructionType::BitOperation => {
                match self.instruction_register.instruction_raw.as_u16() & (0x300) {
                    //2 bit op code bits 9 & 8
                    0x000 => BCF(self),
                    0x100 => BSF(self),
                    0x200 => BTFSC(self),
                    0x300 => BTFSS(self),
                    _ => HALT(self),
                }
            }
            PICInstructionType::ControlTransfer => {
                match self.instruction_register.instruction_raw.as_u16() & (0x300) {
                    //2 bit opcode bits 9 & 8
                    0x000 => RETLW(self),
                    0x100 => CALL(self),
                    0x200 | 0x300 => GOTO(self),
                    _ => HALT(self)
                }
            }
            PICInstructionType::Miscellaneous => {
                //5 bit opcode 4 downto 0
                match self.instruction_register.instruction_raw.as_u16() & (0x01F) {
                    0x000 => NOP(self),
                    0x002 => OPTION(self),
                    0x003 => SLEEP(self),
                    0x004 => CLRWDT(self),
                    0x005..=0x007 => TRIS(self),
                    _ => HALT(self),
                }
            }
            PICInstructionType::OperationsWithW => {
                match self.instruction_register.instruction_raw.as_u16() & (0x300) {
                    //2 bit opcode 9 & 8
                    0x000 => MOVLW(self),
                    0x100 => IORLW(self),
                    0x200 => ANDLW(self),
                    0x300 => XORLW(self),
                    _ => HALT(self),
                }
            }
        };
    }
}

#[derive(Clone, Copy)]
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
