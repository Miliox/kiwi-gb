use crate::MemoryBus;

const COUNTER_DIV: [u64; 4] = [1024, 16, 64, 256];
const DIVIDER_DIV: u64 = 256;

const DIV_ADDR: u16 = 0xFF04;
const TIMA_ADDR: u16 = 0xFF05;
const TMA_ADDR: u16 = 0xFF06;
const TAC_ADDR: u16 = 0xFF07;

#[derive(Debug, Clone)]
pub struct Timer {
    enable: bool,
    overflow_interrupt_requested: bool,

    control: u8,
    counter: u8,
    divider: u8,
    modulo: u8,

    counter_acc: u64,
    counter_div: u64,
    divider_acc: u64,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            enable: true,
            overflow_interrupt_requested: false,

            control: 0xff,
            counter: 0,
            divider: 0,
            modulo: 0,

            counter_acc: 0,
            counter_div: COUNTER_DIV[3],
            divider_acc: 0,
        }
    }
}

impl MemoryBus for Timer {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            DIV_ADDR => self.divider(),
            TIMA_ADDR => self.counter(),
            TMA_ADDR => self.modulo(),
            TAC_ADDR => self.control(),
            _ => panic!()
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            DIV_ADDR => self.reset_divider(),
            TIMA_ADDR => self.set_counter(data),
            TMA_ADDR => self.set_modulo(data),
            TAC_ADDR => self.set_control(data),
            _ => panic!()
        }
    }
}

impl Timer {
    pub fn control(&self) -> u8 {
        self.control
    }

    pub fn set_control(&mut self, control: u8) {
        self.enable = control & 1 << 2 != 0;
        self.counter_div = COUNTER_DIV[(control & 0x3) as usize];
        self.control = control
    }

    pub fn counter(&self) -> u8 {
        self.counter
    }

    pub fn set_counter(&mut self, counter: u8) {
        self.counter = counter;
    }

    pub fn divider(&self) -> u8 {
        self.divider
    }

    pub fn reset_divider(&mut self) {
        self.divider = 0
    }

    pub fn modulo(&self) -> u8 {
        self.modulo
    }

    pub fn set_modulo(&mut self, modulo: u8) {
        self.modulo = modulo
    }

    pub fn overflow_interrupt_requested(&self) -> bool {
        self.overflow_interrupt_requested
    }

    pub fn step(&mut self, ticks: u64) {
        self.overflow_interrupt_requested = false;

        if self.enable {
            self.counter_acc += ticks;
            self.divider_acc += ticks;

            if self.counter_acc >= self.counter_div {
                self.counter_acc -= self.counter_div;

                let (counter, overflow) = self.counter.overflowing_add(1);

                self.counter = if overflow { self.modulo } else { counter };
                self.overflow_interrupt_requested = overflow;
            }

            if self.divider_acc >= DIVIDER_DIV {
                self.divider_acc -= DIVIDER_DIV;
                self.divider = self.divider.wrapping_add(1);
            }
        }
    }
}

#[test]
fn sync_test() {
    let mut timer = Timer::default();

    for _ in 0..64 {
        assert_eq!(0, timer.counter);
        assert_eq!(0, timer.divider);
        timer.step(4)
    }
    assert_eq!(1, timer.counter);
    assert_eq!(1, timer.divider);
    assert_eq!(0, timer.counter_acc);
    assert_eq!(0, timer.divider_acc);

    timer.set_control(0b1111_1100 | 2);

    for i in 0..64 {
        assert_eq!(1 + i / 16, timer.counter);
        assert_eq!(1, timer.divider);
        timer.step(4)
    }

    assert_eq!(5, timer.counter);
    assert_eq!(2, timer.divider);

    timer.set_control(0b1111_1100 | 1);

    for i in 0..64 {
        assert_eq!(5 + i / 4, timer.counter);
        assert_eq!(2, timer.divider);
        timer.step(4)
    }

    assert_eq!(21, timer.counter);
    assert_eq!(3, timer.divider);

    timer.set_control(0b1111_1100 | 0);

    for i in 0..=255 {
        assert_eq!(21, timer.counter);
        assert_eq!(3 + i / 64, timer.divider);
        timer.step(4)
    }

    assert_eq!(22, timer.counter);
    assert_eq!(7, timer.divider);
}
