
use crate::cpu::Cpu;
use crate::mmu::Mmu;
use crate::ppu::Ppu;
use crate::spu::Spu;
use crate::timer::Timer;
use crate::joypad::Joypad;

use crate::cpu::interrupt::Interrupt;
use crate::cpu::flags::Flags;
use crate::ppu::SCREEN_BUFFER_WIDTH;
use crate::joypad::Keys;
use crate::MemoryBus;

use sdl2::audio::AudioQueue;
use sdl2::render::Texture;

pub struct GameBoy {
    ticks: u64,
    cpu: *mut Cpu,
    mmu: *mut Mmu,
    ppu: *mut Ppu,
    spu: *mut Spu,
    timer: *mut Timer,
    joypad: *mut Joypad,
}

const TICKS_PER_SECOND: u64 = 4_194_304;

const TICKS_PER_FRAME:  u64 = TICKS_PER_SECOND / 60;

impl GameBoy {
    pub fn new() -> Self {
        let ticks = 0u64;

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

        let joypad = Box::new(Joypad::default());
        let joypad: *mut Joypad = Box::into_raw(joypad);

        unsafe {
            (*cpu).mmu = mmu;
            (*mmu).cpu = cpu;
            (*mmu).ppu = ppu;
            (*mmu).spu = spu;
            (*mmu).timer = timer;
            (*mmu).joypad = joypad;

            (*cpu).regs.set_flags(Flags::Z | Flags::H | Flags::C);
            (*cpu).regs.set_a(0x01);
            (*cpu).regs.set_f(0xb0);
            (*cpu).regs.set_bc(0x0013);
            (*cpu).regs.set_de(0x00d8);
            (*cpu).regs.set_hl(0x014d);
            (*cpu).regs.set_sp(0xfffe);
            (*cpu).regs.set_pc(0x0100);

            (*mmu).write(0xff05, 0x00);
            (*mmu).write(0xff06, 0x00);
            (*mmu).write(0xff07, 0x00);
            /*
            (*mmu).write(0xff10, 0x80);
            (*mmu).write(0xff11, 0xbf);
            (*mmu).write(0xff12, 0xf3);
            (*mmu).write(0xff14, 0xbf);
            (*mmu).write(0xff16, 0x3f);
            (*mmu).write(0xff17, 0x00);
            (*mmu).write(0xff19, 0xbf);
            (*mmu).write(0xff1a, 0x7f);
            (*mmu).write(0xff1b, 0xff);
            (*mmu).write(0xff1c, 0x9f);
            (*mmu).write(0xff1e, 0xbf);
            (*mmu).write(0xff20, 0xff);
            (*mmu).write(0xff21, 0x00);
            (*mmu).write(0xff22, 0x00);
            (*mmu).write(0xff23, 0xbf);
            (*mmu).write(0xff24, 0x77);
            (*mmu).write(0xff25, 0xf3);
            (*mmu).write(0xff26, 0xf1);
            */
            (*mmu).write(0xff40, 0x91);
            (*mmu).write(0xff42, 0x00);
            (*mmu).write(0xff43, 0x00);
            (*mmu).write(0xff45, 0x00);
            (*mmu).write(0xff47, 0xfc);
            (*mmu).write(0xff48, 0xff);
            (*mmu).write(0xff49, 0xff);
            (*mmu).write(0xff4a, 0x00);
            (*mmu).write(0xff4b, 0x00);
            (*mmu).write(0xffff, 0x00);
        }

        Self { ticks, cpu, mmu, ppu, spu, timer, joypad }
    }

    pub fn load_rom(&mut self, rom: &Vec<u8>) {
        unsafe {
            for i in 0..rom.len() {
                (*self.mmu).cartridge_rom[i] = rom[i];
            }
        }
    }

    pub fn run_next_frame(&mut self) {
        unsafe {
            while self.ticks < TICKS_PER_FRAME {
                let ticks = (*self.cpu).cycle();
                self.ticks += ticks;

                (*self.timer).step(ticks);
                (*self.ppu).step(ticks);

                if (*self.timer).overflow_interrupt_requested() {
                    (*self.cpu).request_interrupt(Interrupt::TIMER);
                }
                if (*self.ppu).lcdc_status_interrupt_requested() {
                    (*self.cpu).request_interrupt(Interrupt::LCDC);
                }
                if (*self.ppu).vertical_blank_interrupt_requested() {
                    (*self.cpu).request_interrupt(Interrupt::VBLANK);
                }
            }
            self.ticks -= TICKS_PER_FRAME;
        }
    }

    pub fn sync_audio(&mut self, ch1: &mut AudioQueue<i8>, ch2: &mut AudioQueue<i8>, ch3: &mut AudioQueue<i8>, ch4: &mut AudioQueue<i8>) {
        unsafe {
            (*self.spu).enqueue_audio_samples(ch1, ch2, ch3, ch4);
        }
    }

    pub fn sync_video(&mut self, texture: &mut Texture) {
        unsafe {
            texture.update(None, (*self.ppu).frame_buffer(), SCREEN_BUFFER_WIDTH).unwrap();
        }
    }

    pub fn sync_pad(&mut self, presses: Keys, releases: Keys) {
        unsafe {
            if !presses.is_empty() {
                (*self.joypad).press(presses);
            }
            if !releases.is_empty() {
                (*self.joypad).release(releases);
                (*self.cpu).request_interrupt(Interrupt::HL_PIN);
            }
        }
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
            drop(Box::from_raw(self.joypad));
        }
    }
}