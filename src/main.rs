#[macro_use] extern crate bitflags;
#[macro_use] extern crate packed_struct;
extern crate pretty_env_logger;
#[macro_use] extern crate log;
extern crate sdl2;

pub mod bios;
pub mod cpu;
pub mod mmu;
pub mod ppu;
pub mod spu;
pub mod timer;
pub mod joypad;
pub mod gb;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::time::{Duration, Instant};

pub trait MemoryBus {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}

pub const TICKS_PER_SECOND: u64 = 4_194_304;
pub const TICKS_PER_FRAME:  u64 = TICKS_PER_SECOND / 60;

pub const BUTTON_A:      Keycode = Keycode::Space;
pub const BUTTON_B:      Keycode = Keycode::LShift;
pub const BUTTON_UP:     Keycode = Keycode::Up;
pub const BUTTON_DOWN:   Keycode = Keycode::Down;
pub const BUTTON_LEFT:   Keycode = Keycode::Left;
pub const BUTTON_RIGHT:  Keycode = Keycode::Right;
pub const BUTTON_START:  Keycode = Keycode::Return;
pub const BUTTON_SELECT: Keycode = Keycode::Backspace;

use gb::GameBoy;

const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / 60);

fn main() {
    pretty_env_logger::init();

    let sdl_context = sdl2::init().unwrap();
    let mut gameboy = GameBoy::new(&sdl_context);

    let args: Vec<String> = std::env::args().collect();
    let rom = std::fs::read(&args[1]).unwrap();
    gameboy.load_rom(&rom);

    let mut frame_begin_timestamp = Instant::now();
    let mut frame_overslept_duration = Duration::from_nanos(0);

    let mut event_pump = sdl_context.event_pump().unwrap();
    'gameloop: loop {
        gameboy.run_next_frame();

        for event in event_pump.poll_iter() {
            gameboy.handle_event(&event);
            match event {
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } | Event::Quit { .. } => break 'gameloop,
                _ => {}
            }
        }

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
                warn!("Frame overrun {:?} {:?}", frame_busy_duration, frame_overslept_duration);
                frame_begin_timestamp = frame_complete_timestamp;
                frame_overslept_duration = Duration::from_nanos(0);
            }
        }
    }
}
