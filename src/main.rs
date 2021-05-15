#[macro_use] extern crate bitflags;
extern crate pretty_env_logger;
#[macro_use] extern crate log;
extern crate sdl2;

use sdl2::audio::{AudioSpecDesired, AudioQueue};
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
pub mod spu;
pub mod timer;
pub mod gb;

use gb::GameBoy;
use ppu::{SCREEN_PIXEL_HEIGHT, SCREEN_PIXEL_WIDTH};

const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / 60);

fn main() {
    pretty_env_logger::init();
    let sdl_context = sdl2::init().unwrap();

    let (mut audio_ch1, mut audio_ch2, mut audio_ch3, mut audio_ch4) = {
        let audio_subsystem = sdl_context.audio().unwrap();

        let audio_spec = AudioSpecDesired { freq: Some(44_100), channels: Some(2), samples: Some(2048) };
        let audio_ch1: AudioQueue<i8> = audio_subsystem.open_queue(None, &audio_spec).unwrap();
        let audio_ch2: AudioQueue<i8> = audio_subsystem.open_queue(None, &audio_spec).unwrap();
        let audio_ch3: AudioQueue<i8> = audio_subsystem.open_queue(None, &audio_spec).unwrap();
        let audio_ch4: AudioQueue<i8> = audio_subsystem.open_queue(None, &audio_spec).unwrap();

        audio_ch1.resume();
        audio_ch2.resume();
        audio_ch3.resume();
        audio_ch4.resume();

        (audio_ch1, audio_ch2, audio_ch3, audio_ch4)
    };

    let window = {
        let scale = 4;
        let width = (SCREEN_PIXEL_WIDTH * scale) as u32;
        let height = (SCREEN_PIXEL_HEIGHT * scale) as u32;

        let video_subsystem = sdl_context.video().unwrap();

        video_subsystem
            .window("Kiwi", width, height)
            .position_centered()
            .build()
            .unwrap()
    };

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

    let mut gameboy = GameBoy::new();

    let args: Vec<String> = std::env::args().collect();
    let rom = std::fs::read(&args[1]).unwrap();
    gameboy.load_rom(&rom);

    let mut event_pump = sdl_context.event_pump().unwrap();
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

        gameboy.run_next_frame();
        gameboy.sync_audio(&mut audio_ch1, &mut audio_ch2, &mut audio_ch3, &mut audio_ch4);
        gameboy.sync_video(&mut texture);

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
                warn!("Frame overrun {:?} {:?}", frame_busy_duration, frame_overslept_duration);
                frame_begin_timestamp = frame_complete_timestamp;
                frame_overslept_duration = Duration::from_nanos(0);
            }
        }
    }
}
