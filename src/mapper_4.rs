use memory::*;

pub struct Mapper4 {
    prg: Vec<u8>,
    prg_ram: Vec<u8>,
    chr: Vec<u8>,

    registers: [u8; 8],
    register_to_update: u8,
    prg_rom_bank_mode: bool,
    chr_inversion: bool,
}

impl Mapper4 {
    pub fn new(prg: Vec<u8>, prg_ram_size: usize, chr: Vec<u8>) -> Mapper4 {
        Mapper4 {
            prg: prg,
            prg_ram: vec![0; prg_ram_size],
            chr: chr,

            registers: [0; 8],
            register_to_update: 0,
            prg_rom_bank_mode: false,
            chr_inversion: false,
        }
    }
}

impl Mapper for Mapper4 {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x6000 ..= 0x7FFF => self.prg_ram[addr as usize - 0x6000],
            0x8000 ..= 0x9FFF => {
                let bank = if self.prg_rom_bank_mode {
                    self.prg.len() / 0x2000 - 2 // second-last bank
                } else {
                    self.registers[6] as usize
                };

                self.prg[bank * 0x2000 + addr as usize - 0x8000]
            },
            0xA000 ..= 0xBFFF => self.prg[self.registers[7] as usize * 0x2000 + addr as usize - 0xA000],
            0xC000 ..= 0xDFFF => {
                let bank = if !self.prg_rom_bank_mode {
                    self.prg.len() / 0x2000 - 2 // second-last bank
                } else {
                    self.registers[6] as usize
                };

                self.prg[bank * 0x2000 + addr as usize - 0x8000]
            },
            0xE000 ..= 0xFFFF => self.prg[self.prg.len() - 0x2000 + addr as usize - 0xE000], // Last bank
            _ => {
                panic!("Read from invalid mapper 4 address {:X}", addr);
            }
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x6000 ..= 0x7FFF => self.prg_ram[addr as usize - 0x6000] = val,
            0x8000 ..= 0x9FFE => {
                if addr%2 == 0 { //bank select
                    self.register_to_update = val&0b0000_0111;
                    self.prg_rom_bank_mode = (val&0b0100_0000) != 0;
                    self.chr_inversion = (val&0b1000_0000) != 0;

                } else { //Write
                    self.registers[self.register_to_update as usize] = val;
                }
            },
            0xA000 ..= 0xBFFF => {}
            0xC000 ..= 0xDFFF => {}
            0xE000 ..= 0xFFFF => {}
            _ => {
                panic!("Write to invalid mapper 4 address {:X}", addr);
            }
        }
    }

    fn read_ppu(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                let bank = if self.chr_inversion {
                    match addr {
                        0x0000 ..= 0x03FF => self.registers[2],
                        0x0400 ..= 0x07FF => self.registers[3],
                        0x0800 ..= 0x0BFF => self.registers[4],
                        0x0C00 ..= 0x0FFF => self.registers[5],
                        0x1000 ..= 0x13FF => self.registers[0]&0xFE,
                        0x1400 ..= 0x17FF => self.registers[0]|0x1,
                        0x1800 ..= 0x1BFF => self.registers[1]&0xFE,
                        0x1C00 ..= 0x1FFF => self.registers[1]|0x1,
                        _ => panic!()
                    }
                } else {
                    match addr {
                        0x0000 ..= 0x03FF => self.registers[0]&0xFE,
                        0x0400 ..= 0x07FF => self.registers[0]|0x1,
                        0x0800 ..= 0x0BFF => self.registers[1]&0xFE,
                        0x0C00 ..= 0x0FFF => self.registers[1]|0x1,
                        0x1000 ..= 0x13FF => self.registers[2],
                        0x1400 ..= 0x17FF => self.registers[3],
                        0x1800 ..= 0x1BFF => self.registers[4],
                        0x1C00 ..= 0x1FFF => self.registers[5],
                        _ => panic!()
                    }
                } as usize;
                let block = (addr as usize / 0x400) * 0x400;

                self.chr[bank * 0x400 + addr as usize - block]
            }
            _ => {
                panic!("Reference to invalid mapper 4 ppu address {:X}", addr);
            }
        }
    }

    fn write_ppu(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => {
                let bank = if self.chr_inversion {
                    match addr {
                        0x0000 ..= 0x03FF => self.registers[2],
                        0x0400 ..= 0x07FF => self.registers[3],
                        0x0800 ..= 0x0BFF => self.registers[4],
                        0x0C00 ..= 0x0FFF => self.registers[5],
                        0x1000 ..= 0x13FF => self.registers[0]&0xFE,
                        0x1400 ..= 0x17FF => self.registers[0]|0x1,
                        0x1800 ..= 0x1BFF => self.registers[1]&0xFE,
                        0x1C00 ..= 0x1FFF => self.registers[1]|0x1,
                        _ => panic!()
                    }
                } else {
                    match addr {
                        0x0000 ..= 0x03FF => self.registers[0]&0xFE,
                        0x0400 ..= 0x07FF => self.registers[0]|0x1,
                        0x0800 ..= 0x0BFF => self.registers[1]&0xFE,
                        0x0C00 ..= 0x0FFF => self.registers[1]|0x1,
                        0x1000 ..= 0x13FF => self.registers[2],
                        0x1400 ..= 0x17FF => self.registers[3],
                        0x1800 ..= 0x1BFF => self.registers[4],
                        0x1C00 ..= 0x1FFF => self.registers[5],
                        _ => panic!()
                    }
                } as usize;
                let block = (addr as usize / 0x400) * 0x400;

                self.chr[bank * 0x400 + addr as usize - block] = val;
            }
            _ => {
                panic!("Reference to invalid mapper 4 ppu address {:X}", addr);
            }
        }
    }
}