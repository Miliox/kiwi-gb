#[macro_use]
extern crate bitflags;

pub trait MemoryBus {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}

pub mod cpu;
pub mod mmu;
pub mod ppu;
pub mod timer;

fn main() {
    println!("Hello, world!");
}
