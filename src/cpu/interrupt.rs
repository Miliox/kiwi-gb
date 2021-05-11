bitflags! {
    #[derive(Default)]
    pub struct Interrupt: u8 {
        const VBLANK = 0b0000_0001;
        const LCDC   = 0b0000_0010;
        const TIMER  = 0b0000_0100;
        const SERIAL = 0b0000_1000;
        const HL_PIN = 0b0001_0000;
    }
}

impl From<u8> for Interrupt {
    fn from(value: u8) -> Self {
        Interrupt::from_bits_truncate(value)
    }
}

impl Into<u8> for Interrupt {
    fn into(self) -> u8 {
        self.bits
    }
}

#[allow(dead_code)]
impl Interrupt {
    pub fn vertical_blank(self) -> bool {
        self.contains(Self::VBLANK)
    }

    pub fn lcdc_status(self) -> bool {
        self.contains(Self::LCDC)
    }

    pub fn timer_overflow(self) -> bool {
        self.contains(Self::TIMER)
    }

    pub fn serial_transfer_complete(self) -> bool {
        self.contains(Self::SERIAL)
    }

    pub fn high_to_low_pin10_to_pin_13(self) -> bool {
        self.contains(Self::HL_PIN)
    }

    pub fn set_vertical_blank(&mut self) {
        self.insert(Self::VBLANK)
    }

    pub fn set_lcdc_status(&mut self) {
        self.insert(Self::LCDC)
    }

    pub fn set_timer_overflow(&mut self) {
        self.insert(Self::TIMER)
    }

    pub fn set_serial_transfer_complete(&mut self) {
        self.insert(Self::SERIAL)
    }

    pub fn set_high_to_low_pin10_to_pin_13(&mut self) {
        self.insert(Self::HL_PIN)
    }

    pub fn reset_vertical_blank(&mut self) {
        self.remove(Self::VBLANK)
    }

    pub fn reset_lcdc_status(&mut self) {
        self.remove(Self::LCDC)
    }

    pub fn reset_timer_overflow(&mut self) {
        self.remove(Self::TIMER)
    }

    pub fn reset_serial_transfer_complete(&mut self) {
        self.remove(Self::SERIAL)
    }

    pub fn reset_high_to_low_pin10_to_pin_13(&mut self) {
        self.remove(Self::HL_PIN)
    }
}
