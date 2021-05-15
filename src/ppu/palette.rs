const COLOR_0: u32 = 0xFF9BBC0F; // WHITE
const COLOR_1: u32 = 0xFF8BAC0F; // LIGHT GRAY
const COLOR_2: u32 = 0xFF306230; // DARK GRAY
const COLOR_3: u32 = 0xFF0F380F; // BLACK

#[derive(Clone, Copy, Debug, Default)]
pub struct Palette {
    pub palette: u8
}

impl Into<u8> for Palette {
    fn into(self) -> u8 {
        self.palette
    }
}

impl From<u8> for Palette {
    fn from(value: u8) -> Self {
        Self { palette: value }
    }
}

impl Into<u32> for Palette {
    fn into(self) -> u32 {
        self.palette as u32
    }
}

#[allow(dead_code)]
impl Palette {
    pub fn palette_color_index(&self, index: u8) -> u8 {
        if index >= 4 {
            panic!();
        }

        let index: u32 = (index as u32) * 2;
        self.palette.wrapping_shr(index) & 0x3
    }

    pub fn palette_color(&self, index: u8) -> u32 {
        if index >= 4 {
            panic!();
        }

        match self.palette_color_index(index) {
            3 => COLOR_3,
            2 => COLOR_2,
            1 => COLOR_1,
            0 => COLOR_0,
            _ => panic!()
        }
    }
}