pub mod lcd_control;
pub mod lcd_control_status;
pub mod palette;
pub mod sprite;

use lcd_control::LcdControl;
use lcd_control_status::LcdControlStatus;
use lcd_control_status::LcdControlMode;
use palette::Palette;
use sprite::Sprite;

use crate::MemoryBus;

use sdl2::pixels::Color;

pub const SCREEN_PIXEL_WIDTH:  usize = 160;
pub const SCREEN_PIXEL_HEIGHT: usize = 144;
pub const SCREEN_PIXEL_SIZE: usize  = SCREEN_PIXEL_HEIGHT * SCREEN_PIXEL_WIDTH;

pub const PIXEL_BIT_DEPTH: usize = 2;
pub const ARGB_BYTES_PER_PIXEL: usize = 4;
pub const SCREEN_BUFFER_SIZE: usize = SCREEN_PIXEL_SIZE * ARGB_BYTES_PER_PIXEL;
pub const SCREEN_BUFFER_WIDTH: usize = SCREEN_PIXEL_WIDTH * ARGB_BYTES_PER_PIXEL;

pub const SHADE_0: Color = Color::RGB(0x9B, 0xBC, 0x0F); // Light
pub const SHADE_1: Color = Color::RGB(0x8B, 0xAC, 0x0F); // Light Gray
pub const SHADE_2: Color = Color::RGB(0x30, 0x62, 0x30); // Dark Gray
pub const SHADE_3: Color = Color::RGB(0x0F, 0x38, 0x0F); // Dark
pub const SHADE: [Color; 4] = [SHADE_0, SHADE_1, SHADE_2, SHADE_3];

pub const TILE_SIZE: usize = 16;
pub const TILE_WIDTH: usize = 8;
pub const TILE_HEIGHT: usize = 8;
pub const TILE_PER_ROW: usize = 32;

#[allow(dead_code)]
pub const TILE_PER_COL: usize = 32;

#[derive(Debug)]
pub struct Ppu {
    lcdc: LcdControl,
    stat: LcdControlStatus,

    scanline: u8,
    scanline_compare: u8,

    scroll_y: u8,
    scroll_x: u8,

    window_y: u8,
    window_x: u8,

    background_palette: Palette,

    object_palette_0: Palette,
    object_palette_1: Palette,

    ticks: u64,
    lcdc_status_interrupt_requested: bool,
    vertical_blank_interrupt_requested: bool,

    back_buffer_index: usize,
    front_buffer_index: usize,
    frame_buffer: [Box<[u8; SCREEN_BUFFER_SIZE]>; 2],

    object_attribute_ram: Box<[Sprite; 40]>,
    video_ram: Box<[u8; 0x2000]>,
}

impl Default for Ppu {
    fn default() -> Self {
        let mut blank_frame: [u8; SCREEN_BUFFER_SIZE] = [0; SCREEN_BUFFER_SIZE];
        for index in 0..blank_frame.len() {
            match index % 4 {
                0 => blank_frame[index] = SHADE_0.a, // A
                1 => blank_frame[index] = SHADE_0.r, // R
                2 => blank_frame[index] = SHADE_0.g, // G
                3 => blank_frame[index] = SHADE_0.b, // B
                _ => panic!(),
            }
        }

        Self {
            lcdc: LcdControl::default(),
            stat: LcdControlStatus::default(),

            scanline: 0,
            scanline_compare: 0,

            scroll_y: 0,
            scroll_x: 0,

            window_x: 0,
            window_y: 0,

            background_palette: Palette::default(),
            object_palette_0: Palette::default(),
            object_palette_1: Palette::default(),

            ticks: 0,
            lcdc_status_interrupt_requested: false,
            vertical_blank_interrupt_requested: false,

            back_buffer_index: 0,
            front_buffer_index: 1,
            frame_buffer: [Box::new(blank_frame), Box::new(blank_frame)],

            object_attribute_ram: Box::new([Sprite::default(); 40]),
            video_ram: Box::new([0; 0x2000]),
        }
    }
}

