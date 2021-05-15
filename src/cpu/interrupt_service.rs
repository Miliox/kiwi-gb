use super::interrupt::Interrupt;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct InterruptService {
    // Master Interrupt Switch
    enabled: bool,

    // Emulate EI instruction behavior
    enable_requested: bool,

    // Emulate DI instruction behavior
    disable_requested: bool,

    // IE register (0xFFFF)
    enabled_flags: Interrupt,

    // IF register (0xFF0F)
    latched_flags: Interrupt,
}

impl InterruptService {
    fn irq(&mut self) -> Interrupt {
        self.enabled_flags & self.latched_flags
    }

    fn enable_interrupt(&mut self) {
        self.enable_requested = true;
    }

    fn disable_interrupt(&mut self) {
        self.disable_requested = true;
    }

    fn interrupt_service_routine(&mut self) -> Option<u16> {
        let irq = self.irq();

        if self.enabled {
            for (int, rst) in vec![
                    (Interrupt::VBLANK, 0x40u16),
                    (Interrupt::LCDC  , 0x48u16),
                    (Interrupt::TIMER , 0x50u16),
                    (Interrupt::SERIAL, 0x58u16), 
                    (Interrupt::HL_PIN, 0x60u16)
                ].iter() {
                if irq.contains(*int) {
                    self.enabled = false;
                    self.latched_flags.remove(*int);
                    return Some(*rst)
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn isr(int: Interrupt, ei: Interrupt) -> Option<u16> {
        let mut int_svc = InterruptService::default();
        int_svc.enabled = true;
        int_svc.enabled_flags = ei;
        int_svc.latched_flags = int;
        int_svc.interrupt_service_routine()
    }

    #[test]
    fn isr_return_test() {
        assert_eq!(Some(0x40u16), isr(Interrupt::VBLANK, Interrupt::all()));
        assert_eq!(Some(0x40u16), isr(Interrupt::all(), Interrupt::all()));

        assert_eq!(Some(0x48u16), isr(Interrupt::LCDC, Interrupt::all()));
        assert_eq!(Some(0x48u16), isr(Interrupt::all() - Interrupt::VBLANK, Interrupt::all()));

        assert_eq!(Some(0x50u16), isr(Interrupt::TIMER, Interrupt::all()));
        assert_eq!(Some(0x50u16), isr(Interrupt::all() - (Interrupt::VBLANK | Interrupt::LCDC), Interrupt::all()));

        assert_eq!(Some(0x58u16), isr(Interrupt::SERIAL, Interrupt::all()));
        assert_eq!(Some(0x58u16), isr(Interrupt::all() - (Interrupt::VBLANK | Interrupt::LCDC | Interrupt::TIMER), Interrupt::all()));

        assert_eq!(Some(0x60u16), isr(Interrupt::HL_PIN, Interrupt::all()));

        assert_eq!(None, isr(Interrupt::empty(), Interrupt::all()));
        assert_eq!(None, isr(Interrupt::VBLANK,  Interrupt::all() - Interrupt::VBLANK));
        assert_eq!(None, isr(Interrupt::LCDC,    Interrupt::all() - Interrupt::LCDC));
        assert_eq!(None, isr(Interrupt::TIMER,   Interrupt::all() - Interrupt::TIMER));
        assert_eq!(None, isr(Interrupt::SERIAL,  Interrupt::all() - Interrupt::SERIAL));
        assert_eq!(None, isr(Interrupt::HL_PIN,  Interrupt::all() - Interrupt::HL_PIN));
    }

    fn int_svc_post_isr(int: Interrupt, ei: Interrupt) -> InterruptService {
        let mut int_svc = InterruptService::default();
        int_svc.enabled = true;
        int_svc.enabled_flags = ei;
        int_svc.latched_flags = int;
        int_svc.interrupt_service_routine();
        int_svc
    }

    #[test]
    fn int_svc_post_isr_test() {
        assert_eq!(InterruptService {
            enabled: false,
            enable_requested: false,
            disable_requested: false,
            enabled_flags: Interrupt::all(),
            latched_flags: Interrupt::empty(),
        }, int_svc_post_isr(Interrupt::VBLANK, Interrupt::all()));

        assert_eq!(InterruptService {
            enabled: false,
            enable_requested: false,
            disable_requested: false,
            enabled_flags: Interrupt::all(),
            latched_flags: Interrupt::empty(),
        }, int_svc_post_isr(Interrupt::LCDC, Interrupt::all()));

        assert_eq!(InterruptService {
            enabled: false,
            enable_requested: false,
            disable_requested: false,
            enabled_flags: Interrupt::all(),
            latched_flags: Interrupt::empty(),
        }, int_svc_post_isr(Interrupt::TIMER, Interrupt::all()));

        assert_eq!(InterruptService {
            enabled: false,
            enable_requested: false,
            disable_requested: false,
            enabled_flags: Interrupt::all(),
            latched_flags: Interrupt::empty(),
        }, int_svc_post_isr(Interrupt::SERIAL, Interrupt::all()));

        assert_eq!(InterruptService {
            enabled: false,
            enable_requested: false,
            disable_requested: false,
            enabled_flags: Interrupt::all(),
            latched_flags: Interrupt::empty(),
        }, int_svc_post_isr(Interrupt::HL_PIN, Interrupt::all()));
    }
}