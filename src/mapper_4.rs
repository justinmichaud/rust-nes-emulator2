use memory::*;

pub struct Mapper4 {
    prg: Vec<u8>,
    prg_ram: Vec<u8>,
    chr: Vec<u8>,
}

impl Mapper4 {
    pub fn new(prg: Vec<u8>, prg_ram_size: usize, chr: Vec<u8>) -> Mapper4 {
        Mapper4 {
            prg: prg,
            prg_ram: vec![0; prg_ram_size],
            chr: chr,
        }
    }
}

impl Mapper for Mapper4 {
    fn read(&mut self, addr: u16) -> u8 {
        panic!("Reference to invalid mapper 4 address {:X}", addr);
    }

    fn write(&mut self, addr: u16, val: u8) {
        panic!("Reference to invalid mapper 4 address {:X}", addr);
    }

    fn read_ppu(&mut self, addr: u16) -> u8 {
        panic!("Reference to invalid mapper 4 address {:X}", addr);
    }

    fn write_ppu(&mut self, addr: u16, val: u8) {
        panic!("Reference to invalid mapper 4 address {:X}", addr);
    }
}