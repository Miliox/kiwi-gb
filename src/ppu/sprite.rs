bitflags! {
    /// VRAM Sprite Attribute Table (OAM)
    /// Bit7   OBJ-to-BG Priority (0=OBJ Above BG, 1=OBJ Behind BG color 1-3)
    /// (Used for both BG and Window. BG color 0 is always behind OBJ)
    /// Bit6   Y flip          (0=Normal, 1=Vertically mirrored)
    /// Bit5   X flip          (0=Normal, 1=Horizontally mirrored)
    /// Bit4   Palette number  **Non CGB Mode Only** (0=OBP0, 1=OBP1)
    /// Bit3   Tile VRAM-Bank  **CGB Mode Only**     (0=Bank 0, 1=Bank 1)
    /// Bit2-0 Palette number  **CGB Mode Only**     (OBP0-7)
    #[derive(Default)]
    struct Flags: u8 {
        const PRIORITY = 1 << 7;
        const FLIP_Y   = 1 << 6;
        const FLIP_X   = 1 << 5;
        const PALETTE  = 1 << 4;
        const UNUSED3  = 1 << 3;
        const UNUSED2  = 1 << 2;
        const UNUSED1  = 1 << 1;
        const UNUSED0  = 1 << 0;
    }
}

impl Into<u8> for Flags {
    fn into(self) -> u8 {
        self.bits()
    }
}

impl From<u8> for Flags {
    fn from(value: u8) -> Self {
        Flags::from_bits(value).unwrap()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Sprite {
    y: u8,
    x: u8,
    tile: u8,
    flags: Flags,
}

impl Into<[u8; 4]> for Sprite {
    fn into(self) -> [u8; 4] {
        [self.x(), self.y(), self.tile(), self.flags()]
    }
}

impl From<[u8; 4]> for Sprite {
    fn from(value: [u8; 4]) -> Self {
        Self {
            y: value[0],
            x: value[1],
            tile: value[2],
            flags: Flags::from(value[3])
        }
    }
}

#[allow(dead_code)]
impl Sprite {
    pub fn x(&self) -> u8 {
        self.x
    }

    pub fn set_x(&mut self, x: u8) {
        self.x = x;
    }

    pub fn y(&self) -> u8 {
        self.y
    }

    pub fn set_y(&mut self, y: u8) {
        self.y = y;
    }

    pub fn tile(&self) -> u8 {
        self.tile
    }

    pub fn set_tile(&mut self, tile: u8) {
        self.tile = tile;
    }

    pub fn flags(&self) -> u8 {
        self.flags.into()
    }

    pub fn set_flags(&mut self, flags: u8) {
        self.flags = flags.into();
    }

    pub fn horizontal_flip(&self) -> bool {
        self.flags.contains(Flags::FLIP_X)
    }

    pub fn vertical_flip(&self) -> bool {
        self.flags.contains(Flags::FLIP_Y)
    }

    pub fn priority(&self) -> bool {
        self.flags.contains(Flags::PRIORITY)
    }

    pub fn palette_index(&self) -> u8 {
        if self.flags.contains(Flags::PALETTE) { 1 } else { 0 }
    }
}