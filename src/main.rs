#[macro_use]
extern crate bitflags;
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::TextureAccess;
use std::time::{Duration, Instant};

pub trait MemoryBus {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}

pub const TICKS_PER_SECOND: u64 = 4_194_304;
pub const TICKS_PER_FRAME:  u64 = TICKS_PER_SECOND / 60;

pub mod bios;
pub mod cpu;
pub mod mmu;
pub mod ppu;
pub mod timer;

use ppu::{SCREEN_PIXEL_HEIGHT, SCREEN_PIXEL_WIDTH, SCREEN_BUFFER_WIDTH};
use cpu::Cpu;
use mmu::Mmu;

const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / 60);

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let scale = 4;
    let width = (SCREEN_PIXEL_WIDTH * scale) as u32;
    let height = (SCREEN_PIXEL_HEIGHT * scale) as u32;

    let window = video_subsystem
        .window("Kiwi", width, height)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture(
            Some(PixelFormatEnum::ARGB32),
            TextureAccess::Static,
            SCREEN_PIXEL_WIDTH as u32,
            SCREEN_PIXEL_HEIGHT as u32,
        )
        .unwrap();

    let mut frame_begin_timestamp = Instant::now();
    let mut frame_overslept_duration = Duration::from_nanos(0);

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut frame_counter: u64 = 0;

    let cpu = Box::new(Cpu::default());
    let cpu: *mut Cpu = Box::into_raw(cpu);

    let mmu = Box::new(Mmu::default());
    let mmu: *mut Mmu = Box::into_raw(mmu);

    unsafe {
        (*cpu).mmu = mmu;
        (*mmu).cpu = cpu;

        (*cpu).r.set_flags(cpu::flags::Flags::Z | cpu::flags::Flags::H | cpu::flags::Flags::C);
        (*cpu).r.set_a(0x01);
        (*cpu).r.set_f(0xb0);
        (*cpu).r.set_bc(0x0013);
        (*cpu).r.set_de(0x00d8);
        (*cpu).r.set_hl(0x014d);
        (*cpu).r.set_sp(0xfffe);
        (*cpu).r.set_pc(0x0100);

        (*mmu).write(0xff05, 0x00);
        (*mmu).write(0xff06, 0x00);
        (*mmu).write(0xff07, 0x00);
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

        let args: Vec<String> = std::env::args().collect();
        let rom = std::fs::read(&args[1]).unwrap();
        for i in 0..rom.len() {
            (*mmu).cartridge_rom[i] = rom[i];
        }

        //for i in 0..bios::DMG_BIOS.len() {
        //    (*mmu).cartridge_rom[i] = bios::DMG_BIOS[i];
        //}
    }

    let mut ticks_counter: u64 = 0;

    'gameloop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => break 'gameloop,
                _ => {}
            }
        }

        unsafe {
            while ticks_counter < TICKS_PER_FRAME {
                let ticks = (*cpu).cycle();
                ticks_counter += ticks;

                (*mmu).timer.step(ticks);
                if (*mmu).timer.overflow_interrupt_requested() {
                    (*cpu).if_reg.set_timer_overflow();
                }
                (*mmu).ppu.step(ticks);
                if (*mmu).ppu.lcdc_status_interrupt_requested() {
                    (*cpu).if_reg.set_lcdc_status();
                }
                if (*mmu).ppu.vertical_blank_interrupt_requested() {
                    (*cpu).if_reg.set_vertical_blank();
                }
            }
            ticks_counter -= TICKS_PER_FRAME;

            texture.update(None, (*mmu).ppu.frame_buffer(), SCREEN_BUFFER_WIDTH).unwrap();
            frame_counter += 1;
        }

        canvas.clear();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        let frame_complete_timestamp = Instant::now();
        let frame_busy_duration = frame_complete_timestamp - frame_begin_timestamp;

        match FRAME_DURATION.checked_sub(frame_busy_duration + frame_overslept_duration) {
            Some(frame_wait_duration) => {
                std::thread::sleep(frame_wait_duration);
                frame_begin_timestamp = Instant::now();
                frame_overslept_duration =
                    (frame_begin_timestamp - frame_complete_timestamp) - frame_wait_duration;
            }
            None => {
                println!(
                    "Frame overrun {:?} {:?} {:?}",
                    frame_counter, frame_busy_duration, frame_overslept_duration
                );
                frame_begin_timestamp = frame_complete_timestamp;
                frame_overslept_duration = Duration::from_nanos(0);
            }
        }
    }

    unsafe {
        drop(Box::from_raw(cpu));
        drop(Box::from_raw(mmu));
    }
}
