bitflags! {
    /// Bit 6 - LYC=LY Coincidence Interrupt (1=Enable) (Read/Write)
    /// Bit 5 - Mode 2 OAM Interrupt         (1=Enable) (Read/Write)
    /// Bit 4 - Mode 1 V-Blank Interrupt     (1=Enable) (Read/Write)
    /// Bit 3 - Mode 0 H-Blank Interrupt     (1=Enable) (Read/Write)
    /// Bit 2 - Coincidence Flag  (0:LYC<>LY, 1:LYC=LY) (Read Only)
    /// Bit 1-0 - Mode Flag       (Mode 0-3, see below) (Read Only)
    ///           0: During H-Blank
    ///           1: During V-Blank
    ///           2: During Searching OAM
    ///           3: During Transferring Data to LCD Driver
    #[derive(Default)]
    pub struct LcdControlStatus: u8 {
        const LINE_Y_COINCIDENCE_INTERRUPT_ENABLE = 1 << 6;
        const MODE_OAM_INTERRUPT_ENABLE           = 1 << 5;
        const MODE_V_BLANK_INTERRUPT_ENABLE       = 1 << 4;
        const MODE_H_BLANK_INTERRUPT_ENABLE       = 1 << 3;
        const LINE_Y_COINCIDENCE_FLAG             = 1 << 2;
        const MODE_FLAG_MASK                      = 0b11;
        const MODE_FLAG_MSB                       = 0b10;
        const MODE_FLAG_LSB                       = 0b01;

        const READ_ONLY_MASK = 0b0000_0111;
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum LcdControlMode {
    HorizontalBlank,
    VerticalBlank,
    ScanningOAM,
    Transfering,
}

impl Into<u8> for LcdControlStatus {
    fn into(self) -> u8 {
        self.bits()
    }
}

impl From<u8> for LcdControlStatus {
    fn from(value: u8) -> Self {
        Self::from_bits(value).unwrap()
    }
}

impl Into<u8> for LcdControlMode {
    fn into(self) -> u8 {
        match self {
            Self::Transfering => 3,
            Self::ScanningOAM => 2,
            Self::VerticalBlank => 1,
            Self::HorizontalBlank => 0,
        }
    }
}

impl From<u8> for LcdControlMode {
    fn from(value: u8) -> Self {
        match value {
            3 => Self::Transfering,
            2 => Self::ScanningOAM,
            1 => Self::VerticalBlank,
            0 => Self::HorizontalBlank,
            _ => panic!()
        }
    }
}

impl Into<LcdControlStatus> for LcdControlMode {
    fn into(self) -> LcdControlStatus {
        LcdControlStatus::from_bits(self.into()).unwrap()
    }
}

impl From<LcdControlStatus> for LcdControlMode {
    fn from(value: LcdControlStatus) -> Self {
        match (value & LcdControlStatus::MODE_FLAG_MASK).bits() {
            3 => Self::Transfering,
            2 => Self::ScanningOAM,
            1 => Self::VerticalBlank,
            0 => Self::HorizontalBlank,
            _ => panic!()
        }
    }
}

#[allow(dead_code)]
impl LcdControlStatus {
    pub fn mode(&self) -> LcdControlMode {
        self.clone().into()
    }

    pub fn set_mode(&mut self, mode: LcdControlMode) {
        self.remove(Self::MODE_FLAG_MASK);
        self.insert(mode.into());
    }

    pub fn scanline_coincidence(&self) -> bool {
        self.contains(Self::LINE_Y_COINCIDENCE_FLAG)
    }

    pub fn set_scanline_coincidence(&mut self, coincidence: bool) {
        if coincidence {
            self.insert(Self::LINE_Y_COINCIDENCE_FLAG);
        } else {
            self.remove(Self::LINE_Y_COINCIDENCE_FLAG);
        }
    }
}