impl MemoryBus for Ppu {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9FFF => self.video_ram[addr as usize - 0x8000],
            0xFE00..=0xFE9F => {
                let sprite_index = addr as usize / 4;
                let sprite_field = addr % 4;
                match sprite_field {
                    0 => self.object_attribute_ram[sprite_index].y(),
                    1 => self.object_attribute_ram[sprite_index].x(),
                    2 => self.object_attribute_ram[sprite_index].tile(),
                    3 => self.object_attribute_ram[sprite_index].flags(),
                    _ => panic!()
                }
            },
            0xFF40 => self.lcdc(),
            0xFF41 => self.stat(),
            0xFF42 => self.scroll_y(),
            0xFF43 => self.scroll_x(),
            0xFF44 => self.scanline(),
            0xFF45 => self.scanline_compare(),
            0xFF47 => self.background_palette(),
            0xFF48 => self.object_palette_0(),
            0xFF49 => self.object_palette_1(),
            0xFF4A => self.window_y(),
            0xFF4B => self.window_x(),
            _ => 0
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x8000..=0x9FFF => { self.video_ram[addr as usize - 0x8000] = data }
            0xFE00..=0xFE9F => {
                let addr = addr as usize - 0xFE00;
                let sprite_index = addr / 4;
                let sprite_field = addr % 4;
                match sprite_field {
                    0 => self.object_attribute_ram[sprite_index].set_y(data),
                    1 => self.object_attribute_ram[sprite_index].set_x(data),
                    2 => self.object_attribute_ram[sprite_index].set_tile(data),
                    3 => self.object_attribute_ram[sprite_index].set_flags(data),
                    _ => panic!()
                }
            },
            0xFF40 => self.set_lcdc(data),
            0xFF41 => self.set_stat(data),
            0xFF42 => self.set_scroll_y(data),
            0xFF43 => self.set_scroll_x(data),
            0xFF45 => self.set_scanline_compare(data),
            0xFF47 => self.set_background_palette(data),
            0xFF48 => self.set_object_palette_0(data),
            0xFF49 => self.set_object_palette_1(data),
            0xFF4A => self.set_window_y(data),
            0xFF4B => self.set_window_x(data),
            _ => { }
        }
    }
}

#[allow(dead_code)]
impl Ppu {
    pub fn lcdc(&self) -> u8 {
        self.lcdc.into()
    }

    pub fn set_lcdc(&mut self, lcdc: u8) {
        self.lcdc = LcdControl::from(lcdc);
    }

    pub fn stat(&self) -> u8 {
        self.stat.into()
    }

    pub fn set_stat(&mut self, stat: u8) {
        self.stat = (LcdControlStatus::from(stat) & !LcdControlStatus::READ_ONLY_MASK)  | (self.stat & LcdControlStatus::READ_ONLY_MASK);
        self.stat.set_scanline_coincidence(self.scanline == self.scanline_compare);
    }

    pub fn mode(&self) -> LcdControlMode {
        self.stat.mode()
    }

    pub fn set_mode(&mut self, mode: LcdControlMode) {
        self.stat.set_mode(mode);

        if mode == LcdControlMode::ScanningOAM && self.stat.contains(LcdControlStatus::MODE_OAM_INTERRUPT_ENABLE) {
            self.lcdc_status_interrupt_requested = true;
        }

        if mode == LcdControlMode::VerticalBlank && self.stat.contains(LcdControlStatus::MODE_V_BLANK_INTERRUPT_ENABLE) {
            self.lcdc_status_interrupt_requested = true;
        }

        if mode == LcdControlMode::HorizontalBlank && self.stat.contains(LcdControlStatus::MODE_H_BLANK_INTERRUPT_ENABLE) {
            self.lcdc_status_interrupt_requested = true;
        }

        self.vertical_blank_interrupt_requested = mode == LcdControlMode::VerticalBlank;
    }

    pub fn scanline(&self) -> u8 {
        self.scanline
    }

    fn increment_scanline(&mut self) {
        self.scanline += 1;
        self.stat.set_scanline_coincidence(self.scanline == self.scanline_compare);

        if self.stat.contains(LcdControlStatus::LINE_Y_COINCIDENCE_INTERRUPT_ENABLE) {
            self.lcdc_status_interrupt_requested = true;
        }
    }

