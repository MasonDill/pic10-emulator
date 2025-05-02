//https://web.archive.org/web/20150811030147/http://ww1.microchip.com/downloads/en/DeviceDoc/41239D.pdf
//http://ww1.microchip.com/downloads/en/DeviceDoc/41228C.pdf

//12 bit word

//SPECIAL REGISTERS
//K is a 8 or 9 bit constant (depending on the instruction type)
//C is the carry flag
//Z is the zero flag


//declare state

// pub mod test;

// let PC = //should point to bootloader in ROM


#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
pub mod pic;
pub mod program_memory;
pub mod data_memory;
pub mod logic;
// Tests module
pub mod tests;