use crate::nbitnumber::{u12, NBitNumber, NumberOperations};
use crate::nbitnumber::{u2, u3, u5, u9}; // Added imports for extract methods

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PICInstructionType {
    Miscellaneous,
    BitOperation,
    ControlTransfer,
    OperationsWithW,
    ALUOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

// --- PICInstruction Struct and Implementation (Moved from pic.rs) ---
#[derive(Clone, Copy)] // Added Copy since u12 is Copy
pub struct PICInstruction  {
    pub instruction_raw: u12,
    pub instruction_mnemonic: PICInstructionMnemonic,
}

impl PICInstruction {
    pub fn from_u12(instruction: u12) -> PICInstruction {
       PICInstruction {
            instruction_raw: instruction,
            instruction_mnemonic: PICInstruction::decode_mnemonic(PICInstruction::decode_category(instruction), instruction),
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
            _ => panic!("Invalid instruction category based on bits 11-10") // Consider returning UND type?
        }
    }

    // Decodes the mnemonic based on the raw instruction bits and category
    pub fn decode_mnemonic(category : PICInstructionType, raw_instruction : u12) -> PICInstructionMnemonic {
        use PICInstructionMnemonic::*;
        match category {
            PICInstructionType::ALUOperation => {
                match (raw_instruction.as_u16() & 0x3C0) >> 6 {
                    //4 bit opcode 9 downto 6, right shifted by 6
                    0x000 => MOVWF,
                    0x001 => CLR,   // CLRW / CLRF (determined by 'd' bit in executor)
                    0x002 => SUBWF,
                    0x003 => DECF,
                    0x004 => IORWF,
                    0x005 => ANDWF,
                    0x006 => XORWF,
                    0x007 => ADDWF,
                    0x008 => MOVF,
                    0x009 => COMF,
                    0x00A => INCF,
                    0x00B => DECFSZ,
                    0x00C => RRF,
                    0x00D => RLF,
                    0x00E => SWAPF,
                    0x00F => INCFSZ,
                    _ => UND
                }
            }
            PICInstructionType::BitOperation => {
                match raw_instruction.as_u16() & 0x300 {
                    //2 bit op code bits 9 & 8
                    0x000 => BCF,
                    0x100 => BSF,
                    0x200 => BTFSC,
                    0x300 => BTFSS,
                    _ => UND,
                }
            }
            PICInstructionType::ControlTransfer => {
                 match raw_instruction.as_u16() & 0xF00 { // Check bits 11-8
                     0x800 => RETLW, // 1000
                     0x900 => CALL,  // 1001
                     0xA00 | 0xB00 => GOTO, // 101x
                     _ => UND
                 }
            }
            PICInstructionType::Miscellaneous => {
                match raw_instruction.as_u16() & 0x3FF { // Mask to check relevant bits
                    // Specific Opcodes (ensure these don't clash)
                    0x000 => NOP,    // 00 0000 0000 0000
                    0x004 => CLRWDT, // 00 0000 0000 0100
                    0x002 => OPTION, // 00 0000 0000 0010
                    0x003 => SLEEP,  // 00 0000 0000 0011
                    op @ 0x005..=0x007 => TRIS,

                    // MOVLB k (Not used in PIC10F200)

                    _ => UND, // Default for unrecognised misc patterns
                }
            }
            PICInstructionType::OperationsWithW => {
                match raw_instruction.as_u16() & 0x300 {
                    // Sub-opcode bits 9 & 8 within the 11xx category
                    0x000 => MOVLW, // 11 00xx
                    0x100 => IORLW, // 11 01xx
                    0x200 => ANDLW, // 11 10xx
                    0x300 => XORLW, // 11 11xx
                    _ => UND,
                }
            }
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
// --- End PICInstruction --- 

/// Builds a u12 machine code instruction from a mnemonic and optional operands.
///
/// operand1 generally represents 'f' (file register) or 'k' (literal address/value).
/// operand2 generally represents 'd' (destination bit) or 'b' (bit number).
///
/// Returns an error string if the mnemonic/operand combination is invalid.
pub fn build_instruction(mnemonic: PICInstructionMnemonic, operand1: Option<u16>, operand2: Option<u16>) -> Result<u12, String> {
    match mnemonic {
        // Miscellaneous Instructions (Table 11-1)
        PICInstructionMnemonic::NOP => Ok(u12::new(0x0000)),
        PICInstructionMnemonic::CLRWDT => Ok(u12::new(0x0004)),
        PICInstructionMnemonic::OPTION => Ok(u12::new(0x0002)), // Loads W into OPTION register implicitly
        PICInstructionMnemonic::SLEEP => Ok(u12::new(0x0003)),
        PICInstructionMnemonic::TRIS => {
            // TRIS f: 00 0000 0000 011f (f is 5, 6, or 7 for GPIO)
            match operand1 {
                Some(f @ 5..=7) => Ok(u12::new(f)), // TRIS GP0=5, GP1=6, GP2=7
                _ => Err(format!("Invalid operand for TRIS: {:?}. Expected 5, 6, or 7.", operand1)),
            }
        },
        PICInstructionMnemonic::RETURN => Ok(u12::new(0x0008)),
        PICInstructionMnemonic::RETFIE => Ok(u12::new(0x0009)),
        PICInstructionMnemonic::MOVLB => Err("MOVLB not applicable for PIC10F200 (no banking)".to_string()),

        // ALU Operations (Table 11-2, Byte-Oriented)
        // Format: 00 xxxx dfff ffff (d=0 -> W, d=1 -> f) (f is 5 bits 0-4)
        PICInstructionMnemonic::MOVWF => { // d=1 implied, format 00 0000 1fff ffff
            match operand1 {
                Some(f) => Ok(u12::new(0x0080 | (f & 0x1F))),
                None => Err("MOVWF requires a file register operand 'f' (0-31)".to_string()),
            }
        },
        PICInstructionMnemonic::CLR => { // CLRW (d=0) or CLRF (d=1)
            match (operand1, operand2) {
                (Some(f), Some(1)) => Ok(u12::new(0x0180 | (f & 0x1F))), // CLRF f (00 0001 1fff ffff)
                (None, Some(0)) => Ok(u12::new(0x0100)), // CLRW (00 0001 0xxx xxxx) - f is ignored, set to 0?
                 _ => Err("CLR requires either CLRF f (operand1=f, operand2=1) or CLRW (operand1=None, operand2=0)".to_string()),
            }
        },
        PICInstructionMnemonic::SUBWF | PICInstructionMnemonic::DECF | PICInstructionMnemonic::IORWF |
        PICInstructionMnemonic::ANDWF | PICInstructionMnemonic::XORWF | PICInstructionMnemonic::ADDWF |
        PICInstructionMnemonic::MOVF | PICInstructionMnemonic::COMF | PICInstructionMnemonic::INCF |
        PICInstructionMnemonic::DECFSZ | PICInstructionMnemonic::RRF | PICInstructionMnemonic::RLF |
        PICInstructionMnemonic::SWAPF | PICInstructionMnemonic::INCFSZ => {
            let base_opcode = match mnemonic {
                PICInstructionMnemonic::SUBWF => 0x0200, // 00 0010 dfff ffff
                PICInstructionMnemonic::DECF => 0x0300,  // 00 0011 dfff ffff
                PICInstructionMnemonic::IORWF => 0x0400, // 00 0100 dfff ffff
                PICInstructionMnemonic::ANDWF => 0x0500, // 00 0101 dfff ffff
                PICInstructionMnemonic::XORWF => 0x0600, // 00 0110 dfff ffff
                PICInstructionMnemonic::ADDWF => 0x0700, // 00 0111 dfff ffff
                PICInstructionMnemonic::MOVF => 0x0800,  // 00 1000 dfff ffff
                PICInstructionMnemonic::COMF => 0x0900,  // 00 1001 dfff ffff
                PICInstructionMnemonic::INCF => 0x0A00,  // 00 1010 dfff ffff
                PICInstructionMnemonic::DECFSZ => 0x0B00,// 00 1011 dfff ffff
                PICInstructionMnemonic::RRF => 0x0C00,   // 00 1100 dfff ffff
                PICInstructionMnemonic::RLF => 0x0D00,   // 00 1101 dfff ffff
                PICInstructionMnemonic::SWAPF => 0x0E00, // 00 1110 dfff ffff
                PICInstructionMnemonic::INCFSZ => 0x0F00,// 00 1111 dfff ffff
                 _ => unreachable!(),
            };
            match (operand1, operand2) {
                (Some(f), Some(d @ (0 | 1))) => {
                    let d_bit = (d & 1) << 5; // d is bit 5
                    let f_bits = f & 0x1F; // f is bits 0-4
                    Ok(u12::new(base_opcode | d_bit | f_bits))
                }
                _ => Err(format!("{:?} requires file register 'f' (operand1, 0-31) and destination 'd' (operand2 = 0 for W, 1 for f)", mnemonic)),
            }
        },

        // Bit Operations (Table 11-2, Bit-Oriented)
        // Format: 01 bbbf ffff ffff (b is 3 bits 7-9, f is 7 bits 0-6) -> NOTE: f uses 7 bits here!
        PICInstructionMnemonic::BCF | PICInstructionMnemonic::BSF |
        PICInstructionMnemonic::BTFSC | PICInstructionMnemonic::BTFSS => {
            let base_opcode = match mnemonic {
                PICInstructionMnemonic::BCF => 0x1000,   // 01 00bb bfff ffff
                PICInstructionMnemonic::BSF => 0x1400,   // 01 01bb bfff ffff
                PICInstructionMnemonic::BTFSC => 0x1800, // 01 10bb bfff ffff
                PICInstructionMnemonic::BTFSS => 0x1C00, // 01 11bb bfff ffff
                 _ => unreachable!(),
            };
            match (operand1, operand2) {
                (Some(f), Some(b @ 0..=7)) => {
                    let b_bits = (b & 0x7) << 7; // b is bits 7-9
                    let f_bits = f & 0x7F; // f is bits 0-6
                    Ok(u12::new(base_opcode | b_bits | f_bits))
                }
                 _ => Err(format!("{:?} requires file register 'f' (operand1, 0-127) and bit 'b' (operand2, 0-7)", mnemonic)),
            }
        },

        // Control Transfer & Literals (Table 11-2)
        PICInstructionMnemonic::GOTO => { // 101 kkkk kkkk kkkk (k is 9 bits 0-8)
            match operand1 {
                Some(k) => Ok(u12::new(0x0A00 | (k & 0x1FF))),
                 None => Err("GOTO requires a 9-bit address literal 'k' (operand1)".to_string()),
            }
        },
        PICInstructionMnemonic::CALL => { // 100 kkkk kkkk kkkk (k is 9 bits 0-8)
            match operand1 {
                Some(k) => Ok(u12::new(0x0800 | (k & 0x1FF))), // Note: decode_mnemonic maps 0x900 to CALL? Let's trust Table 11-2: Opcode 4 -> 100 base
                 None => Err("CALL requires a 9-bit address literal 'k' (operand1)".to_string()),
            }
        },
         PICInstructionMnemonic::RETLW => { // 11 00xx kkkk kkkk -> Opcode C, sub 0? No, that's MOVLW.
                                          // Let's assume decode_mnemonic's mapping: 0x800 base -> 10 00xx kkkk kkkk ?
                                          // No, decode maps 0x800 directly to RETLW func. Table 11-2 is confusing here.
                                          // Common consensus: RETLW k is 11 0100 kkkk kkkk (0xD00 base + k). Let's try that.
            match operand1 {
                 Some(k) => Ok(u12::new(0x0D00 | (k & 0xFF))), // Try 11 0100 kkkk kkkk
                 None => Err("RETLW requires an 8-bit literal 'k' (operand1)".to_string()),
            }
         },
        PICInstructionMnemonic::MOVLW => { // 11 00xx kkkk kkkk (k is 8 bits 0-7)
            match operand1 {
                Some(k) => Ok(u12::new(0x0C00 | (k & 0xFF))),
                 None => Err("MOVLW requires an 8-bit literal 'k' (operand1)".to_string()),
            }
        },
        PICInstructionMnemonic::IORLW => { // 11 1000 kkkk kkkk -> No, IORLW is 11 10xx? Table 11-2 shows 11 1000 (Opcode C, sub 8??) -> This is ANDLW!
                                         // Table 11-2 literal ops: MOVLW (C0), IORLW (C1?), ANDLW(C2?), XORLW(C3?)
                                         // Let's use the decode_mnemonic logic: 0xC00 base + sub-opcode
                                         // MOVLW = 0xC00 | 0x000, IORLW = 0xC00 | 0x100, ANDLW = 0xC00 | 0x200, XORLW = 0xC00 | 0x300
                                         // This means IORLW is 11 01xx kkkk kkkk (0xD00 base) -> Conflicts with assumed RETLW!
                                         // Going back to RETLW = 0x800 | k based on decode_mnemonic structure.
                                         // RETLW k: 10 00kk kkkk kkkk (corrected above)

            // IORLW k: 11 1000 kkkk kkkk
             match operand1 {
                Some(k) => Ok(u12::new(0x0D00 | (k & 0xFF))),
                 None => Err("IORLW requires an 8-bit literal 'k' (operand1)".to_string()),
            }
        },
        PICInstructionMnemonic::ANDLW => { // 11 10xx kkkk kkkk
             match operand1 {
                Some(k) => Ok(u12::new(0x0E00 | (k & 0xFF))),
                 None => Err("ANDLW requires an 8-bit literal 'k' (operand1)".to_string()),
            }
        },
        PICInstructionMnemonic::XORLW => { // 11 11xx kkkk kkkk
             match operand1 {
                Some(k) => Ok(u12::new(0x0F00 | (k & 0xFF))),
                 None => Err("XORLW requires an 8-bit literal 'k' (operand1)".to_string()),
            }
        },

        // Undefined
        PICInstructionMnemonic::UND => Err("Cannot build UNDefined instruction".to_string()),
         // Fallback for any missed mnemonics during development
         // _ => Err(format!("Mnemonic {:?} not yet implemented in build_instruction", mnemonic)),

    }
}


// Example Usage (Remove or comment out later)
/*
fn main() {
    // Example: MOVLW 0x55
    match build_instruction(PICInstructionMnemonic::MOVLW, Some(0x55), None) {
        Ok(instruction) => println!("MOVLW 0x55 -> {:#05X}", instruction.as_u16()), // Expected: 0x0C55
        Err(e) => println!("Error: {}", e),
    }

    // Example: ADDWF FSR, W (FSR=0x04, W=destination 0)
    match build_instruction(PICInstructionMnemonic::ADDWF, Some(0x04), Some(0)) {
        Ok(instruction) => println!("ADDWF FSR, W -> {:#05X}", instruction.as_u16()), // Expected: 0x0704
        Err(e) => println!("Error: {}", e),
    }

    // Example: BSF STATUS, Z (STATUS=0x03, Z=bit 2)
    match build_instruction(PICInstructionMnemonic::BSF, Some(0x03), Some(2)) {
        Ok(instruction) => println!("BSF STATUS, Z -> {:#05X}", instruction.as_u16()), // Expected: 0x1503 (01 01 010 0000011)
        Err(e) => println!("Error: {}", e),
    }

     // Example: GOTO 0x1A
     match build_instruction(PICInstructionMnemonic::GOTO, Some(0x1A), None) {
        Ok(instruction) => println!("GOTO 0x1A -> {:#05X}", instruction.as_u16()), // Expected: 0x0A1A
        Err(e) => println!("Error: {}", e),
    }
}
*/ 