    fn reset_scanline(&mut self) {
        self.scanline = 0;
        self.stat.set_scanline_coincidence(self.scanline == self.scanline_compare);

        if self.stat.contains(LcdControlStatus::LINE_Y_COINCIDENCE_INTERRUPT_ENABLE) {
            self.lcdc_status_interrupt_requested = true;
        }
    }

    pub fn scanline_compare(&self) -> u8 {
        self.scanline_compare
    }

    pub fn set_scanline_compare(&mut self, lyc: u8) {
        self.scanline_compare = lyc;
    }

    pub fn background_palette(&self) -> u8 {
        self.background_palette.into()
    }

    pub fn set_background_palette(&mut self, bgp: u8) {
        self.background_palette = bgp.into();
    }

    pub fn object_palette_0(&self) -> u8 {
        self.object_palette_0.into()
    }

    pub fn set_object_palette_0(&mut self, obp0: u8) {
        self.object_palette_0 = obp0.into();
    }

    pub fn object_palette_1(&self) -> u8 {
        self.object_palette_1.into()
    }

    pub fn set_object_palette_1(&mut self, obp1: u8) {
        self.object_palette_1 = obp1.into();
    }

    pub fn scroll_x(&self) -> u8 {
        self.scroll_x
    }

    pub fn set_scroll_x(&mut self, scroll_x: u8) {
        self.scroll_x = scroll_x;
    }

    pub fn scroll_y(&self) -> u8 {
        self.scroll_y
    }

    pub fn set_scroll_y(&mut self, scroll_y: u8) {
        self.scroll_y = scroll_y;
    }

    pub fn window_x(&self) -> u8 {
        self.window_x
    }

    pub fn set_window_x(&mut self, window_x: u8) {
        self.window_x = window_x;
    }

    pub fn window_y(&self) -> u8 {
        self.window_y
    }

    pub fn set_window_y(&mut self, window_y: u8) {
        self.window_y = window_y;
    }

    pub fn lcdc_status_interrupt_requested(&self) -> bool {
        self.lcdc_status_interrupt_requested
    }

    pub fn vertical_blank_interrupt_requested(&self) -> bool {
        self.vertical_blank_interrupt_requested
    }

    pub fn read_video_ram(&self, addr: u16) -> u8 {
        self.video_ram[addr as usize]
    }

    pub fn write_video_ram(&mut self, addr: u16, data: u8) {
        self.video_ram[addr as usize] = data;
    }

    pub fn frame_buffer(&self) -> &[u8; SCREEN_BUFFER_SIZE] {
        &self.frame_buffer[self.front_buffer_index]
    }

    pub fn read_object_attribute_ram(&self, addr: u16) -> u8 {
        let sprite_index = addr as usize / 4;
        let sprite_field = addr % 4;
        match sprite_field {
            0 => self.object_attribute_ram[sprite_index].y(),
            1 => self.object_attribute_ram[sprite_index].x(),
            2 => self.object_attribute_ram[sprite_index].tile(),
            3 => self.object_attribute_ram[sprite_index].flags(),
            _ => panic!()
        }
    }

    pub fn write_object_attribute_ram(&mut self, addr: u16, data: u8) {
        let sprite_index = addr as usize / 4;
        let sprite_field = addr % 4;
        match sprite_field {
            0 => self.object_attribute_ram[sprite_index].set_y(data),
            1 => self.object_attribute_ram[sprite_index].set_x(data),
            2 => self.object_attribute_ram[sprite_index].set_tile(data),
            3 => self.object_attribute_ram[sprite_index].set_flags(data),
            _ => panic!()
        }
    }

    pub fn populate_object_attribute_ram(&mut self, data: &[u8; 160]) {
        for sprite_index in 0..40 {
            let beg = sprite_index * 4;
            self.object_attribute_ram[sprite_index] = [data[beg+0],
                                                       data[beg+1],
                                                       data[beg+2],
                                                       data[beg+3]].into();
        }
    }

