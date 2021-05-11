bitflags! {
    #[derive(Default)]
    pub struct Flags: u8 {
        const Z = 0b1000_0000;
        const N = 0b0100_0000;
        const H = 0b0010_0000;
        const C = 0b0001_0000;
    }
}

impl From<u8> for Flags {
    fn from(value: u8) -> Self {
        Flags::from_bits_truncate(value)
    }
}

impl Into<u8> for Flags {
    fn into(self) -> u8 {
        self.bits
    }
}

#[allow(dead_code)]
impl Flags {
    pub fn carry(self) -> bool {
        self.contains(Self::C)
    }

    pub fn half(self) -> bool {
        self.contains(Self::H)
    }

    pub fn sub(self) -> bool {
        self.contains(Self::N)
    }

    pub fn zero(self) -> bool {
        self.contains(Self::Z)
    }

    pub fn set_carry(&mut self) {
        self.insert(Self::C);
    }

    pub fn set_half(&mut self) {
        self.insert(Self::H);
    }

    pub fn set_sub(&mut self) {
        self.insert(Self::N);
    }

    pub fn set_zero(&mut self) {
        self.insert(Self::Z);
    }

    pub fn reset_carry(&mut self) {
        self.remove(Self::C);
    }

    pub fn reset_half(&mut self) {
        self.remove(Self::H);
    }

    pub fn reset_sub(&mut self) {
        self.remove(Self::N);
    }

    pub fn reset_zero(&mut self) {
        self.remove(Self::Z);
    }

    pub fn set_carry_if(&mut self, cond: bool) {
        if cond { self.set_carry() } else { self.reset_carry() };
    }

    pub fn set_half_if(&mut self, cond: bool) {
        if cond { self.set_half() } else { self.reset_half() };
    }

    pub fn set_sub_if(&mut self, cond: bool) {
        if cond { self.set_sub() } else { self.reset_sub() };
    }

    pub fn set_zero_if(&mut self, cond: bool) {
        if cond { self.set_zero() } else { self.reset_zero() };
    }

    pub fn toggle_carry(&mut self) {
        self.toggle(Self::C);
    }

    pub fn toggle_half(&mut self) {
        self.toggle(Self::H);
    }

    pub fn toggle_sub(&mut self) {
        self.toggle(Self::N);
    }

    pub fn toggle_zero(&mut self) {
        self.toggle(Self::Z);
    }
}