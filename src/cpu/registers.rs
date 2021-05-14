use super::flags::Flags;
use super::interrupt::Interrupt;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,

    bc: u16,
    de: u16,
    hl: u16,
    sp: u16,
    pc: u16,

    flags: Flags,
    int_e: Interrupt,
    int_f: Interrupt,
}

#[allow(dead_code)]
impl Registers {
    pub fn a(&self) -> u8 { self.a }
    pub fn f(&self) -> u8 { self.flags.into() }
    pub fn b(&self) -> u8 { self.b }
    pub fn c(&self) -> u8 { self.c }
    pub fn d(&self) -> u8 { self.d }
    pub fn e(&self) -> u8 { self.e }
    pub fn h(&self) -> u8 { self.h }
    pub fn l(&self) -> u8 { self.l }

    pub fn af(&self) -> u16 { join(self.a, self.flags.into()) }
    pub fn bc(&self) -> u16 { self.bc }
    pub fn de(&self) -> u16 { self.de }
    pub fn hl(&self) -> u16 { self.hl }
    pub fn sp(&self) -> u16 { self.sp }
    pub fn pc(&self) -> u16 { self.pc }

    pub fn flags(&self) -> Flags { self.flags }

    pub fn set_flags(&mut self, flags: Flags) {
        self.flags = flags
    }

    pub fn set_a(&mut self, r: u8) {
        self.a = r;
    }

    pub fn set_f(&mut self, r: u8) {
        self.flags = Flags::from(r);
    }

    pub fn set_b(&mut self, r: u8) {
        self.b = r;
        self.bc = join(self.b, self.c);
    }

    pub fn set_c(&mut self, r: u8) {
        self.c = r;
        self.bc = join(self.b, self.c);
    }

    pub fn set_d(&mut self, r: u8) {
        self.d = r;
        self.de = join(self.d, self.e);
    }

    pub fn set_e(&mut self, r: u8) {
        self.e = r;
        self.de = join(self.d, self.e);
    }

    pub fn set_h(&mut self, r: u8) {
        self.h = r;
        self.hl = join(self.h, self.l);
    }

    pub fn set_l(&mut self, r: u8) {
        self.l = r;
        self.hl = join(self.h, self.l);
    }

    pub fn set_af(&mut self, r: u16) {
        let [a, f] = split(r);
        self.a = a;
        self.flags = Flags::from(f);
    }

    pub fn set_bc(&mut self, r: u16) {
        self.bc = r;
        let [b, c] = split(r);
        self.b = b;
        self.c = c;
    }

    pub fn set_de(&mut self, r: u16) {
        self.de = r;
        let [d, e] = split(r);
        self.d = d;
        self.e = e;
    }

    pub fn set_hl(&mut self, r: u16) {
        self.hl = r;
        let [h, l] = split(r);
        self.h = h;
        self.l = l;
    }

    pub fn set_sp(&mut self, r: u16) {
        self.sp = r;
    }

    pub fn set_pc(&mut self, r: u16) {
        self.pc = r;
    }
}

#[inline(always)]
fn join(h: u8, l: u8) -> u16 { u16::from_be_bytes([h, l]) }

#[inline(always)]
fn split(r: u16) -> [u8; 2] { r.to_be_bytes() }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_test() {
        let r = 0xABCDu16;
        let [h, l] = split(r);
        assert_eq!(0xAB, h);
        assert_eq!(0xCD, l);
    }

    #[test]
    fn join_test() {
        let r = join(0xAB, 0xCD);
        assert_eq!(0xABCD, r);
    }

    #[test]
    fn registers_default_test() {
        let r = Registers::default();
        assert_eq!(0x00, r.a());
        assert_eq!(0x00, r.f());
        assert_eq!(0x00, r.b());
        assert_eq!(0x00, r.c());
        assert_eq!(0x00, r.d());
        assert_eq!(0x00, r.e());
        assert_eq!(0x00, r.h());
        assert_eq!(0x00, r.l());

        assert_eq!(0x0000, r.af());
        assert_eq!(0x0000, r.bc());
        assert_eq!(0x0000, r.de());
        assert_eq!(0x0000, r.hl());
        assert_eq!(0x0000, r.sp());
        assert_eq!(0x0000, r.pc());
    }

    #[test]
    fn set8_read16_test() {
        let mut r = Registers::default();
        r.set_a(0xAA);
        r.set_f(0xFF);
        r.set_b(0xBB);
        r.set_c(0xCC);
        r.set_d(0xDD);
        r.set_e(0xEE);
        r.set_h(0x88);
        r.set_l(0x11);

        assert_eq!(0xAA, r.a());
        assert_eq!(0xF0, r.f());
        assert_eq!(0xBB, r.b());
        assert_eq!(0xCC, r.c());
        assert_eq!(0xDD, r.d());
        assert_eq!(0xEE, r.e());
        assert_eq!(0x88, r.h());
        assert_eq!(0x11, r.l());

        assert_eq!(0xAAF0, r.af());
        assert_eq!(0xBBCC, r.bc());
        assert_eq!(0xDDEE, r.de());
        assert_eq!(0x8811, r.hl());
    }

    #[test]
    fn set16_read8_test() {
        let mut r = Registers::default();
        r.set_af(0xAAFF);
        r.set_bc(0xBBCC);
        r.set_de(0xDDEE);
        r.set_hl(0x8811);
        r.set_sp(0x1234);
        r.set_pc(0x6789);

        assert_eq!(0xAAF0, r.af());
        assert_eq!(0xBBCC, r.bc());
        assert_eq!(0xDDEE, r.de());
        assert_eq!(0x8811, r.hl());
        assert_eq!(0x1234, r.sp());
        assert_eq!(0x6789, r.pc());

        assert_eq!(0xAA, r.a());
        assert_eq!(0xF0, r.f());
        assert_eq!(0xBB, r.b());
        assert_eq!(0xCC, r.c());
        assert_eq!(0xDD, r.d());
        assert_eq!(0xEE, r.e());
        assert_eq!(0x88, r.h());
        assert_eq!(0x11, r.l());
    }
}