    pub fn render_scanline(&mut self) {
        let tile_map_base_addr: usize = match self.lcdc.contains(
            LcdControl::BACKGROUND_AND_TILE_MAP_DISPLAY_SELECT) {
            true => 0x1C00,
            false => 0x1800,
        };

        let tile_data_base_addr: usize = match self.lcdc.contains(
            LcdControl::BACKGROUND_AND_TILE_DATA_DISPLAY_SELECT) {
            true => 0x0000,
            false => 0x1000,
        };

        let y = self.scanline as usize;
        let tile_y = self.scanline.wrapping_add(self.scroll_y) as usize;
        let frame_buffer = &mut self.frame_buffer[self.back_buffer_index];

        let mut x = 0;
        while x < SCREEN_PIXEL_WIDTH {
            let tile_x = (x as u8).wrapping_add(self.scroll_x) as usize;
            let tile_map_offset = (tile_x / TILE_WIDTH) + (tile_y / TILE_HEIGHT) * TILE_PER_ROW;

            let tile_map_addr = tile_map_base_addr + tile_map_offset;
            let tile_map = self.video_ram[tile_map_addr] as usize;

            let tile_data_addr: usize = if tile_map >= 128 && tile_data_base_addr != 0  {
                let tile_map = 255 - tile_map + 1;
                tile_data_base_addr - tile_map * TILE_SIZE
            } else {
                tile_data_base_addr + tile_map * TILE_SIZE
            } + (tile_y % TILE_HEIGHT) * PIXEL_BIT_DEPTH;

            let tile_data_lsb = self.video_ram[tile_data_addr];
            let tile_data_msb = self.video_ram[tile_data_addr + 1];

            let bit_index = 7 - (tile_x % 8) as u32;

            let palette_index = (tile_data_lsb.wrapping_shr(bit_index) & 1) * 2 +
                                (tile_data_msb.wrapping_shr(bit_index) & 1);

            let pallete_shift = palette_index as u32 * PIXEL_BIT_DEPTH as u32;

            let palette_mask = 3;

            let palette: u32 = self.background_palette.into();

            let shade_index = (palette.wrapping_shr(pallete_shift) & palette_mask) as usize;

            let pos: usize = (x + y * SCREEN_PIXEL_WIDTH) * ARGB_BYTES_PER_PIXEL;

            let shade = &SHADE[shade_index];
            frame_buffer[pos + 0] = shade.a;
            frame_buffer[pos + 1] = shade.r;
            frame_buffer[pos + 2] = shade.g;
            frame_buffer[pos + 3] = shade.b;

            x += 1;
        }
    }

    pub fn step(&mut self, ticks: u64) {
        self.ticks += ticks;
        self.lcdc_status_interrupt_requested = false;
        self.vertical_blank_interrupt_requested = false;

        match self.mode() {
            LcdControlMode::HorizontalBlank => {
                if self.ticks >= 204 {
                    self.ticks -= 204;
                    self.increment_scanline();
    
                    if self.scanline >= 143 {
                        self.set_mode(LcdControlMode::VerticalBlank);
                    } else {
                        self.set_mode(LcdControlMode::ScanningOAM);
                    }
    
                }
            }
            LcdControlMode::VerticalBlank => {
                if self.ticks >= 456 {
                    self.ticks -= 456;
                    self.increment_scanline();

                    if self.scanline > 153 {
                        self.set_mode(LcdControlMode::ScanningOAM);
                        self.reset_scanline();

                        // Swap frame buffers (XOR SWAP)
                        self.back_buffer_index  ^= self.front_buffer_index;
                        self.front_buffer_index ^= self.back_buffer_index;
                        self.back_buffer_index  ^= self.front_buffer_index;
                    }
                }
            }
            LcdControlMode::ScanningOAM => {
                if self.ticks >= 80 {
                    self.ticks -= 80;
                    self.set_mode(LcdControlMode::Transfering);
                }
            }
            LcdControlMode::Transfering => {
                if self.ticks >= 172 {
                    self.ticks -= 172;
                    self.set_mode(LcdControlMode::HorizontalBlank);

                    // TODO: Render line
                    self.render_scanline();
                }
            }
        }
    }
}
