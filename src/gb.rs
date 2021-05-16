
use crate::cpu::Cpu;
use crate::mmu::Mmu;
use crate::ppu::Ppu;
use crate::spu::Spu;
use crate::timer::Timer;
use crate::joypad::Joypad;

use crate::cpu::interrupt::Interrupt;
use crate::cpu::flags::Flags;
use crate::ppu::*;
use crate::joypad::Keys;
use crate::MemoryBus;

use sdl2::Sdl;
use sdl2::audio::{AudioSpecDesired, AudioQueue};
use sdl2::event::*;
use sdl2::keyboard::*;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::*;
use sdl2::video::*;

pub struct GameBoy {
    ticks: u64,

    // #region input
    joypad_pressed_keys: Keys,
    joypad_released_keys: Keys,
    // #endregion

    // #region hardware
    cpu: *mut Cpu,
    mmu: *mut Mmu,
    ppu: *mut Ppu,
    spu: *mut Spu,
    timer: *mut Timer,
    joypad: *mut Joypad,
    // #endregion

    // #region audio-output
    audio_channel_1: AudioQueue<i8>,
    audio_channel_2: AudioQueue<i8>,
    audio_channel_3: AudioQueue<i8>,
    audio_channel_4: AudioQueue<i8>,
    // #endregion

    // #region video-output
    window_canvas: Canvas<Window>,
    //window_texture_creator: TextureCreator<WindowContext>,
    window_texture: Texture,
    // #endregion
}

const TICKS_PER_SECOND: u64 = 4_194_304;
const TICKS_PER_FRAME:  u64 = TICKS_PER_SECOND / 60;

const BUTTON_A:      Keycode = Keycode::Space;
const BUTTON_B:      Keycode = Keycode::LShift;
const BUTTON_UP:     Keycode = Keycode::Up;
const BUTTON_DOWN:   Keycode = Keycode::Down;
const BUTTON_LEFT:   Keycode = Keycode::Left;
const BUTTON_RIGHT:  Keycode = Keycode::Right;
const BUTTON_START:  Keycode = Keycode::Return;
const BUTTON_SELECT: Keycode = Keycode::Backspace;

impl GameBoy {
    pub fn new(sdl: &Sdl) -> Self {
        // #region sdl
        let (audio_channel_1, audio_channel_2, audio_channel_3, audio_channel_4) = {
            let audio_subsystem = sdl.audio().unwrap();

            let spec = AudioSpecDesired { freq: Some(44_100), channels: Some(2), samples: Some(2048) };
            let ch1: AudioQueue<i8> = audio_subsystem.open_queue(None, &spec).unwrap();
            let ch2: AudioQueue<i8> = audio_subsystem.open_queue(None, &spec).unwrap();
            let ch3: AudioQueue<i8> = audio_subsystem.open_queue(None, &spec).unwrap();
            let ch4: AudioQueue<i8> = audio_subsystem.open_queue(None, &spec).unwrap();

            ch1.resume();
            ch2.resume();
            ch3.resume();
            ch4.resume();

            (ch1, ch2, ch3, ch4)
        };

        let window = {
            let scale = 4;
            let width = (SCREEN_PIXEL_WIDTH * scale) as u32;
            let height = (SCREEN_PIXEL_HEIGHT * scale) as u32;

            let video_subsystem = sdl.video().unwrap();

            video_subsystem
                .window("Kiwi", width, height)
                .position_centered()
                .build()
                .unwrap()
        };

        let window_canvas = window.into_canvas().build().unwrap();

        let window_texture: Texture = {
            window_canvas.texture_creator().create_texture(
                Some(PixelFormatEnum::ARGB32),
                TextureAccess::Static,
                SCREEN_PIXEL_WIDTH as u32,
                SCREEN_PIXEL_HEIGHT as u32,
            ).unwrap()
        };
        // #endregion

        // #region raw-ptr
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
        // #endregion

        unsafe {
            // #region mmap
            (*cpu).mmu = mmu;
            (*mmu).cpu = cpu;
            (*mmu).ppu = ppu;
            (*mmu).spu = spu;
            (*mmu).timer = timer;
            (*mmu).joypad = joypad;
            // #endregion

            // #region bios-skip
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
            // #endregion
        }

        let ticks = 0u64;

        Self {
            ticks,
            cpu,
            mmu,
            ppu,
            spu,
            timer,
            joypad,

            audio_channel_1,
            audio_channel_2,
            audio_channel_3,
            audio_channel_4,

            window_canvas,
            window_texture,

            joypad_pressed_keys: Keys::empty(),
            joypad_released_keys: Keys::empty(),
        }
    }

    pub fn load_rom(&mut self, rom: &Vec<u8>) {
        unsafe {
            for i in 0..rom.len() {
                (*self.mmu).cartridge_rom[i] = rom[i];
            }
        }
    }

    pub fn handle_event(&mut self, evt: &Event) {
        let window_canvas_id = self.window_canvas.window().id();
        match evt {
            Event::KeyDown { keycode: Some(keycode), repeat: false, window_id, ..} => {
                if *window_id == window_canvas_id {
                    let keys = match *keycode {
                        BUTTON_A      => Keys::A,
                        BUTTON_B      => Keys::B,
                        BUTTON_UP     => Keys::UP,
                        BUTTON_DOWN   => Keys::DOWN,
                        BUTTON_LEFT   => Keys::LEFT,
                        BUTTON_RIGHT  => Keys::RIGHT,
                        BUTTON_START  => Keys::START,
                        BUTTON_SELECT => Keys::SELECT,
                        _ => Keys::empty(),
                    };
                    self.joypad_pressed_keys.insert(keys);
                    self.joypad_released_keys.remove(keys);
                }
            }
            Event::KeyUp { keycode: Some(keycode), repeat: false, window_id, ..} => {
                if *window_id == window_canvas_id {
                    let keys = match *keycode {
                        BUTTON_A      => Keys::A,
                        BUTTON_B      => Keys::B,
                        BUTTON_UP     => Keys::UP,
                        BUTTON_DOWN   => Keys::DOWN,
                        BUTTON_LEFT   => Keys::LEFT,
                        BUTTON_RIGHT  => Keys::RIGHT,
                        BUTTON_START  => Keys::START,
                        BUTTON_SELECT => Keys::SELECT,
                        _ => Keys::empty(),
                    };
                    self.joypad_released_keys.insert(keys);
                    self.joypad_pressed_keys.remove(keys);
                }
            }
            _ => { }
        }
    }

    pub fn run_next_frame(&mut self) {
        unsafe {
            if !self.joypad_released_keys.is_empty() {
                (*self.joypad).release(self.joypad_released_keys);
                self.joypad_released_keys = Keys::empty();
            }
            if !self.joypad_pressed_keys.is_empty() {
                (*self.joypad).press(self.joypad_pressed_keys);
                (*self.cpu).request_interrupt(Interrupt::HL_PIN);
                self.joypad_pressed_keys = Keys::empty();
            }

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

            (*self.spu).enqueue_audio_samples(
                &mut self.audio_channel_1,
                &mut self.audio_channel_2,
                &mut self.audio_channel_3,
                &mut self.audio_channel_4);

            self.window_texture.update(None, (*self.ppu).frame_buffer(), SCREEN_BUFFER_WIDTH).unwrap();
            self.window_canvas.clear();
            self.window_canvas.copy(&mut self.window_texture, None, None).unwrap();
            self.window_canvas.present();
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