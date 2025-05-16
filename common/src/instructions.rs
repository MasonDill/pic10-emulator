use crate::nbitnumber::{self, u12, NBitNumber, NumberOperations, NBit};
use crate::nbitnumber::{u2, u3, u5, u9}; // Added imports for extract methods
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

// Define a trait for instruction properties
// pub trait InstructionProperties<const N: usize> {
//     /// Returns the base opcode mask for the instruction mnemonic as an NBitNumber<N>.
//     /// This typically represents the fixed bits of the instruction's opcode.
//     /// Variable bits (like addresses 'f', destination 'd', literals 'k', bit 'b') are masked out.
//     /// The returned value is masked/truncated to N bits.
//     fn get_opcode(&self) -> NBitNumber<N>;
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PICInstructionType {
    Miscellaneous,
    BitOperation,
    ControlTransfer,
    OperationsWithW,
    ALUOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum PICInstructionMnemonic {
    // basic instructions
    ADDWF,
    ANDWF,
    CLRF,
    CLRW,
    COMF,
    DECF,
    DECFSZ,
    INCF,
    INCFSZ,
    IORWF,
    MOVF,
    MOVWF,
    NOP,
    RLF,
    RRF,
    SUBWF,
    SWAPF,
    XORWF,

    // bit-oriented file register operations
    BCF,
    BSF,
    BTFSC,
    BTFSS,

    // literal and control operations
    ANDLW,
    CALL,
    CLRWDT,
    GOTO,
    IORLW,
    MOVLW,
    OPTION,
    RETLW,
    SLEEP,
    TRIS,
    XORLW,
    
    //Undefined Instruction (not defined in the data sheet, but useful for error handling)
    UND
}

type Opcode = (NBit, PICInstructionMnemonic, &'static str);
static OPCODES: &[Opcode] = &[
    (NBit::N6(NBitNumber::<6>::new(0b000111)),          PICInstructionMnemonic::ADDWF,   "ADDWF"),
    (NBit::N6(NBitNumber::<6>::new(0b000101)),          PICInstructionMnemonic::ANDWF,   "ANDWF"),
    (NBit::N7(NBitNumber::<7>::new(0b0000011)),         PICInstructionMnemonic::CLRF,    "CLRF"),
    (NBit::N12(NBitNumber::<12>::new(0b000001000000)),  PICInstructionMnemonic::CLRW,    "CLRW"),
    (NBit::N6(NBitNumber::<6>::new(0b001001)),          PICInstructionMnemonic::COMF,    "COMF"),
    (NBit::N6(NBitNumber::<6>::new(0b000011)),          PICInstructionMnemonic::DECF,    "DECF"),
    (NBit::N6(NBitNumber::<6>::new(0b001011)),          PICInstructionMnemonic::DECFSZ,  "DECFSZ"),
    (NBit::N6(NBitNumber::<6>::new(0b001010)),          PICInstructionMnemonic::INCF,    "INCF"),
    (NBit::N6(NBitNumber::<6>::new(0b001111)),          PICInstructionMnemonic::INCFSZ,  "INCFSZ"),
    (NBit::N6(NBitNumber::<6>::new(0b000100)),          PICInstructionMnemonic::IORWF,   "IORWF"),
    (NBit::N6(NBitNumber::<6>::new(0b001000)),          PICInstructionMnemonic::MOVF,    "MOVF"),
    (NBit::N6(NBitNumber::<6>::new(0b000001)),          PICInstructionMnemonic::MOVWF,   "MOVWF"),
    (NBit::N12(NBitNumber::<12>::new(0b000000000000)),  PICInstructionMnemonic::NOP,     "NOP"),
    (NBit::N6(NBitNumber::<6>::new(0b001101)),          PICInstructionMnemonic::RLF,     "RLF"),
    (NBit::N6(NBitNumber::<6>::new(0b001100)),          PICInstructionMnemonic::RRF,     "RRF"),
    (NBit::N6(NBitNumber::<6>::new(0b000010)),          PICInstructionMnemonic::SUBWF,   "SUBWF"),
    (NBit::N6(NBitNumber::<6>::new(0b001110)),          PICInstructionMnemonic::SWAPF,   "SWAPF"),
    (NBit::N6(NBitNumber::<6>::new(0b000110)),          PICInstructionMnemonic::XORWF,   "XORWF"),

    // Bit-oriented
    (NBit::N7(NBitNumber::<7>::new(0b0100_000)),        PICInstructionMnemonic::BCF,     "BCF"),
    (NBit::N7(NBitNumber::<7>::new(0b0101_000)),        PICInstructionMnemonic::BSF,     "BSF"),
    (NBit::N7(NBitNumber::<7>::new(0b0100_100)),        PICInstructionMnemonic::BTFSC,   "BTFSC"),
    (NBit::N7(NBitNumber::<7>::new(0b0101_100)),        PICInstructionMnemonic::BTFSS,   "BTFSS"),

    // Literal and Control
    (NBit::N8(NBitNumber::<8>::new(0b1110_0000)),       PICInstructionMnemonic::ANDLW,   "ANDLW"),
    (NBit::N4(NBitNumber::<4>::new(0b1001)),            PICInstructionMnemonic::CALL,    "CALL"),
    (NBit::N12(NBitNumber::<12>::new(0b000000011000)),  PICInstructionMnemonic::CLRWDT,  "CLRWDT"),
    (NBit::N3(NBitNumber::<3>::new(0b101)),             PICInstructionMnemonic::GOTO,    "GOTO"),
    (NBit::N8(NBitNumber::<8>::new(0b1101_0000)),       PICInstructionMnemonic::IORLW,   "IORLW"),
    (NBit::N8(NBitNumber::<8>::new(0b1100_0000)),       PICInstructionMnemonic::MOVLW,   "MOVLW"),
    (NBit::N12(NBitNumber::<12>::new(0b000000000010)),  PICInstructionMnemonic::OPTION,  "OPTION"),
    (NBit::N12(NBitNumber::<12>::new(0b000000000010)),  PICInstructionMnemonic::RETLW,   "RETLW"),
    (NBit::N12(NBitNumber::<12>::new(0b000000011100)),  PICInstructionMnemonic::SLEEP,   "SLEEP"),
    (NBit::N12(NBitNumber::<12>::new(0b000000000001)),  PICInstructionMnemonic::TRIS,    "TRIS"),
    (NBit::N8(NBitNumber::<8>::new(0b1111_0000)),       PICInstructionMnemonic::XORLW,   "XORLW"),
];


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
            instruction_mnemonic: PICInstruction::decode_mnemonic(instruction),
        }
    }

    pub fn from_mnemonic(mnemonic: PICInstructionMnemonic) -> PICInstruction {
        PICInstruction {
            instruction_raw: u12::new(0),
            instruction_mnemonic: mnemonic,
        }
    }

    pub fn encode_mnemonic(mnemonic: PICInstructionMnemonic) -> u12 {
        todo!()
    }

    fn align_opcode<const N: usize>(n: NBitNumber<N>) -> u12 {
        if N > 12 {
            panic!("N must be less than or equal to 12");
        }
        // move to the 12th bit position
        let shifted: u16 = n.value << (12 - N);

        let mask_bits: usize = 12 - N;
        let mask: u16 = (1 << mask_bits) - 1;
        let nmask: u16 = !mask;
        let result: u16 = shifted & nmask;
        
        return NBitNumber::<12>::new(result);
    }

    // Decodes the mnemonic based on the raw instruction bits
    pub fn decode_mnemonic(raw_instruction : NBitNumber<12>) -> PICInstructionMnemonic {
        let aligned_target: u12 = Self::align_opcode::<12>(raw_instruction);

        // Iterate over opcodes and check for a match
        for (opcode, mnemonic, _) in OPCODES {
            let aligned_opcode: u12;
            match opcode {
                NBit::N1(n) => aligned_opcode = Self::align_opcode::<1>(*n),
                NBit::N2(n) => aligned_opcode = Self::align_opcode::<2>(*n),
                NBit::N3(n) => aligned_opcode = Self::align_opcode::<3>(*n),
                NBit::N4(n) => aligned_opcode = Self::align_opcode::<4>(*n),
                NBit::N5(n) => aligned_opcode = Self::align_opcode::<5>(*n),
                NBit::N6(n) => aligned_opcode = Self::align_opcode::<6>(*n),
                NBit::N7(n) => aligned_opcode = Self::align_opcode::<7>(*n),
                NBit::N8(n) => aligned_opcode = Self::align_opcode::<8>(*n),
                NBit::N9(n) => aligned_opcode = Self::align_opcode::<9>(*n),
                NBit::N10(n) => aligned_opcode = Self::align_opcode::<10>(*n),
                NBit::N11(n) => aligned_opcode = Self::align_opcode::<11>(*n),
                NBit::N12(n) => aligned_opcode = Self::align_opcode::<12>(*n),
                _ => panic!("Invalid opcode length"),
            }

            if aligned_opcode == aligned_target {
                return *mnemonic;
            }
        }

        // If no match found after checking all patterns
        PICInstructionMnemonic::UND
    }

    pub fn extract_k(&self) -> u8{
        (self.instruction_raw.as_u16() & 0x0FF) as u8
    }

    pub fn extract_d(&self) -> NBitNumber<1>{
        NBitNumber::new(self.instruction_raw.as_u16() & 0x020)
    }

    pub fn extract_f(&self) -> NBitNumber<5>{
       NBitNumber::new((self.instruction_raw.as_u16() & 0x01F).into())
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
