use crate::MemoryBus;
use crate::cpu::Cpu;
use crate::ppu::Ppu;
use crate::timer::Timer;

use std::ptr;

#[derive(Debug)]
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

    pub timer: Box<Timer>,

    pub ppu: Box<Ppu>,

    pub cpu: *mut Cpu,
}

impl Default for Mmu {
    fn default() -> Self {
        Self {
            cartridge_rom: Box::new([0; 0x8000]),
            cartridge_ram: Box::new([0; 0x2000]),
            ram: Box::new([0; 0x2000 + 127]),
            timer: Box::new(Timer::default()),
            ppu: Box::new(Ppu::default()),
            cpu: ptr::null_mut(),
        }
    }
}

impl MemoryBus for Mmu {
    fn read(&self, addr: u16) -> u8 {
        if addr < 0x8000 {        // 0x0000..=0x7FFF (Cartridge ROM)
            self.cartridge_rom[addr as usize]
        } else if addr < 0xA000 { // 0x8000..=0x9FFF (Video RAM)
            self.ppu.read(addr)
        } else if addr < 0xC000 { // 0xA000..=0xBFFF (Cartridge RAM)
            self.cartridge_ram[(addr as usize - 0xA000)]
        } else if addr < 0xE000 { // 0xC000..=0xDFFF (Internal RAM)
            self.ram[addr as usize - 0xC000]
        } else if addr < 0xFE00 { // 0xE000..=0xFDFF (Echo RAM)
            self.ram[(addr - 0xE000) as usize]
        } else if addr < 0xFEA0 { // 0xFE00..=0xFE9F (OAM)
            0
        } else if addr < 0xFF00 { // 0xFEA0..=0xFEFF (Unusable)
            0
        } else if addr < 0xFF80 { // 0xFF00..=0xFF7F (Hardware IO)
            match addr {
                // Joypad
                0xFF00 => 0x3F,

                // Serial
                0xFF01 => 0xFF,
                0xFF02 => 0x03,

                // Timer
                0xFF04..=0xFF07 =>self.timer.read(addr),

                // PPU
                0xFF40..=0xFF4B => self.ppu.read(addr),

                // CPU
                0xFF0F => unsafe { (*self.cpu).read(addr) }
                _ => 0
            }
        } else if addr < 0xFFFF { // 0xFF80..=0xFFFE (Zero Page)
            self.ram[0x2000 + (addr - 0xFF80) as usize]
        } else {
            unsafe { (*self.cpu).read(addr) }
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if addr < 0x8000 {        // 0x0000..=0x7FFF (Cartridge ROM)
            // println!("crom write {:02X} => {:04X}", data, addr);
            // read-only, but writting to it configures the rom bank switch
            self.cartridge_rom[addr as usize] = data;
        } else if addr < 0xA000 { // 0x8000..=0x9FFF (Video RAM)
            self.ppu.write(addr, data)
        } else if addr < 0xC000 { // 0xA000..=0xBFFF (Cartridge RAM)
            self.cartridge_ram[addr as usize - 0xA000] = data;
        } else if addr < 0xE000 { // 0xC000..=0xDFFF (Internal RAM)
            self.ram[(addr as usize - 0xC000)] = data
        } else if addr < 0xFE00 { // 0xE000..=0xFDFF (Echo RAM)
            self.ram[(addr as usize - 0xE000)] = data
        } else if addr < 0xFEA0 { // 0xFE00..=0xFE9F (OAM)
            self.ppu.write(addr, data)
        } else if addr < 0xFF00 { // 0xFEA0..=0xFEFF (Unusable)
        } else if addr < 0xFF80 { // 0xFF00..=0xFF7F (Hardware IO)
            match addr {
                // CPU
                0xFF0F => unsafe { (*self.cpu).write(addr, data); }

                // Timer
                0xFF04..=0xFF07 => self.timer.write(addr, data),

                // DMA
                0xFF46 => {
                    if data <= 0xF1 {
                        let addr = u16::from_be_bytes([data, 0x00]);

                        let mut oam: [u8; 160] = [0; 160];
                        for i in 0..160 {
                            oam[i] = self.read(addr + i as u16)
                        }

                        self.ppu.populate_object_attribute_ram(&oam);
                    }
                }

                // PPU
                0xFF40..=0xFF4B => self.ppu.write(addr, data),

                _ => { }
            }
        } else if addr < 0xFFFF { // 0xFF80..=0xFFFE (Zero Page)
            self.ram[0x2000 + (addr - 0xFF80) as usize] = data
        } else {
            unsafe { (*self.cpu).write(addr, data) }
        }
    }
}