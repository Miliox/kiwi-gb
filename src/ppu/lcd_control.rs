bitflags! {
    /// LCD Control (R/W)
    /// 
    /// Bit 7 - LCD Display Enable             (0=Off, 1=On)
    /// Bit 6 - Window Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
    /// Bit 5 - Window Display Enable          (0=Off, 1=On)
    /// Bit 4 - BG & Window Tile Data Select   (0=8800-97FF, 1=8000-8FFF)
    /// Bit 3 - BG Tile Map Display Select     (0=9800-9BFF, 1=9C00-9FFF)
    /// Bit 2 - OBJ (Sprite) Size              (0=8x8, 1=8x16)
    /// Bit 1 - OBJ (Sprite) Display Enable    (0=Off, 1=On)
    /// Bit 0 - BG/Window Display/Priority     (0=Off, 1=On)
    pub struct LcdControl: u8 {
        const LCD_DISPLAY_ON = 1 << 7;

        const WINDOW_TILE_MAP_DISPLAY_SELECT = 1 << 6;
        const WINDOW_DISPLAY_ON              = 1 << 5;

        const BACKGROUND_AND_TILE_DATA_DISPLAY_SELECT = 1 << 4;
        const BACKGROUND_AND_TILE_MAP_DISPLAY_SELECT  = 1 << 3;
        
        const OBJECT_SPRITE_SIZE_SELECT = 1 << 2;
        const OBJECT_SPRITE_DISPLAY_ON  = 1 << 1;

        const BACKGROUND_AND_TILE_DISPLAY_ON = 1 << 0;
    }
}

impl Default for LcdControl {
    fn default() -> Self {
        LcdControl::from_bits(0x91).unwrap()
    }
}

impl Into<u8> for LcdControl {
    fn into(self) -> u8 {
        self.bits()
    }
}

impl From<u8> for LcdControl {
    fn from(value: u8) -> Self {
        Self::from_bits(value).unwrap()
    }
}

#[allow(dead_code)]
impl LcdControl {
    pub fn is_background_on(&self) -> bool {
        self.contains(Self::BACKGROUND_AND_TILE_DISPLAY_ON)
    }

    pub fn is_lcd_on(&self) -> bool {
        self.contains(Self::LCD_DISPLAY_ON)
    }

    pub fn is_object_sprite_on(&self) -> bool {
        self.contains(Self::OBJECT_SPRITE_DISPLAY_ON)
    }

    pub fn is_window_on(&self) -> bool {
        self.contains(Self::WINDOW_DISPLAY_ON)
    }

    pub fn object_sprite_size(&self) -> (/* width */ u8, /* height */ u8) {
        if self.contains(Self::OBJECT_SPRITE_SIZE_SELECT) {
            (8, 16)
        } else {
            (8, 8)
        }
    }

    pub fn background_tile_data_addr(&self) -> (/* signed */ bool, /* beg_addr */ u16, /* end_addr */ u16) {
        if self.contains(Self::BACKGROUND_AND_TILE_DATA_DISPLAY_SELECT) {
            (false, 0x8000, 0x8FFF)
        } else {
            (false, 0x8800, 0x97FF)
        }
    }

    pub fn background_tile_map_addr(&self) -> (/* signed */ bool, /* beg_addr */ u16, /* end_addr */ u16) {
        if self.contains(Self::BACKGROUND_AND_TILE_DATA_DISPLAY_SELECT) {
            (false, 0x9C00, 0x9FFF)
        } else {
            (false, 0x9800, 0x9BFF)
        }
    }

    pub fn window_tile_map_addr(&self) -> (/* signed */ bool, /* beg_addr */ u16, /* end_addr */ u16) {
        if self.contains(Self::WINDOW_TILE_MAP_DISPLAY_SELECT) {
            (false, 0x9C00, 0x9FFF)
        } else {
            (false, 0x9800, 0x9BFF)
        }
    }
}