use crate::{nbitnumber::{
    u12, u9, u5, u3, u2,
    NumberOperations, NBitNumber
}, data_memory::RegisterFile, program_memory::ProgramMemory, data_memory::SpecialPurposeRegisters, instructions::*};

#[derive(Clone, Copy)]

//Highest level wrapper of the MCU
pub struct PIC10F200 {
    pub data_memory : RegisterFile,
    pub program_memory : ProgramMemory,
    pub program_counter : u9,
    pub current_instruction : PICInstruction,
    pub w_register : u8,
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

trait TuringMachine {
    fn power_on_initialize(&mut self);
    fn fetch(&mut self);
    fn execute(&mut self);
    fn tick(&mut self);
    fn decode_mnemonic(&mut self);
}

impl TuringMachine for PIC10F200 {
    fn power_on_initialize(&mut self) {
        //data sheet page 18
        self.data_memory.write(u5::new(SpecialPurposeRegisters::PCL as u16), 0xFF);
        self.data_memory.write(u5::new(SpecialPurposeRegisters::STATUS as u16), 0x18);
        self.data_memory.write(u5::new(SpecialPurposeRegisters::FSR as u16), 0x70);
        self.data_memory.write(u5::new(SpecialPurposeRegisters::OSCCAL as u16), 0xFE);
        self.data_memory.write(u5::new(SpecialPurposeRegisters::CMCON0 as u16), 0xFF);
    }

    fn tick(&mut self) {
        //One instruction cycle consists of four clock cycles
        //Internal clock is 4MHz, so one instruction cycle is 1us
        //4 MHz is max clock speed for this chip
        //Execute first, per the pipeline flow
        self.execute(); //the first cycle should skip execution, AKA when PCL == RESET_VECTOR
        self.fetch();
    }

    fn fetch(&mut self) {
        //Read the program counter
        let PCL = self.data_memory.read(u5::new(SpecialPurposeRegisters::PCL as u16));
        self.program_counter = u9::new(PCL as u16); //translate PC to u9 (we might want to sign extend it for off chip memory

        
        self.current_instruction = PICInstruction::from_u12(self.program_memory.fetch(self.program_counter));
    }

    fn execute(&mut self) {
        //start the pipeline
        self.decode_mnemonic();
    }

    fn decode_mnemonic(&mut self)
    {
        match self.current_instruction.instruction_category {
            PICInstructionType::ALUOperation => {
                match (self.current_instruction.instruction_raw.as_u16() & 0x3C0) >> 6 {
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
                match self.current_instruction.instruction_raw.as_u16() & (0x300) {
                    //2 bit op code bits 9 & 8
                    0x000 => BCF(self),
                    0x100 => BSF(self),
                    0x200 => BTFSC(self),
                    0x300 => BTFSS(self),
                    _ => HALT(self),
                }
            }
            PICInstructionType::ControlTransfer => {
                match self.current_instruction.instruction_raw.as_u16() & (0x300) {
                    //2 bit opcode bits 9 & 8
                    0x000 => RETLW(self),
                    0x100 => CALL(self),
                    0x200 | 0x300 => GOTO(self),
                    _ => HALT(self)
                }
            }
            PICInstructionType::Miscellaneous => {
                //5 bit opcode 4 downto 0
                match self.current_instruction.instruction_raw.as_u16() & (0x01F) {
                    0x000 => NOP(self),
                    0x002 => OPTION(self),
                    0x003 => SLEEP(self),
                    0x004 => CLRWDT(self),
                    0x005..=0x007 => TRIS(self),
                    _ => HALT(self),
                }
            }
            PICInstructionType::OperationsWithW => {
                match self.current_instruction.instruction_raw.as_u16() & (0x300) {
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

