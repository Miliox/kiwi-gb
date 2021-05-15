
use crate::cpu::Cpu;
use crate::mmu::Mmu;
use crate::ppu::Ppu;
use crate::spu::Spu;
use crate::timer::Timer;

struct GameBoy {
    cpu: *mut Cpu,
    mmu: *mut Mmu,
    ppu: *mut Ppu,
    spu: *mut Spu,
    timer: *mut Timer,
}

impl GameBoy {
    pub fn new() -> Self {
        let cpu = Box::new(Cpu::default());
        let cpu: *mut Cpu = Box::into_raw(cpu);

        let mmu = Box::new(Mmu::default());
        let mmu: *mut Mmu = Box::into_raw(mmu);

        let ppu = Box::new(Ppu::default());
        let ppu: *mut Ppu = Box::into_raw(ppu);

        let spu = Box::new(Spu::default());
        let spu: *mut Spu = Box::into_raw(spu);

        let timer = Box::new(Timer::default());
        let timer: *mut Timer = Box::into_raw(timer);

        Self { cpu, mmu, ppu, spu, timer }
    }
}

impl Drop for GameBoy {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.mmu));
            drop(Box::from_raw(self.cpu));
            drop(Box::from_raw(self.ppu));
            drop(Box::from_raw(self.spu));
            drop(Box::from_raw(self.timer));
        }
    }
}