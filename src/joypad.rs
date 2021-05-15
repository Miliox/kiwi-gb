bitflags! {
    #[derive(Default)]
    pub struct P1: u8 {
        const OUT5 = 0b0010_0000; // Button Keys
        const OUT4 = 0b0001_0000; // Direction Keys

        const IN3 = 0b0000_1000; // Down, Start
        const IN2 = 0b0000_0100; // Up, Select
        const IN1 = 0b0000_0010; // Left, B
        const IN0 = 0b0000_0001; // Right , A
    }
}

bitflags! {
    #[derive(Default)]
    pub struct Keys: u8 {
        const START  = 0b1000_0000;
        const SELECT = 0b0100_0000;
        const B      = 0b0010_0000;
        const A      = 0b0001_0000;
        const DOWN   = 0b0000_1000;
        const UP     = 0b0000_0100;
        const LEFT   = 0b0000_0010;
        const RIGHT  = 0b0000_0001;
    }
}

pub struct Joypad {
    p1: P1,
    keys: Keys,
}

impl Default for Joypad {
    fn default() -> Self {
        Self {
            p1: P1::from_bits_truncate(0x1F),
            keys: Keys::empty(),
        }
    }
}

impl Joypad {
    pub fn press(&mut self, keys: Keys) {
        self.keys.insert(keys);
        self.set_p1(self.p1());
    }

    pub fn release(&mut self, keys: Keys) {
        self.keys.remove(keys);
        self.set_p1(self.p1());
    }

    pub fn p1(&self) -> u8 { self.p1.bits() }

    pub fn set_p1(&mut self, p1: u8) {
        let output = P1::from_bits_truncate(p1) & (P1::OUT4 | P1::OUT5);

        let keys = self.keys.bits();
        let dpad = keys & 0x0F;
        let btns = keys.wrapping_shr(4);

        let mut input: u8 = 0;
        if output.contains(P1::OUT5) {
            input |= dpad;
        }
        if output.contains(P1::OUT4) {
            input |= btns;
        }
        input = !input & 0x0F;

        self.p1 = output | P1::from_bits_truncate(input);
    }
}

