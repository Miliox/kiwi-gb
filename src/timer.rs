use crate::MemoryBus;

const COUNTER_DIV: [u16; 4] = [1024, 16, 64, 256];

const DIV_ADDR: u16 = 0xFF04;
const TIMA_ADDR: u16 = 0xFF05;
const TMA_ADDR: u16 = 0xFF06;
const TAC_ADDR: u16 = 0xFF07;

#[derive(Debug, Clone)]
pub struct Timer {
    overflow_interrupt_requested: bool,
    control: u8,

    // divider
    ticks_acc: u16,
    ticks_reset: bool,

    // timer
    timer_acc: u8,
    timer_modulo: u8,
    timer_in_bit: bool,
    timer_in_mask: u16,
    timer_enable: bool,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            overflow_interrupt_requested: false,

            control: 0xff,

            ticks_acc: 0,
            ticks_reset: false,

            timer_acc: 0,
            timer_modulo: 0,
            timer_in_bit: false,
            timer_in_mask: COUNTER_DIV[3],
            timer_enable: true,
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
            DIV_ADDR => self.ticks_reset = true,
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
        self.timer_enable = control & 1 << 2 != 0;
        self.timer_in_mask = COUNTER_DIV[(control & 0x3) as usize];
        self.control = control
    }

    pub fn counter(&self) -> u8 {
        self.timer_acc
    }

    pub fn set_counter(&mut self, counter: u8) {
        self.timer_acc = counter;
    }

    pub fn divider(&self) -> u8 {
        (self.ticks_acc >> 8) as u8
    }

    pub fn modulo(&self) -> u8 {
        self.timer_modulo
    }

    pub fn set_modulo(&mut self, modulo: u8) {
        self.timer_modulo = modulo
    }

    pub fn overflow_interrupt_requested(&self) -> bool {
        self.overflow_interrupt_requested
    }

    pub fn step(&mut self, ticks: u64) {
        self.ticks_acc = if self.ticks_reset {
            0
        } else {
            self.ticks_acc.wrapping_add(ticks as u16)
        };
        self.ticks_reset = false;

        self.overflow_interrupt_requested = false;
        if self.timer_enable {
            let next_in_bit = (self.ticks_acc & self.timer_in_mask) != 0;
            if self.timer_in_bit && !next_in_bit {
                let (counter, overflow) = self.timer_acc.overflowing_add(1);
                self.timer_acc = if overflow { self.timer_modulo } else { counter };
                self.overflow_interrupt_requested = overflow;
            }
            self.timer_in_bit = next_in_bit;
        }
    }
}

#[test]
fn sync_test() {
    let mut timer = Timer::default();

    for _ in 0..64 {
        assert_eq!(0, timer.counter());
        assert_eq!(0, timer.divider());
        timer.step(4)
    }
    assert_eq!(0, timer.counter());
    assert_eq!(1, timer.divider());
    assert_eq!(256, timer.ticks_acc);
}
