use crate::cpu::Cpu;
use std::ptr;

#[derive(Clone, Debug)]
pub struct Mmu {
    // Cartrige ROM
    // - $0000..=$7FFFF (ROM)
    pub cartridge_rom: Box<[u8; 0x8000]>,

    // Cartrige RAM
    // - $A000..=$BFFFF (RAM)
    pub cartridge_ram: Box<[u8; 0x2000]>,

    // Random Access Memory
    // - $C000..=$DFFF (Internal RAM)
    // - $E000..=$FDFF (Echo of Internal RAM)
    // - $FF80..=$FFFE (Zero Page)
    pub ram: Box<[u8; 0x2000 + 127]>,

    pub cpu: *mut Cpu,
}

impl Default for Mmu {
    fn default() -> Self {
        Self {
            cartridge_rom: Box::new([0; 0x8000]),
            cartridge_ram: Box::new([0; 0x2000]),
            ram: Box::new([0; 0x2000 + 127]),
            cpu: ptr::null_mut(),
        }
    }
}

impl Mmu {
    pub fn read(&self, addr: u16) -> u8 {
        if addr < 0x8000 {        // 0x0000..=0x7FFF (Cartridge ROM)
            self.cartridge_rom[addr as usize]
        } else if addr < 0xA000 { // 0x8000..=0x9FFF (Video RAM)
            0
        } else if addr < 0xC000 { // 0xA000..=0xBFFF (Cartridge RAM)
            self.cartridge_ram[(addr - 0xA000) as usize]
        } else if addr < 0xE000 { // 0xC000..=0xDFFF (Internal RAM)
            self.ram[(addr - 0xC000) as usize]
        } else if addr < 0xFE00 { // 0xE000..=0xFDFF (Echo RAM)
            self.ram[(addr - 0xE000) as usize]
        } else if addr < 0xFEA0 { // 0xFE00..=0xFE9F (OAM)
            0
        } else if addr < 0xFF00 { // 0xFEA0..=0xFEFF (Unusable)
            0
        } else if addr < 0xFF80 { // 0xFF00..=0xFF7F (Hardware IO)
            match addr {
                // CPU
                0xFF0F => unsafe {
                    (*self.cpu).interruption_flag()
                }
                _ => 0
            }
        } else if addr < 0xFFFF { // 0xFF80..=0xFFFE (Zero Page)
            self.ram[0x2000 + (addr - 0xFF80) as usize]
        } else {
            unsafe {
                (*self.cpu).interruption_enable_register()
            }
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        let addr = addr as usize;
        if addr < 0x8000 {        // 0x0000..=0x7FFF (Cartridge ROM)
            // println!("crom write {:02X} => {:04X}", data, addr);
            // read-only, but writting to it configures the rom bank switch
            self.cartridge_rom[addr] = data;
        } else if addr < 0xA000 { // 0x8000..=0x9FFF (Video RAM)
        } else if addr < 0xC000 { // 0xA000..=0xBFFF (Cartridge RAM)
            self.cartridge_ram[addr - 0xA000] = data;
        } else if addr < 0xE000 { // 0xC000..=0xDFFF (Internal RAM)
            self.ram[(addr - 0xC000)] = data
        } else if addr < 0xFE00 { // 0xE000..=0xFDFF (Echo RAM)
            self.ram[(addr - 0xE000)] = data
        } else if addr < 0xFEA0 { // 0xFE00..=0xFE9F (OAM)
        } else if addr < 0xFF00 { // 0xFEA0..=0xFEFF (Unusable)
        } else if addr < 0xFF80 { // 0xFF00..=0xFF7F (Hardware IO)
            match addr {
                // CPU
                0xFF0F => unsafe {
                    (*self.cpu).set_interruption_flag(data);
                }
                _ => { }
            }
        } else if addr < 0xFFFF { // 0xFF80..=0xFFFE (Zero Page)
            self.ram[0x2000 + (addr - 0xFF80) as usize] = data
        } else {
            unsafe {
                (*self.cpu).set_interruption_enable_register(data)
            }
        }
    }
}