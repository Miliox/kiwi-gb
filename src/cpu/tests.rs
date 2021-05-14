
use super::*;
use flags::Flags;

unsafe fn build() -> (*mut Cpu, *mut Mmu) {
    let cpu = Box::new(Cpu::default());
    let cpu: *mut Cpu = Box::into_raw(cpu);

    let mmu = Box::new(Mmu::default());
    let mmu: *mut Mmu = Box::into_raw(mmu);

    (*cpu).mmu = mmu;
    (*mmu).cpu = cpu;

    (cpu, mmu)
}

unsafe fn destroy(component: (*mut Cpu, *mut Mmu)) {
    let (cpu, mmu): (*mut Cpu, *mut Mmu) = component;
    drop(Box::from_raw(cpu));
    drop(Box::from_raw(mmu));
}

macro_rules! int_test {
    ($int:expr, $addr:literal) => {
        unsafe {
            let (cpu, mmu) = build();
            (*cpu).interrupt_enabled = true;
            (*cpu).regs.set_sp(0xFFFE);
            (*mmu).cartridge_rom[$addr] = 0xD9;

            (*cpu).cycle();
            let r1 = (*cpu).registers();

            // Interrupt
            (*cpu).interrupt_latched_flags = $int;
            (*cpu).interrupt_enabled_flags = $int;

            (*cpu).cycle();
            let r2 = (*cpu).registers();

            (*cpu).cycle();
            let r3 = (*cpu).registers();

            (*cpu).cycle();

            destroy((cpu, mmu));

            assert_eq!(0x01, r1.pc());
            assert_eq!($addr, r2.pc());
            assert_eq!(0x02, r3.pc());
        }
    }
}

#[test]
fn vblank_int_test() {
    int_test!(Interrupt::VBLANK, 0x40);
}

#[test]
fn lcdc_int_test() {
    int_test!(Interrupt::LCDC, 0x48);
}

#[test]
fn timer_int_test() {
    int_test!(Interrupt::TIMER, 0x50);
}

#[test]
fn serial_int_test() {
    int_test!(Interrupt::SERIAL, 0x58);
}

#[test]
fn joypad_int_test() {
    int_test!(Interrupt::HL_PIN, 0x60);
}

#[test]
fn nop_test() {
    unsafe {
        let (cpu, mmu) = build();
        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        destroy((cpu, mmu));

        assert_eq!(4, tk);
        assert_eq!(1, r2.pc());

        let mut rr = r2.clone();
        rr.set_pc(r1.pc());
        assert_eq!(rr, r1);
    }
}

#[test]
fn stop_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0x01);
        (*mmu).cartridge_rom[0] = 0x10;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(4, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());

        let mut rr = r2.clone();
        rr.set_pc(r1.pc());

        assert_eq!(r1, rr);
    }
}

#[test]
fn halt_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0x76;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(4, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
    }
}

#[test]
fn di_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).interrupt_enabled = true;
        //(*cpu).next_interrupt_enabled = true;
        (*mmu).cartridge_rom[0] = 0xF3;

        let r1 = (*cpu).registers();

        let tk = (*cpu).cycle();
        let ie1 = (*cpu).interrupt_enabled;
        //let nie1 = (*cpu).next_interrupt_enabled;

        let r2 = (*cpu).registers();

        (*cpu).cycle();
        let ie2 = (*cpu).interrupt_enabled;
        //let nie2 = (*cpu).next_interrupt_enabled;

        destroy((cpu, mmu));

        assert_eq!(4, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(true, ie1);
        //assert_eq!(false, nie1);
        assert_eq!(false, ie2);
        //assert_eq!(false, nie2);
    }
}

#[test]
fn ei_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).interrupt_enabled = false;
        //(*cpu).next_interrupt_enabled = false;
        (*mmu).cartridge_rom[0] = 0xFB;

        let r1 = (*cpu).registers();

        let tk = (*cpu).cycle();
        let ie1 = (*cpu).interrupt_enabled;
        //let nie1 = (*cpu).next_interrupt_enabled;

        let r2 = (*cpu).registers();

        (*cpu).cycle();
        let ie2 = (*cpu).interrupt_enabled;
        //let nie2 = (*cpu).next_interrupt_enabled;

        destroy((cpu, mmu));

        assert_eq!(4, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(false, ie1);
        //assert_eq!(true, nie1);
        assert_eq!(true, ie2);
        //assert_eq!(true, nie2);
    }
}

macro_rules! ld_r16_d16_test {
    ($opcode:literal, $r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*mmu).cartridge_rom[0] = $opcode;
            (*mmu).cartridge_rom[1] = 0xEF;
            (*mmu).cartridge_rom[2] = 0xBE;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();

            destroy((cpu, mmu));

            assert_eq!(12, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(3, r2.pc());
            assert_eq!(0x0000, r1.$r());
            assert_eq!(0xBEEF, r2.$r());
        }
    }
}

#[test]
fn ld_bc_d16_test() {
    ld_r16_d16_test!(0x01, bc);
}

#[test]
fn ld_de_d16_test() {
    ld_r16_d16_test!(0x11, de);
}

#[test]
fn ld_hl_d16_test() {
    ld_r16_d16_test!(0x21, hl);
}

#[test]
fn ld_sp_d16_test() {
    ld_r16_d16_test!(0x31, sp);
}

#[test]
fn ld_sp_hl_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0xF9;
        (*cpu).regs.set_hl(0xABCD);
        (*cpu).regs.set_sp(0xFFFE);

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0xFFFE, r1.sp());
        assert_eq!(0xABCD, r2.sp());
    }
}

#[test]
fn ld_hl_sp_add_positive_d8_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0xF8;
        (*mmu).cartridge_rom[1] = 0x01;
        (*cpu).regs.set_sp(0x4000);

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(12, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
        assert_eq!(0x4000, r2.sp());
        assert_eq!(0x4001, r2.hl());
    }
}

#[test]
fn ld_hl_sp_add_negative_d8_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0xF8;
        (*mmu).cartridge_rom[1] = 0xFF;
        (*cpu).regs.set_sp(0x4000);

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(12, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
        assert_eq!(0x4000, r2.sp());
        assert_eq!(0x3FFF, r2.hl());
    }
}

macro_rules! ld_r16_addr_a_test {
    ($opcode:literal, $r_set:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*mmu).cartridge_rom[0] = $opcode;
            (*cpu).regs.set_a(0x99);
            (*cpu).regs.$r_set(0xA000);

            let d1 = (*mmu).cartridge_ram[0];
            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();
            let d2 = (*mmu).cartridge_ram[0];

            destroy((cpu, mmu));

            assert_eq!(8, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(1, r2.pc());
            assert_eq!(0x00, d1);
            assert_eq!(0x99, d2);
        }
    };
}

#[test]
fn ld_bc_addr_a_test() {
    ld_r16_addr_a_test!(0x02, set_bc);
}

#[test]
fn ld_de_addr_a_test() {
    ld_r16_addr_a_test!(0x12, set_de);
}

macro_rules! inc_r16_test {
    ($opcode:literal, $r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*mmu).cartridge_rom[0] = $opcode;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();

            destroy((cpu, mmu));

            assert_eq!(8, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(1, r2.pc());
            assert_eq!(0, r1.$r());
            assert_eq!(1, r2.$r());
        }
    };
}

#[test]
fn inc_bc_test() {
    inc_r16_test!(0x03, bc);
}

#[test]
fn inc_de_test() {
    inc_r16_test!(0x13, de);
}

#[test]
fn inc_hl_test() {
    inc_r16_test!(0x23, hl);
}

#[test]
fn inc_sp_test() {
    inc_r16_test!(0x33, sp);
}

#[test]
fn inc_hl_addr_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0x34;
        (*mmu).cartridge_ram[0] = 0x7F;

        let d1 = (*mmu).cartridge_ram[0];
        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let d2 = (*mmu).cartridge_ram[0];

        destroy((cpu, mmu));

        assert_eq!(12, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x7F, d1);
        assert_eq!(0x80, d2);
    }
}

macro_rules! inc_r8_test {
    ($opcode:literal, $r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*mmu).cartridge_rom[0] = $opcode;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();

            destroy((cpu, mmu));

            assert_eq!(4, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(1, r2.pc());
            assert_eq!(0, r1.$r());
            assert_eq!(1, r2.$r());
        }
    };
}

#[test]
fn inc_b_test() {
    inc_r8_test!(0x04, b);
}

#[test]
fn inc_d_test() {
    inc_r8_test!(0x14, d);
}

#[test]
fn inc_h_test() {
    inc_r8_test!(0x24, h);
}


#[test]
fn inc_c_test() {
    inc_r8_test!(0x0C, c);
}

#[test]
fn inc_e_test() {
    inc_r8_test!(0x1C, e);
}

#[test]
fn inc_l_test() {
    inc_r8_test!(0x2C, l);
}

#[test]
fn inc_a_test() {
    inc_r8_test!(0x3C, a);
}

macro_rules! dec_r8_test {
    ($opcode:literal, $r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*mmu).cartridge_rom[0] = $opcode;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();

            destroy((cpu, mmu));

            assert_eq!(4, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(1, r2.pc());
            assert_eq!(0, r1.$r());
            assert_eq!(255, r2.$r());
        }
    };
}

#[test]
fn dec_b_test() {
    dec_r8_test!(0x05, b);
}

#[test]
fn dec_d_test() {
    dec_r8_test!(0x15, d);
}

#[test]
fn dec_h_test() {
    dec_r8_test!(0x25, h);
}

#[test]
fn dec_hl_addr_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0x35;
        (*mmu).cartridge_ram[0] = 0x7F;

        let d1 = (*mmu).cartridge_ram[0];
        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let d2 = (*mmu).cartridge_ram[0];

        destroy((cpu, mmu));

        assert_eq!(12, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x7F, d1);
        assert_eq!(0x7E, d2);
    }
}

#[test]
fn dec_c_test() {
    dec_r8_test!(0x0D, c);
}

#[test]
fn dec_e_test() {
    dec_r8_test!(0x1D, e);
}

#[test]
fn dec_l_test() {
    dec_r8_test!(0x2D, l);
}

#[test]
fn dec_a_test() {
    dec_r8_test!(0x3D, a);
}

macro_rules! ld_r8_d8_test {
    ($opcode:literal, $r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*mmu).cartridge_rom[0] = $opcode;
            (*mmu).cartridge_rom[1] = 0xAB;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();

            destroy((cpu, mmu));
            assert_eq!(8, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(2, r2.pc());
            assert_eq!(0, r1.$r());
            assert_eq!(0xAB, r2.$r());
        }
    };
}

#[test]
fn ld_b_d8_test() {
    ld_r8_d8_test!(0x06, b);
}

#[test]
fn ld_d_d8_test() {
    ld_r8_d8_test!(0x16, d);
}

#[test]
fn ld_h_d8_test() {
    ld_r8_d8_test!(0x26, h);
}

#[test]
fn ld_c_d8_test() {
    ld_r8_d8_test!(0x0E, c);
}

#[test]
fn ld_e_d8_test() {
    ld_r8_d8_test!(0x1E, e);
}

#[test]
fn ld_l_d8_test() {
    ld_r8_d8_test!(0x2E, l);
}

#[test]
fn ld_a_d8_test() {
    ld_r8_d8_test!(0x3E, a);
}

#[test]
fn ld_hl_addr_d8_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0x36;
        (*mmu).cartridge_rom[1] = 0xAB;

        let d1 = (*mmu).cartridge_ram[0];
        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let d2 = (*mmu).cartridge_ram[0];

        destroy((cpu, mmu));

        assert_eq!(12, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
        assert_eq!(0, d1);
        assert_eq!(0xAB, d2);
    }
}

#[test]
fn ld_a_de_addr_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_de(0xA000);
        (*mmu).cartridge_rom[0] = 0x1A;
        (*mmu).cartridge_ram[0] = 0xFF;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x00, r1.a());
        assert_eq!(0xFF, r2.a());

        let mut rr = r2.clone();
        rr.set_a(r1.a());
        rr.set_pc(r1.pc());
        assert_eq!(rr, r1);
    }
}

#[test]
fn ld_a_hli_addr_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0x2A;
        (*mmu).cartridge_ram[0] = 0xFF;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x00, r1.a());
        assert_eq!(0xFF, r2.a());
        assert_eq!(0xA000, r1.hl());
        assert_eq!(0xA001, r2.hl());

        let mut rr = r2.clone();
        rr.set_a(r1.a());
        rr.set_hl(r1.hl());
        rr.set_pc(r1.pc());
        assert_eq!(rr, r1);
    }
}

#[test]
fn ld_a_hld_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0x3A;
        (*mmu).cartridge_ram[0] = 0xFF;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x00, r1.a());
        assert_eq!(0xFF, r2.a());
        assert_eq!(0xA000, r1.hl());
        assert_eq!(0x9FFF, r2.hl());

        let mut rr = r2.clone();
        rr.set_a(r1.a());
        rr.set_hl(r1.hl());
        rr.set_pc(r1.pc());
        assert_eq!(rr, r1);
    }
}

#[test]
fn ld_a16_sp_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_sp(0x1234);
        (*mmu).cartridge_rom[0] = 0x08;
        (*mmu).cartridge_rom[1] = 0x00;
        (*mmu).cartridge_rom[2] = 0xA0;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        let lsb = (*mmu).cartridge_ram[0];
        let msb = (*mmu).cartridge_ram[1];

        destroy((cpu, mmu));

        assert_eq!(20, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(3, r2.pc());
        assert_eq!(0x34, lsb);
        assert_eq!(0x12, msb);

        let mut rr = r2.clone();
        rr.set_pc(r1.pc());
        assert_eq!(rr, r1);
    }
}

#[test]
fn ld_a8_a_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0xFF);
        (*mmu).cartridge_rom[0] = 0xE0;
        (*mmu).cartridge_rom[1] = 0x90;

        let d1 = (*mmu).ram[0x2010];
        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let d2 = (*mmu).ram[0x2010];

        destroy((cpu, mmu));

        assert_eq!(12, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
        assert_eq!(0x00, d1);
        assert_eq!(0xFF, d2);
    }
}

#[test]
fn ld_a_a8_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0xF0;
        (*mmu).cartridge_rom[1] = 0x90;
        (*mmu).ram[0x2010] = 0xFF;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(12, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
        assert_eq!(0x00, r1.a());
        assert_eq!(0xFF, r2.a());
    }
}

#[test]
fn ld_c_zp_a_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0xFF);
        (*cpu).regs.set_c(0x90);
        (*mmu).cartridge_rom[0] = 0xE2;
        (*mmu).cartridge_rom[1] = 0x90;

        let d1 = (*mmu).ram[0x2010];
        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let d2 = (*mmu).ram[0x2010];

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x00, d1);
        assert_eq!(0xFF, d2);
    }
}

#[test]
fn ld_a_c_zp_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_c(0x90);
        (*mmu).cartridge_rom[0] = 0xF2;
        (*mmu).cartridge_rom[1] = 0x90;
        (*mmu).ram[0x2010] = 0xFF;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x00, r1.a());
        assert_eq!(0xFF, r2.a());
    }
}



#[test]
fn ld_a16_a_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0xFF);
        (*cpu).regs.set_c(0x90);
        (*mmu).cartridge_rom[0] = 0xEA;
        (*mmu).cartridge_rom[1] = 0x90;
        (*mmu).cartridge_rom[2] = 0xFF;

        let d1 = (*mmu).ram[0x2010];
        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let d2 = (*mmu).ram[0x2010];

        destroy((cpu, mmu));

        assert_eq!(16, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(3, r2.pc());
        assert_eq!(0x00, d1);
        assert_eq!(0xFF, d2);
    }
}

#[test]
fn ld_a_a16_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_c(0x90);
        (*mmu).cartridge_rom[0] = 0xFA;
        (*mmu).cartridge_rom[1] = 0x90;
        (*mmu).cartridge_rom[2] = 0xFF;
        (*mmu).ram[0x2010] = 0xFF;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(16, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(3, r2.pc());
        assert_eq!(0x00, r1.a());
        assert_eq!(0xFF, r2.a());
    }
}

macro_rules! add_hl_r16_test {
    ($opcode:literal, $set_r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*cpu).regs.set_hl(0x10F8);
            (*cpu).regs.$set_r(0x1010);
            (*mmu).cartridge_rom[0] = $opcode;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();

            destroy((cpu, mmu));

            assert_eq!(8, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(1, r2.pc());
            assert_eq!(0x10F8, r1.hl());
            assert_eq!(0x2108, r2.hl());
        }
    };
}

#[test]
fn add_hl_bc_test() {
    add_hl_r16_test!(0x09, set_bc);
}

#[test]
fn add_hl_de_test() {
    add_hl_r16_test!(0x19, set_de);
}

#[test]
fn add_hl_hl_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0x10F8);
        (*mmu).cartridge_rom[0] = 0x29;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x10F8, r1.hl());
        assert_eq!(0x21F0, r2.hl());
    }
}

#[test]
fn add_hl_sp_test() {
    add_hl_r16_test!(0x39, set_sp);
}

#[test]
fn add_sp_s8_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_sp(0xFF00);
        (*mmu).cartridge_rom[0] = 0xE8;
        (*mmu).cartridge_rom[1] = 0x7F;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(16, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
        assert_eq!(0xFF00, r1.sp());
        assert_eq!(0xFF7F, r2.sp());
    }
}

macro_rules! dec_r16_test {
    ($opcode:literal, $r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*mmu).cartridge_rom[0] = $opcode;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();

            destroy((cpu, mmu));

            assert_eq!(8, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(1, r2.pc());
            assert_eq!(0, r1.$r());
            assert_eq!(0xFFFF, r2.$r());
        }
    }
}

#[test]
fn dec_bc_test() {
    dec_r16_test!(0x0B, bc);
}

#[test]
fn dec_de_test() {
    dec_r16_test!(0x1B, de);
}

#[test]
fn dec_hl_test() {
    dec_r16_test!(0x2B, hl);
}

#[test]
fn dec_sp_test() {
    dec_r16_test!(0x3B, sp);
}

macro_rules! ld_r8_r8_test {
    ($opcode:literal, $dst:tt, $src:tt, $set_src:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*cpu).regs.$set_src(0xFF);
            (*mmu).cartridge_rom[0] = $opcode;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();

            destroy((cpu, mmu));

            assert_eq!(4, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(1, r2.pc());
            assert_eq!(0xFF, r2.$dst());
            assert_eq!(0xFF, r2.$src());
        }
    }
}

#[test]
fn ld_b_b_test() {
    ld_r8_r8_test!(0x40, b, b, set_b);
}

#[test]
fn ld_b_c_test() {
    ld_r8_r8_test!(0x41, b, c, set_c);
}

#[test]
fn ld_b_d_test() {
    ld_r8_r8_test!(0x42, b, d, set_d);
}

#[test]
fn ld_b_e_test() {
    ld_r8_r8_test!(0x43, b, e, set_e);
}

#[test]
fn ld_b_h_test() {
    ld_r8_r8_test!(0x44, b, h, set_h);
}

#[test]
fn ld_b_l_test() {
    ld_r8_r8_test!(0x45, b, l, set_l);
}

#[test]
fn ld_b_a_test() {
    ld_r8_r8_test!(0x47, b, a, set_a);
}

#[test]
fn ld_c_b_test() {
    ld_r8_r8_test!(0x48, c, b, set_b);
}

#[test]
fn ld_c_c_test() {
    ld_r8_r8_test!(0x49, c, c, set_c);
}

#[test]
fn ld_c_d_test() {
    ld_r8_r8_test!(0x4A, c, d, set_d);
}

#[test]
fn ld_c_e_test() {
    ld_r8_r8_test!(0x4B, c, e, set_e);
}

#[test]
fn ld_c_h_test() {
    ld_r8_r8_test!(0x4C, c, h, set_h);
}

#[test]
fn ld_c_l_test() {
    ld_r8_r8_test!(0x4D, c, l, set_l);
}

#[test]
fn ld_c_a_test() {
    ld_r8_r8_test!(0x4F, c, a, set_a);
}

#[test]
fn ld_d_b_test() {
    ld_r8_r8_test!(0x50, d, b, set_b);
}

#[test]
fn ld_d_c_test() {
    ld_r8_r8_test!(0x51, d, c, set_c);
}

#[test]
fn ld_d_d_test() {
    ld_r8_r8_test!(0x52, d, d, set_d);
}

#[test]
fn ld_d_e_test() {
    ld_r8_r8_test!(0x53, d, e, set_e);
}

#[test]
fn ld_d_h_test() {
    ld_r8_r8_test!(0x54, d, h, set_h);
}

#[test]
fn ld_d_l_test() {
    ld_r8_r8_test!(0x55, d, l, set_l);
}

#[test]
fn ld_d_a_test() {
    ld_r8_r8_test!(0x57, d, a, set_a);
}

#[test]
fn ld_e_b_test() {
    ld_r8_r8_test!(0x58, e, b, set_b);
}

#[test]
fn ld_e_c_test() {
    ld_r8_r8_test!(0x59, e, c, set_c);
}

#[test]
fn ld_e_d_test() {
    ld_r8_r8_test!(0x5A, e, d, set_d);
}

#[test]
fn ld_e_e_test() {
    ld_r8_r8_test!(0x5B, e, e, set_e);
}

#[test]
fn ld_e_h_test() {
    ld_r8_r8_test!(0x5C, e, h, set_h);
}

#[test]
fn ld_e_l_test() {
    ld_r8_r8_test!(0x5D, e, l, set_l);
}

#[test]
fn ld_e_a_test() {
    ld_r8_r8_test!(0x5F, e, a, set_a);
}

#[test]
fn ld_h_b_test() {
    ld_r8_r8_test!(0x60, h, b, set_b);
}

#[test]
fn ld_h_c_test() {
    ld_r8_r8_test!(0x61, h, c, set_c);
}

#[test]
fn ld_h_d_test() {
    ld_r8_r8_test!(0x62, h, d, set_d);
}

#[test]
fn ld_h_e_test() {
    ld_r8_r8_test!(0x63, h, e, set_e);
}

#[test]
fn ld_h_h_test() {
    ld_r8_r8_test!(0x64, h, h, set_h);
}

#[test]
fn ld_h_l_test() {
    ld_r8_r8_test!(0x65, h, l, set_l);
}

#[test]
fn ld_h_a_test() {
    ld_r8_r8_test!(0x67, h, a, set_a);
}

#[test]
fn ld_l_b_test() {
    ld_r8_r8_test!(0x68, l, b, set_b);
}

#[test]
fn ld_l_c_test() {
    ld_r8_r8_test!(0x69, l, c, set_c);
}

#[test]
fn ld_l_d_test() {
    ld_r8_r8_test!(0x6A, l, d, set_d);
}

#[test]
fn ld_l_e_test() {
    ld_r8_r8_test!(0x6B, l, e, set_e);
}

#[test]
fn ld_l_h_test() {
    ld_r8_r8_test!(0x6C, l, h, set_h);
}

#[test]
fn ld_l_l_test() {
    ld_r8_r8_test!(0x6D, l, l, set_l);
}

#[test]
fn ld_l_a_test() {
    ld_r8_r8_test!(0x6F, l, a, set_a);
}

#[test]
fn ld_a_b_test() {
    ld_r8_r8_test!(0x78, a, b, set_b);
}

#[test]
fn ld_a_c_test() {
    ld_r8_r8_test!(0x79, a, c, set_c);
}

#[test]
fn ld_a_d_test() {
    ld_r8_r8_test!(0x7A, a, d, set_d);
}

#[test]
fn ld_a_e_test() {
    ld_r8_r8_test!(0x7B, a, e, set_e);
}

#[test]
fn ld_a_h_test() {
    ld_r8_r8_test!(0x7C, a, h, set_h);
}

#[test]
fn ld_a_l_test() {
    ld_r8_r8_test!(0x7D, a, l, set_l);
}

#[test]
fn ld_a_a_test() {
    ld_r8_r8_test!(0x7F, a, a, set_a);
}

macro_rules! ld_r8_r16_addr_test {
    ($opcode:literal, $dst:tt, $src:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*cpu).regs.$src(0xA000);
            (*mmu).cartridge_rom[0] = $opcode;
            (*mmu).cartridge_ram[0] = 0xFF;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();

            destroy((cpu, mmu));

            assert_eq!(8, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(1, r2.pc());
            assert_eq!(0xFF, r2.$dst());
        }
    }
}

#[test]
fn ld_b_hl_addr_test() {
    ld_r8_r16_addr_test!(0x46, b, set_hl);
}

#[test]
fn ld_c_hl_addr_test() {
    ld_r8_r16_addr_test!(0x4E, c, set_hl);
}

#[test]
fn ld_d_hl_addr_test() {
    ld_r8_r16_addr_test!(0x56, d, set_hl);
}

#[test]
fn ld_e_hl_addr_test() {
    ld_r8_r16_addr_test!(0x5E, e, set_hl);
}

#[test]
fn ld_h_hl_addr_test() {
    ld_r8_r16_addr_test!(0x66, h, set_hl);
}

#[test]
fn ld_l_hl_addr_test() {
    ld_r8_r16_addr_test!(0x6E, l, set_hl);
}

#[test]
fn ld_a_hl_addr_test() {
    ld_r8_r16_addr_test!(0x7E, a, set_hl);
}

#[test]
fn ld_a_bc_addr_test() {
    ld_r8_r16_addr_test!(0x0A, a, set_bc);
}

macro_rules! ld_hl_addr_r8_test {
    ($opcode:literal, $set_r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*cpu).regs.set_hl(0xA000);
            (*cpu).regs.$set_r(0xFF);
            (*mmu).cartridge_rom[0] = $opcode;

            let d1 = (*mmu).cartridge_ram[0];
            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();
            let d2 = (*mmu).cartridge_ram[0];

            destroy((cpu, mmu));

            assert_eq!(8, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(1, r2.pc());
            assert_eq!(0x00, d1);
            assert_eq!(0xFF, d2);
        }
    }
}

#[test]
fn ld_hl_addr_b_test() {
    ld_hl_addr_r8_test!(0x70, set_b);
}

#[test]
fn ld_hl_addr_c_test() {
    ld_hl_addr_r8_test!(0x71, set_c);
}

#[test]
fn ld_hl_addr_d_test() {
    ld_hl_addr_r8_test!(0x72, set_d);
}

#[test]
fn ld_hl_addr_e_test() {
    ld_hl_addr_r8_test!(0x73, set_e);
}

#[test]
fn ld_hl_addr_a_test() {
    ld_hl_addr_r8_test!(0x77, set_a);
}

#[test]
fn ld_hl_addr_h_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA080);
        (*mmu).cartridge_rom[0] = 0x74;

        let d1 = (*mmu).cartridge_ram[0x80];
        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let d2 = (*mmu).cartridge_ram[0x80];

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x00, d1);
        assert_eq!(0xA0, d2);
    }
}

#[test]
fn ld_hl_addr_l_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA080);
        (*mmu).cartridge_rom[0] = 0x75;

        let d1 = (*mmu).cartridge_ram[0x80];
        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let d2 = (*mmu).cartridge_ram[0x80];

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x00, d1);
        assert_eq!(0x80, d2);
    }
}

#[test]
fn ld_hli_addr_a_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0xFF);
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0x22;

        let d1 = (*mmu).cartridge_ram[0];
        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let d2 = (*mmu).cartridge_ram[0];

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0, d1);
        assert_eq!(0xFF, d2);
        assert_eq!(0xA001, r2.hl());
    }
}

#[test]
fn ld_hld_addr_a_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0xFF);
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0x32;

        let d1 = (*mmu).cartridge_ram[0];
        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let d2 = (*mmu).cartridge_ram[0];

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0, d1);
        assert_eq!(0xFF, d2);
        assert_eq!(0x9FFF, r2.hl());
    }
}

macro_rules! add_a_r8_test {
    ($opcode:literal, $set_r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*cpu).regs.set_a(0x91);
            (*cpu).regs.$set_r(0x2F);
            (*mmu).cartridge_rom[0] = $opcode;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();

            destroy((cpu, mmu));

            assert_eq!(4, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(1, r2.pc());
            assert_eq!(0x91, r1.a());
            assert_eq!(0xC0, r2.a());
        }
    }
}

#[test]
fn add_a_b_test() {
    add_a_r8_test!(0x80, set_b);
}

#[test]
fn add_a_c_test() {
    add_a_r8_test!(0x81, set_c);
}

#[test]
fn add_a_d_test() {
    add_a_r8_test!(0x82, set_d);
}

#[test]
fn add_a_e_test() {
    add_a_r8_test!(0x83, set_e);
}

#[test]
fn add_a_h_test() {
    add_a_r8_test!(0x84, set_h);
}

#[test]
fn add_a_l_test() {
    add_a_r8_test!(0x85, set_l);
}

#[test]
fn add_a_hl_addr_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0x91);
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0x86;
        (*mmu).cartridge_ram[0] = 0x2F;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x91, r1.a());
        assert_eq!(0xC0, r2.a());
    }
}

#[test]
fn add_a_a_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0x91);
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0x87;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(4, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x91, r1.a());
        assert_eq!(0x22, r2.a());
    }
}

#[test]
fn add_a_d8_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0x91);
        (*mmu).cartridge_rom[0] = 0xC6;
        (*mmu).cartridge_rom[1] = 0x2F;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
        assert_eq!(0x91, r1.a());
        assert_eq!(0xC0, r2.a());
    }
}

macro_rules! adc_a_r8_test {
    ($opcode:literal, $set_src:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*cpu).regs.set_a(0x91);
            (*cpu).regs.$set_src(0x2F);
            (*cpu).regs.set_flags(Flags::C);
            (*mmu).cartridge_rom[0] = $opcode;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();

            destroy((cpu, mmu));

            assert_eq!(4, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(1, r2.pc());
            assert_eq!(0x91, r1.a());
            assert_eq!(0xC1, r2.a());
        }
    }
}

#[test]
fn adc_a_b_test() {
    adc_a_r8_test!(0x88, set_b);
}

#[test]
fn adc_a_c_test() {
    adc_a_r8_test!(0x89, set_c);
}

#[test]
fn adc_a_d_test() {
    adc_a_r8_test!(0x8A, set_d);
}

#[test]
fn adc_a_e_test() {
    adc_a_r8_test!(0x8B, set_e);
}

#[test]
fn adc_a_h_test() {
    adc_a_r8_test!(0x8C, set_h);
}

#[test]
fn adc_a_l_test() {
    adc_a_r8_test!(0x8D, set_l);
}

#[test]
fn adc_a_hl_addr_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0x91);
        (*cpu).regs.set_hl(0xA000);
        (*cpu).regs.set_flags(Flags::C);
        (*mmu).cartridge_rom[0] = 0x8E;
        (*mmu).cartridge_ram[0] = 0x2F;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x91, r1.a());
        assert_eq!(0xC1, r2.a());
    }
}

#[test]
fn adc_a_a_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0x91);
        (*cpu).regs.set_hl(0xA000);
        (*cpu).regs.set_flags(Flags::C);
        (*mmu).cartridge_rom[0] = 0x8F;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(4, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x91, r1.a());
        assert_eq!(0x23, r2.a());
    }
}

#[test]
fn adc_a_d8_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0x91);
        (*cpu).regs.set_flags(Flags::C);
        (*mmu).cartridge_rom[0] = 0xCE;
        (*mmu).cartridge_rom[1] = 0x2F;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
        assert_eq!(0x91, r1.a());
        assert_eq!(0xC1, r2.a());
    }
}

macro_rules! sub_a_r8_test {
    ($opcode:literal, $set_r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*cpu).regs.set_a(0x91);
            (*cpu).regs.$set_r(0x2F);
            (*mmu).cartridge_rom[0] = $opcode;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();

            destroy((cpu, mmu));

            assert_eq!(4, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(1, r2.pc());
            assert_eq!(0x91, r1.a());
            assert_eq!(0x62, r2.a());
        }
    }
}

#[test]
fn sub_a_b_test() {
    sub_a_r8_test!(0x90, set_b);
}

#[test]
fn sub_a_c_test() {
    sub_a_r8_test!(0x91, set_c);
}

#[test]
fn sub_a_d_test() {
    sub_a_r8_test!(0x92, set_d);
}

#[test]
fn sub_a_e_test() {
    sub_a_r8_test!(0x93, set_e);
}

#[test]
fn sub_a_h_test() {
    sub_a_r8_test!(0x94, set_h);
}

#[test]
fn sub_a_l_test() {
    sub_a_r8_test!(0x95, set_l);
}

#[test]
fn sub_a_hl_addr_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0x91);
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0x96;
        (*mmu).cartridge_ram[0] = 0x2F;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x91, r1.a());
        assert_eq!(0x62, r2.a());
    }
}

#[test]
fn sub_a_a_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0x91);
        (*mmu).cartridge_rom[0] = 0x97;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(4, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x91, r1.a());
        assert_eq!(0x00, r2.a());
    }
}

#[test]
fn sub_a_d8_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0x91);
        (*mmu).cartridge_rom[0] = 0xD6;
        (*mmu).cartridge_rom[1] = 0x2F;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
        assert_eq!(0x91, r1.a());
        assert_eq!(0x62, r2.a());
    }
}

macro_rules! sbc_a_r8_test {
    ($opcode:literal, $set_r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*cpu).regs.set_a(0x91);
            (*cpu).regs.$set_r(0x2F);
            (*cpu).regs.set_flags(Flags::C);
            (*mmu).cartridge_rom[0] = $opcode;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();

            destroy((cpu, mmu));

            assert_eq!(4, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(1, r2.pc());
            assert_eq!(0x91, r1.a());
            assert_eq!(0x61, r2.a());
        }
    }
}

#[test]
fn sbc_a_b_test() {
    sbc_a_r8_test!(0x98, set_b);
}

#[test]
fn sbc_a_c_test() {
    sbc_a_r8_test!(0x99, set_c);
}

#[test]
fn sbc_a_d_test() {
    sbc_a_r8_test!(0x9A, set_d);
}

#[test]
fn sbc_a_e_test() {
    sbc_a_r8_test!(0x9B, set_e);
}

#[test]
fn sbc_a_h_test() {
    sbc_a_r8_test!(0x9C, set_h);
}

#[test]
fn sbc_a_l_test() {
    sbc_a_r8_test!(0x9D, set_l);
}

#[test]
fn sbc_a_hl_addr_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0x91);
        (*cpu).regs.set_hl(0xA000);
        (*cpu).regs.set_flags(Flags::C);
        (*mmu).cartridge_rom[0] = 0x9E;
        (*mmu).cartridge_ram[0] = 0x2F;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x91, r1.a());
        assert_eq!(0x61, r2.a());
    }
}

#[test]
fn sbc_a_a_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0x91);
        (*cpu).regs.set_hl(0xA000);
        (*cpu).regs.set_flags(Flags::C);
        (*mmu).cartridge_rom[0] = 0x9F;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(4, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x91, r1.a());
        assert_eq!(0xFF, r2.a());
    }
}

#[test]
fn sbc_a_d8_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0x91);
        (*cpu).regs.set_flags(Flags::C);
        (*mmu).cartridge_rom[0] = 0xDE;
        (*mmu).cartridge_rom[1] = 0x2F;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
        assert_eq!(0x91, r1.a());
        assert_eq!(0x61, r2.a());
    }
}

macro_rules! and_a_r8_test {
    ($opcode:literal, $set_r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*cpu).regs.set_a(0b00111100);
            (*cpu).regs.$set_r(0b00001111);
            (*mmu).cartridge_rom[0] = $opcode;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();

            destroy((cpu, mmu));

            assert_eq!(4, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(1, r2.pc());
            assert_eq!(0b00111100, r1.a());
            assert_eq!(0b00001100, r2.a());
        }
    }
}

#[test]
fn and_a_b_test() {
    and_a_r8_test!(0xA0, set_b);
}

#[test]
fn and_a_c_test() {
    and_a_r8_test!(0xA1, set_c);
}

#[test]
fn and_a_d_test() {
    and_a_r8_test!(0xA2, set_d);
}

#[test]
fn and_a_e_test() {
    and_a_r8_test!(0xA3, set_e);
}

#[test]
fn and_a_h_test() {
    and_a_r8_test!(0xA4, set_h);
}

#[test]
fn and_a_l_test() {
    and_a_r8_test!(0xA5, set_l);
}

#[test]
fn and_a_hl_addr_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0b00111100);
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0xA6;
        (*mmu).cartridge_ram[0] = 0b00001111;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0b00111100, r1.a());
        assert_eq!(0b00001100, r2.a());
    }
}

#[test]
fn and_a_a_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0b00111100);
        (*mmu).cartridge_rom[0] = 0xA7;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(4, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0b00111100, r1.a());
        assert_eq!(0b00111100, r2.a());
    }
}

#[test]
fn and_a_d8_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0b00111100);
        (*mmu).cartridge_rom[0] = 0xE6;
        (*mmu).cartridge_rom[1] = 0b00001111;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
        assert_eq!(0b00111100, r1.a());
        assert_eq!(0b00001100, r2.a());
    }
}

macro_rules! xor_a_r8_test {
    ($opcode:literal, $set_r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*cpu).regs.set_a(0b00111100);
            (*cpu).regs.$set_r(0b00001111);
            (*mmu).cartridge_rom[0] = $opcode;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();

            destroy((cpu, mmu));

            assert_eq!(4, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(1, r2.pc());
            assert_eq!(0b00111100, r1.a());
            assert_eq!(0b00110011, r2.a());
        }
    }
}

#[test]
fn xor_a_b_test() {
    xor_a_r8_test!(0xA8, set_b);
}

#[test]
fn xor_a_c_test() {
    xor_a_r8_test!(0xA9, set_c);
}

#[test]
fn xor_a_d_test() {
    xor_a_r8_test!(0xAA, set_d);
}

#[test]
fn xor_a_e_test() {
    xor_a_r8_test!(0xAB, set_e);
}

#[test]
fn xor_a_h_test() {
    xor_a_r8_test!(0xAC, set_h);
}

#[test]
fn xor_a_l_test() {
    xor_a_r8_test!(0xAD, set_l);
}

#[test]
fn xor_a_hl_addr_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0b00111100);
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0xAE;
        (*mmu).cartridge_ram[0] = 0b00001111;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0b00111100, r1.a());
        assert_eq!(0b00110011, r2.a());
    }
}

#[test]
fn xor_a_a_addr_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0b00111100);
        (*mmu).cartridge_rom[0] = 0xAF;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(4, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0b00111100, r1.a());
        assert_eq!(0b00000000, r2.a());
    }
}

#[test]
fn xor_a_d8_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0b00111100);
        (*mmu).cartridge_rom[0] = 0xEE;
        (*mmu).cartridge_rom[1] = 0b00001111;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
        assert_eq!(0b00111100, r1.a());
        assert_eq!(0b00110011, r2.a());
    }
}

macro_rules! or_a_r8_test {
    ($opcode:literal, $set_r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*cpu).regs.set_a(0b00111100);
            (*cpu).regs.$set_r(0b00001111);
            (*mmu).cartridge_rom[0] = $opcode;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();

            destroy((cpu, mmu));

            assert_eq!(4, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(1, r2.pc());
            assert_eq!(0b00111100, r1.a());
            assert_eq!(0b00111111, r2.a());
        }
    }
}

#[test]
fn or_a_b_test() {
    or_a_r8_test!(0xB0, set_b);
}

#[test]
fn or_a_c_test() {
    or_a_r8_test!(0xB1, set_c);
}

#[test]
fn or_a_d_test() {
    or_a_r8_test!(0xB2, set_d);
}

#[test]
fn or_a_e_test() {
    or_a_r8_test!(0xB3, set_e);
}

#[test]
fn or_a_h_test() {
    or_a_r8_test!(0xB4, set_h);
}

#[test]
fn or_a_l_test() {
    or_a_r8_test!(0xB5, set_l);
}

#[test]
fn or_a_hl_addr_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0b00111100);
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0xB6;
        (*mmu).cartridge_ram[0] = 0b00001111;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0b00111100, r1.a());
        assert_eq!(0b00111111, r2.a());
    }
}

#[test]
fn or_a_a_addr_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0b00111100);
        (*mmu).cartridge_rom[0] = 0xB7;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(4, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0b00111100, r1.a());
        assert_eq!(0b00111100, r2.a());
    }
}

#[test]
fn or_a_d8_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0b00111100);
        (*mmu).cartridge_rom[0] = 0xF6;
        (*mmu).cartridge_rom[1] = 0b00001111;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
        assert_eq!(0b00111100, r1.a());
        assert_eq!(0b00111111, r2.a());
    }
}

macro_rules! cp_r8_test {
    ($opcode:literal, $set_r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*cpu).regs.set_a(0x0F);
            (*cpu).regs.$set_r(0x0F);
            (*mmu).cartridge_rom[0] = $opcode;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();

            destroy((cpu, mmu));

            assert_eq!(4, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(1, r2.pc());
            assert_eq!(0x0F, r1.a());
            assert_eq!(0x0F, r2.a());
            assert_eq!(Flags::Z | Flags::N, r2.flags());
        }
    }
}

#[test]
fn cp_b_test() {
    cp_r8_test!(0xB8, set_b);
}

#[test]
fn cp_c_test() {
    cp_r8_test!(0xB9, set_c);
}

#[test]
fn cp_d_test() {
    cp_r8_test!(0xBA, set_d);
}

#[test]
fn cp_e_test() {
    cp_r8_test!(0xBB, set_e);
}

#[test]
fn cp_h_test() {
    cp_r8_test!(0xBC, set_h);
}

#[test]
fn cp_l_test() {
    cp_r8_test!(0xBD, set_l);
}

#[test]
fn cp_a_hl_addr_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0x0F);
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0xBE;
        (*mmu).cartridge_ram[0] = 0x0F;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x0F, r1.a());
        assert_eq!(0x0F, r2.a());
        assert_eq!(Flags::Z | Flags::N, r2.flags());
    }
}

#[test]
fn cp_a_test() {
    cp_r8_test!(0xBF, set_a);
}

#[test]
fn cp_a_d8_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0x0F);
        (*mmu).cartridge_rom[0] = 0xFE;
        (*mmu).cartridge_rom[1] = 0x0F;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
        assert_eq!(0x0F, r1.a());
        assert_eq!(0x0F, r2.a());
        assert_eq!(Flags::Z | Flags::N, r2.flags());
    }
}

#[test]
fn daa_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0x0A);
        (*mmu).cartridge_rom[0] = 0x27;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(4, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x0A, r1.a());
        assert_eq!(0x10, r2.a());
    }
}

#[test]
fn rlca_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0x80);
        (*mmu).cartridge_rom[0] = 0x07;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(4, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x80, r1.a());
        assert_eq!(0x01, r2.a());
        assert_eq!(Flags::empty(), r1.flags());
        assert_eq!(Flags::C,       r2.flags());

        let mut rr = r2.clone();
        rr.set_flags(r1.flags());
        rr.set_a(r1.a());
        rr.set_pc(r1.pc());
        assert_eq!(rr, r1);
    }
}

#[test]
fn rrca_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0x01);
        (*mmu).cartridge_rom[0] = 0x0F;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(4, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x01, r1.a());
        assert_eq!(0x80, r2.a());
        assert_eq!(Flags::empty(), r1.flags());
        assert_eq!(Flags::C,       r2.flags());

        let mut rr = r2.clone();
        rr.set_flags(r1.flags());
        rr.set_a(r1.a());
        rr.set_pc(r1.pc());
        assert_eq!(rr, r1);
    }
}

#[test]
fn rla_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0x81);
        (*mmu).cartridge_rom[0] = 0x17;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();

        destroy((cpu, mmu));

        assert_eq!(4, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x81, r1.a());
        assert_eq!(0x02, r2.a());
        assert_eq!(Flags::empty(), r1.flags());
        assert_eq!(Flags::C,       r2.flags());
    }
}

#[test]
fn rra_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0x81);
        (*mmu).cartridge_rom[0] = 0x1F;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(4, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(0x81, r1.a());
        assert_eq!(0x40, r2.a());
        assert_eq!(Flags::empty(), r1.flags());
        assert_eq!(Flags::C,       r2.flags());
    }
}

macro_rules! rlc_r8_test {
    ($opcode:literal, $r:tt, $set_r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*cpu).regs.$set_r(0x80);
            (*mmu).cartridge_rom[0] = 0xCB;
            (*mmu).cartridge_rom[1] = $opcode;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();
            (*cpu).cycle();

            destroy((cpu, mmu));

            assert_eq!(8, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(2, r2.pc());
            assert_eq!(0x80, r1.$r());
            assert_eq!(0x01, r2.$r());
            assert_eq!(Flags::empty(), r1.flags());
            assert_eq!(Flags::C,       r2.flags());
        }
    }
}

#[test]
fn rlc_b_test() {
    rlc_r8_test!(0x00, b, set_b);
}

#[test]
fn rlc_c_test() {
    rlc_r8_test!(0x01, c, set_c);
}

#[test]
fn rlc_d_test() {
    rlc_r8_test!(0x02, d, set_d);
}

#[test]
fn rlc_e_test() {
    rlc_r8_test!(0x03, e, set_e);
}

#[test]
fn rlc_h_test() {
    rlc_r8_test!(0x04, h, set_h);
}

#[test]
fn rlc_l_test() {
    rlc_r8_test!(0x05, l, set_l);
}

#[test]
fn rlc_hl_addr_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0xCB;
        (*mmu).cartridge_rom[1] = 0x06;
        (*mmu).cartridge_ram[0] = 0x80;

        let d1 = (*mmu).cartridge_ram[0];
        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let d2 = (*mmu).cartridge_ram[0];

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
        assert_eq!(0x80, d1);
        assert_eq!(0x01, d2);
        assert_eq!(Flags::empty(), r1.flags());
        assert_eq!(Flags::C,       r2.flags());
    }
}

#[test]
fn rlc_a_test() {
    rlc_r8_test!(0x07, a, set_a);
}

macro_rules! rrc_r8_test {
    ($opcode:literal, $r:tt, $set_r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*cpu).regs.$set_r(0x01);
            (*mmu).cartridge_rom[0] = 0xCB;
            (*mmu).cartridge_rom[1] = $opcode;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();
            (*cpu).cycle();

            destroy((cpu, mmu));

            assert_eq!(8, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(2, r2.pc());
            assert_eq!(0x01, r1.$r());
            assert_eq!(0x80, r2.$r());
            assert_eq!(Flags::empty(), r1.flags());
            assert_eq!(Flags::C,       r2.flags());
        }
    }
}

#[test]
fn rrc_b_test() {
    rrc_r8_test!(0x08, b, set_b);
}


#[test]
fn rrc_c_test() {
    rrc_r8_test!(0x09, c, set_c);
}

#[test]
fn rrc_d_test() {
    rrc_r8_test!(0x0A, d, set_d);
}

#[test]
fn rrc_e_test() {
    rrc_r8_test!(0x0B, e, set_e);
}

#[test]
fn rrc_h_test() {
    rrc_r8_test!(0x0C, h, set_h);
}

#[test]
fn rrc_l_test() {
    rrc_r8_test!(0x0D, l, set_l);
}

#[test]
fn rrc_hl_addr_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0xCB;
        (*mmu).cartridge_rom[1] = 0x0E;
        (*mmu).cartridge_ram[0] = 0x01;

        let d1 = (*mmu).cartridge_ram[0];
        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let d2 = (*mmu).cartridge_ram[0];

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
        assert_eq!(0x01, d1);
        assert_eq!(0x80, d2);
        assert_eq!(Flags::empty(), r1.flags());
        assert_eq!(Flags::C,       r2.flags());
    }
}

#[test]
fn rrc_a_test() {
    rrc_r8_test!(0x0F, a, set_a);
}

macro_rules! rl_r8_test {
    ($opcode:literal, $r:tt, $set_r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*cpu).regs.$set_r(0x80);
            (*cpu).regs.set_flags(Flags::C);
            (*mmu).cartridge_rom[0] = 0xCB;
            (*mmu).cartridge_rom[1] = $opcode;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();
            (*cpu).cycle();

            destroy((cpu, mmu));

            assert_eq!(8, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(2, r2.pc());
            assert_eq!(0x80, r1.$r());
            assert_eq!(0x01, r2.$r());
            assert_eq!(Flags::C, r1.flags());
            assert_eq!(Flags::C, r2.flags());
        }
    }
}

#[test]
fn rl_b_test() {
    rl_r8_test!(0x10, b, set_b);
}

#[test]
fn rl_c_test() {
    rl_r8_test!(0x11, c, set_c);
}

#[test]
fn rl_d_test() {
    rl_r8_test!(0x12, d, set_d);
}

#[test]
fn rl_e_test() {
    rl_r8_test!(0x13, e, set_e);
}

#[test]
fn rl_h_test() {
    rl_r8_test!(0x14, h, set_h);
}

#[test]
fn rl_l_test() {
    rl_r8_test!(0x15, l, set_l);
}

#[test]
fn rl_hl_addr_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0xCB;
        (*mmu).cartridge_rom[1] = 0x16;
        (*mmu).cartridge_ram[0] = 0x80;

        let d1 = (*mmu).cartridge_ram[0];
        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let d2 = (*mmu).cartridge_ram[0];

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
        assert_eq!(0x80, d1);
        assert_eq!(0x00, d2);
        assert_eq!(Flags::empty(), r1.flags());
        assert_eq!(Flags::C | Flags::Z,       r2.flags());
    }
}

#[test]
fn rl_a_test() {
    rl_r8_test!(0x17, a, set_a);
}

macro_rules! rr_r8_test {
    ($opcode:literal, $r:tt, $set_r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*cpu).regs.$set_r(0x01);
            (*cpu).regs.set_flags(Flags::C);
            (*mmu).cartridge_rom[0] = 0xCB;
            (*mmu).cartridge_rom[1] = $opcode;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();
            (*cpu).cycle();

            destroy((cpu, mmu));

            assert_eq!(8, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(2, r2.pc());
            assert_eq!(0x01, r1.$r());
            assert_eq!(0x80, r2.$r());
            assert_eq!(Flags::C, r1.flags());
            assert_eq!(Flags::C, r2.flags());
        }
    }
}

#[test]
fn rr_b_test() {
    rr_r8_test!(0x18, b, set_b);
}


#[test]
fn rr_c_test() {
    rr_r8_test!(0x19, c, set_c);
}

#[test]
fn rr_d_test() {
    rr_r8_test!(0x1A, d, set_d);
}

#[test]
fn rr_e_test() {
    rr_r8_test!(0x1B, e, set_e);
}

#[test]
fn rr_h_test() {
    rr_r8_test!(0x1C, h, set_h);
}

#[test]
fn rr_l_test() {
    rr_r8_test!(0x1D, l, set_l);
}

#[test]
fn rr_hl_addr_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0xCB;
        (*mmu).cartridge_rom[1] = 0x1E;
        (*mmu).cartridge_ram[0] = 0x01;

        let d1 = (*mmu).cartridge_ram[0];
        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let d2 = (*mmu).cartridge_ram[0];

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
        assert_eq!(0x01, d1);
        assert_eq!(0x00, d2);
        assert_eq!(Flags::empty(), r1.flags());
        assert_eq!(Flags::C | Flags::Z, r2.flags());
    }
}

#[test]
fn rr_a_test() {
    rr_r8_test!(0x1F, a, set_a);
}

macro_rules! sla_r8_test {
    ($opcode:literal, $r:tt, $set_r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*cpu).regs.$set_r(0x80);
            (*mmu).cartridge_rom[0] = 0xCB;
            (*mmu).cartridge_rom[1] = $opcode;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();
            (*cpu).cycle();

            destroy((cpu, mmu));

            assert_eq!(8, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(2, r2.pc());
            assert_eq!(0x80, r1.$r());
            assert_eq!(0x00, r2.$r());
            assert_eq!(Flags::empty(), r1.flags());
            assert_eq!(Flags::C | Flags::Z, r2.flags());
        }
    }
}

#[test]
fn sla_b_test() {
    sla_r8_test!(0x20, b, set_b);
}

#[test]
fn sla_c_test() {
    sla_r8_test!(0x21, c, set_c);
}

#[test]
fn sla_d_test() {
    sla_r8_test!(0x22, d, set_d);
}

#[test]
fn sla_e_test() {
    sla_r8_test!(0x23, e, set_e);
}

#[test]
fn sla_h_test() {
    sla_r8_test!(0x24, h, set_h);
}

#[test]
fn sla_l_test() {
    sla_r8_test!(0x25, l, set_l);
}

#[test]
fn sla_hl_addr_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0xCB;
        (*mmu).cartridge_rom[1] = 0x26;
        (*mmu).cartridge_ram[0] = 0x80;

        let d1 = (*mmu).cartridge_ram[0];
        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let d2 = (*mmu).cartridge_ram[0];

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
        assert_eq!(0x80, d1);
        assert_eq!(0x00, d2);
        assert_eq!(Flags::empty(), r1.flags());
        assert_eq!(Flags::C | Flags::Z, r2.flags());
    }
}

#[test]
fn sla_a_test() {
    sla_r8_test!(0x27, a, set_a);
}

macro_rules! sra_r8_test {
    ($opcode:literal, $r:tt, $set_r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*cpu).regs.$set_r(0x81);
            (*mmu).cartridge_rom[0] = 0xCB;
            (*mmu).cartridge_rom[1] = $opcode;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();
            (*cpu).cycle();

            destroy((cpu, mmu));

            assert_eq!(8, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(2, r2.pc());
            assert_eq!(0x81, r1.$r());
            assert_eq!(0xC0, r2.$r());
            assert_eq!(Flags::empty(), r1.flags());
            assert_eq!(Flags::C, r2.flags());
        }
    }
}

#[test]
fn sra_b_test() {
    sra_r8_test!(0x28, b, set_b);
}

#[test]
fn sra_c_test() {
    sra_r8_test!(0x29, c, set_c);
}

#[test]
fn sra_d_test() {
    sra_r8_test!(0x2A, d, set_d);
}

#[test]
fn sra_e_test() {
    sra_r8_test!(0x2B, e, set_e);
}

#[test]
fn sra_h_test() {
    sra_r8_test!(0x2C, h, set_h);
}

#[test]
fn sra_l_test() {
    sra_r8_test!(0x2D, l, set_l);
}

#[test]
fn sra_hl_addr_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0xCB;
        (*mmu).cartridge_rom[1] = 0x2E;
        (*mmu).cartridge_ram[0] = 0x81;

        let d1 = (*mmu).cartridge_ram[0];
        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let d2 = (*mmu).cartridge_ram[0];

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
        assert_eq!(0x81, d1);
        assert_eq!(0xC0, d2);
        assert_eq!(Flags::empty(), r1.flags());
        assert_eq!(Flags::C, r2.flags());
    }
}

#[test]
fn sra_a_test() {
    sra_r8_test!(0x2F, a, set_a);
}

macro_rules! swap_r8_test {
    ($opcode:literal, $r:tt, $set_r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*cpu).regs.$set_r(0x81);
            (*mmu).cartridge_rom[0] = 0xCB;
            (*mmu).cartridge_rom[1] = $opcode;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();
            (*cpu).cycle();

            destroy((cpu, mmu));

            assert_eq!(8, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(2, r2.pc());
            assert_eq!(0x81, r1.$r());
            assert_eq!(0x18, r2.$r());
        }
    }
}


#[test]
fn swap_b_test() {
    swap_r8_test!(0x30, b, set_b);
}

#[test]
fn swap_c_test() {
    swap_r8_test!(0x31, c, set_c);
}

#[test]
fn swap_d_test() {
    swap_r8_test!(0x32, d, set_d);
}

#[test]
fn swap_e_test() {
    swap_r8_test!(0x33, e, set_e);
}

#[test]
fn swap_h_test() {
    swap_r8_test!(0x34, h, set_h);
}

#[test]
fn swap_l_test() {
    swap_r8_test!(0x35, l, set_l);
}

#[test]
fn swap_hl_addr() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0xCB;
        (*mmu).cartridge_rom[1] = 0x36;
        (*mmu).cartridge_ram[0] = 0x81;

        let d1 = (*mmu).cartridge_ram[0];
        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let d2 = (*mmu).cartridge_ram[0];
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
        assert_eq!(0x81, d1);
        assert_eq!(0x18, d2);
    }
}

#[test]
fn swap_a_test() {
    swap_r8_test!(0x37, a, set_a);
}

macro_rules! srl_r8_test {
    ($opcode:literal, $r:tt, $set_r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*cpu).regs.$set_r(0x81);
            (*mmu).cartridge_rom[0] = 0xCB;
            (*mmu).cartridge_rom[1] = $opcode;

            let r1 = (*cpu).registers();
            let tk = (*cpu).cycle();
            let r2 = (*cpu).registers();
            (*cpu).cycle();

            destroy((cpu, mmu));

            assert_eq!(8, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(2, r2.pc());
            assert_eq!(0x81, r1.$r());
            assert_eq!(0x40, r2.$r());
            assert_eq!(Flags::empty(), r1.flags());
            assert_eq!(Flags::C, r2.flags());
        }
    }
}

#[test]
fn srl_b_test() {
    srl_r8_test!(0x38, b, set_b);
}

#[test]
fn srl_c_test() {
    srl_r8_test!(0x39, c, set_c);
}

#[test]
fn srl_d_test() {
    srl_r8_test!(0x3A, d, set_d);
}

#[test]
fn srl_e_test() {
    srl_r8_test!(0x3B, e, set_e);
}

#[test]
fn srl_h_test() {
    srl_r8_test!(0x3C, h, set_h);
}

#[test]
fn srl_l_test() {
    srl_r8_test!(0x3D, l, set_l);
}

#[test]
fn srl_hl_addr_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0xCB;
        (*mmu).cartridge_rom[1] = 0x3E;
        (*mmu).cartridge_ram[0] = 0x81;

        let d1 = (*mmu).cartridge_ram[0];
        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let d2 = (*mmu).cartridge_ram[0];

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
        assert_eq!(0x81, d1);
        assert_eq!(0x40, d2);
        assert_eq!(Flags::empty(), r1.flags());
        assert_eq!(Flags::C, r2.flags());
    }
}

#[test]
fn srl_a_test() {
    srl_r8_test!(0x3F, a, set_a);
}

macro_rules! bit_set_r8_test {
    ($opcode:literal, $r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*mmu).cartridge_rom[0] = 0xCB;
            (*mmu).cartridge_rom[1] = $opcode;
            (*mmu).cartridge_rom[2] = 0xCB;
            (*mmu).cartridge_rom[3] = $opcode + 8;
            (*mmu).cartridge_rom[4] = 0xCB;
            (*mmu).cartridge_rom[5] = $opcode + 16;
            (*mmu).cartridge_rom[6] = 0xCB;
            (*mmu).cartridge_rom[7] = $opcode + 24;
            (*mmu).cartridge_rom[8] = 0xCB;
            (*mmu).cartridge_rom[9] = $opcode + 32;
            (*mmu).cartridge_rom[10] = 0xCB;
            (*mmu).cartridge_rom[11] = $opcode + 40;
            (*mmu).cartridge_rom[12] = 0xCB;
            (*mmu).cartridge_rom[13] = $opcode + 48;
            (*mmu).cartridge_rom[14] = 0xCB;
            (*mmu).cartridge_rom[15] = $opcode + 56;

            let t1 = (*cpu).cycle();
            let r1 = (*cpu).registers();
            let t2 = (*cpu).cycle();
            let r2 = (*cpu).registers();
            let t3 = (*cpu).cycle();
            let r3 = (*cpu).registers();
            let t4 = (*cpu).cycle();
            let r4 = (*cpu).registers();
            let t5 = (*cpu).cycle();
            let r5 = (*cpu).registers();
            let t6 = (*cpu).cycle();
            let r6 = (*cpu).registers();
            let t7 = (*cpu).cycle();
            let r7 = (*cpu).registers();
            let t8 = (*cpu).cycle();
            let r8 = (*cpu).registers();
            (*cpu).cycle();

            destroy((cpu, mmu));

            assert_eq!(8, t1);
            assert_eq!(8, t2);
            assert_eq!(8, t3);
            assert_eq!(8, t4);
            assert_eq!(8, t5);
            assert_eq!(8, t6);
            assert_eq!(8, t7);
            assert_eq!(8, t8);

            assert_eq!(0x01, r1.$r());
            assert_eq!(0x03, r2.$r());
            assert_eq!(0x07, r3.$r());
            assert_eq!(0x0F, r4.$r());
            assert_eq!(0x1F, r5.$r());
            assert_eq!(0x3F, r6.$r());
            assert_eq!(0x7F, r7.$r());
            assert_eq!(0xFF, r8.$r());
        }
    }
}

#[test]
fn bit_set_b_test() {
    bit_set_r8_test!(0xC0, b);
    bit_set_r8_test!(0xC1, c);
    bit_set_r8_test!(0xC2, d);
    bit_set_r8_test!(0xC3, e);
    bit_set_r8_test!(0xC4, h);
    bit_set_r8_test!(0xC5, l);
    bit_set_r8_test!(0xC7, a);
}

macro_rules! bit_reset_r8_test {
    ($opcode:literal, $r:tt, $set_r:tt) => {
        unsafe {
            let (cpu, mmu) = build();
            (*cpu).regs.$set_r(0xFF);
            (*mmu).cartridge_rom[0] = 0xCB;
            (*mmu).cartridge_rom[1] = $opcode;
            (*mmu).cartridge_rom[2] = 0xCB;
            (*mmu).cartridge_rom[3] = $opcode + 8;
            (*mmu).cartridge_rom[4] = 0xCB;
            (*mmu).cartridge_rom[5] = $opcode + 16;
            (*mmu).cartridge_rom[6] = 0xCB;
            (*mmu).cartridge_rom[7] = $opcode + 24;
            (*mmu).cartridge_rom[8] = 0xCB;
            (*mmu).cartridge_rom[9] = $opcode + 32;
            (*mmu).cartridge_rom[10] = 0xCB;
            (*mmu).cartridge_rom[11] = $opcode + 40;
            (*mmu).cartridge_rom[12] = 0xCB;
            (*mmu).cartridge_rom[13] = $opcode + 48;
            (*mmu).cartridge_rom[14] = 0xCB;
            (*mmu).cartridge_rom[15] = $opcode + 56;

            let t1 = (*cpu).cycle();
            let r1 = (*cpu).registers();
            let t2 = (*cpu).cycle();
            let r2 = (*cpu).registers();
            let t3 = (*cpu).cycle();
            let r3 = (*cpu).registers();
            let t4 = (*cpu).cycle();
            let r4 = (*cpu).registers();
            let t5 = (*cpu).cycle();
            let r5 = (*cpu).registers();
            let t6 = (*cpu).cycle();
            let r6 = (*cpu).registers();
            let t7 = (*cpu).cycle();
            let r7 = (*cpu).registers();
            let t8 = (*cpu).cycle();
            let r8 = (*cpu).registers();
            (*cpu).cycle();

            destroy((cpu, mmu));

            assert_eq!(8, t1);
            assert_eq!(8, t2);
            assert_eq!(8, t3);
            assert_eq!(8, t4);
            assert_eq!(8, t5);
            assert_eq!(8, t6);
            assert_eq!(8, t7);
            assert_eq!(8, t8);

            assert_eq!(0xfe, r1.$r());
            assert_eq!(0xfc, r2.$r());
            assert_eq!(0xf8, r3.$r());
            assert_eq!(0xf0, r4.$r());
            assert_eq!(0xe0, r5.$r());
            assert_eq!(0xc0, r6.$r());
            assert_eq!(0x80, r7.$r());
            assert_eq!(0x00, r8.$r());
        }
    }
}

#[test]
fn bit_reset_b_test() {
    bit_reset_r8_test!(0x80, b, set_b);
    bit_reset_r8_test!(0x81, c, set_c);
    bit_reset_r8_test!(0x82, d, set_d);
    bit_reset_r8_test!(0x83, e, set_e);
    bit_reset_r8_test!(0x84, h, set_h);
    bit_reset_r8_test!(0x85, l, set_l);
    bit_reset_r8_test!(0x87, a, set_a);
}

#[test]
fn cpl_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_a(0x28);
        (*mmu).cartridge_rom[0] = 0x2F;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(4, tk);
        assert_eq!(0x00, r1.pc());
        assert_eq!(0x1, r2.pc());
        assert_eq!(0x28, r1.a());
        assert_eq!(0xD7, r2.a());
    }
}

#[test]
fn scf_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0x37;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(4, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(Flags::empty(), r1.flags());
        assert_eq!(Flags::C, r2.flags());
    }
}

#[test]
fn ccf_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_flags(Flags::C);
        (*mmu).cartridge_rom[0] = 0x3F;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(4, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(1, r2.pc());
        assert_eq!(Flags::C, r1.flags());
        assert_eq!(Flags::empty(), r2.flags());
    }
}

#[test]
fn jr_forward_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0x18;
        (*mmu).cartridge_rom[1] = 0x10;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(12, tk);
        assert_eq!(0x00, r1.pc());
        assert_eq!(0x12, r2.pc());
    }
}

#[test]
fn jr_backward_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0x18;
        (*mmu).cartridge_rom[1] = 0xFE;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(12, tk);
        assert_eq!(0x00, r1.pc());
        assert_eq!(0x00, r2.pc());
    }
}

#[test]
fn jr_zero_forward_zero_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*cpu).regs.set_flags(Flags::Z);
        (*mmu).cartridge_rom[0] = 0x28;
        (*mmu).cartridge_rom[1] = 0x10;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(12, tk);
        assert_eq!(0x00, r1.pc());
        assert_eq!(0x12, r2.pc());
    }
}

#[test]
fn jr_zero_backward_zero_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*cpu).regs.set_flags(Flags::Z);
        (*mmu).cartridge_rom[0] = 0x28;
        (*mmu).cartridge_rom[1] = 0xFE;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(12, tk);
        assert_eq!(0x00, r1.pc());
        assert_eq!(0x00, r2.pc());
    }
}

#[test]
fn jr_zero_forward_zero_not_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0x28;
        (*mmu).cartridge_rom[1] = 0x10;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
    }
}

#[test]
fn jr_zero_backward_zero_not_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0x28;
        (*mmu).cartridge_rom[1] = 0xFE;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
    }
}

#[test]
fn jr_not_zero_forward_zero_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*cpu).regs.set_flags(Flags::Z);
        (*mmu).cartridge_rom[0] = 0x20;
        (*mmu).cartridge_rom[1] = 0x10;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
    }
}

#[test]
fn jr_not_zero_backward_zero_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*cpu).regs.set_flags(Flags::Z);
        (*mmu).cartridge_rom[0] = 0x20;
        (*mmu).cartridge_rom[1] = 0xFE;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
    }
}

#[test]
fn jr_not_zero_forward_zero_not_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0x20;
        (*mmu).cartridge_rom[1] = 0x10;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(12, tk);
        assert_eq!(0x00, r1.pc());
        assert_eq!(0x12, r2.pc());
    }
}

#[test]
fn jr_not_zero_backward_zero_not_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0x20;
        (*mmu).cartridge_rom[1] = 0xFE;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(12, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(0, r2.pc());
    }
}

#[test]
fn jr_not_carry_forward_carry_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*cpu).regs.set_flags(Flags::C);
        (*mmu).cartridge_rom[0] = 0x30;
        (*mmu).cartridge_rom[1] = 0x10;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
    }
}

#[test]
fn jr_not_carry_backward_carry_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*cpu).regs.set_flags(Flags::C);
        (*mmu).cartridge_rom[0] = 0x30;
        (*mmu).cartridge_rom[1] = 0xFE;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
    }
}

#[test]
fn jr_not_carry_forward_carry_not_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0x30;
        (*mmu).cartridge_rom[1] = 0x10;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(12, tk);
        assert_eq!(0x00, r1.pc());
        assert_eq!(0x12, r2.pc());
    }
}

#[test]
fn jr_not_carry_backward_carry_not_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0x30;
        (*mmu).cartridge_rom[1] = 0xFE;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(12, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(0, r2.pc());
    }
}


#[test]
fn jr_carry_forward_carry_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*cpu).regs.set_flags(Flags::C);
        (*mmu).cartridge_rom[0] = 0x38;
        (*mmu).cartridge_rom[1] = 0x10;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(12, tk);
        assert_eq!(0x00, r1.pc());
        assert_eq!(0x12, r2.pc());
    }
}

#[test]
fn jr_carry_backward_carry_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*cpu).regs.set_flags(Flags::C);
        (*mmu).cartridge_rom[0] = 0x38;
        (*mmu).cartridge_rom[1] = 0xFE;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(12, tk);
        assert_eq!(0x00, r1.pc());
        assert_eq!(0x00, r2.pc());
    }
}

#[test]
fn jr_carry_forward_carry_not_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0x38;
        (*mmu).cartridge_rom[1] = 0x10;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
    }
}

#[test]
fn jr_carry_backward_carry_not_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0xA000);
        (*mmu).cartridge_rom[0] = 0x38;
        (*mmu).cartridge_rom[1] = 0xFE;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(8, tk);
        assert_eq!(0, r1.pc());
        assert_eq!(2, r2.pc());
    }
}

#[test]
fn jp_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0xC3;
        (*mmu).cartridge_rom[1] = 0x00;
        (*mmu).cartridge_rom[2] = 0x40;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(16, tk);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x4000, r2.pc());
    }
}

#[test]
fn jp_hl_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_hl(0x4000);
        (*mmu).cartridge_rom[0] = 0xE9;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(4, tk);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x4000, r2.pc());
    }
}

#[test]
fn jp_zero_with_zero_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_flags(Flags::Z);
        (*mmu).cartridge_rom[0] = 0xCA;
        (*mmu).cartridge_rom[1] = 0x00;
        (*mmu).cartridge_rom[2] = 0x40;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(16, tk);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x4000, r2.pc());
    }
}

#[test]
fn jp_zero_with_zero_not_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0xCA;
        (*mmu).cartridge_rom[1] = 0x00;
        (*mmu).cartridge_rom[2] = 0x40;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(12, tk);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x0003, r2.pc());
    }
}

#[test]
fn jp_carry_with_carry_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_flags(Flags::C);
        (*mmu).cartridge_rom[0] = 0xDA;
        (*mmu).cartridge_rom[1] = 0x00;
        (*mmu).cartridge_rom[2] = 0x40;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(16, tk);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x4000, r2.pc());
    }
}

#[test]
fn jp_carry_with_carry_not_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0xDA;
        (*mmu).cartridge_rom[1] = 0x00;
        (*mmu).cartridge_rom[2] = 0x40;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(12, tk);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x0003, r2.pc());
    }
}

#[test]
fn jp_not_zero_with_zero_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_flags(Flags::Z);
        (*mmu).cartridge_rom[0] = 0xC2;
        (*mmu).cartridge_rom[1] = 0x00;
        (*mmu).cartridge_rom[2] = 0x40;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(12, tk);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x0003, r2.pc());
    }
}

#[test]
fn jp_not_zero_with_zero_not_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0xC2;
        (*mmu).cartridge_rom[1] = 0x00;
        (*mmu).cartridge_rom[2] = 0x40;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(16, tk);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x4000, r2.pc());
    }
}

#[test]
fn jp_not_carry_with_carry_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_flags(Flags::C);
        (*mmu).cartridge_rom[0] = 0xD2;
        (*mmu).cartridge_rom[1] = 0x00;
        (*mmu).cartridge_rom[2] = 0x40;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(12, tk);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x0003, r2.pc());
    }
}

#[test]
fn jp_not_carry_with_carry_not_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0xD2;
        (*mmu).cartridge_rom[1] = 0x00;
        (*mmu).cartridge_rom[2] = 0x40;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(16, tk);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x4000, r2.pc());
    }
}

#[test]
fn call_ret_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0xCD;
        (*mmu).cartridge_rom[1] = 0x00;
        (*mmu).cartridge_rom[2] = 0x40;
        (*mmu).cartridge_rom[0x4000] = 0xC9;

        let r1 = (*cpu).registers();
        let t1 = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let t2 = (*cpu).cycle();
        let r3 = (*cpu).registers();
        (*cpu).cycle();
        let ie = (*cpu).interrupt_enabled;
        //let nie = (*cpu).next_interrupt_enabled;

        destroy((cpu, mmu));

        assert_eq!(24, t1);
        assert_eq!(16, t2);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x4000, r2.pc());
        assert_eq!(0x0003, r3.pc());
        assert_eq!(false, ie);
        //assert_eq!(false, nie);
    }
}

#[test]
fn call_reti_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0xCD;
        (*mmu).cartridge_rom[1] = 0x00;
        (*mmu).cartridge_rom[2] = 0x40;
        (*mmu).cartridge_rom[0x4000] = 0xD9;

        let r1 = (*cpu).registers();
        let t1 = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let t2 = (*cpu).cycle();
        let r3 = (*cpu).registers();
        (*cpu).cycle();
        let ie = (*cpu).interrupt_enabled;
        //let nie = (*cpu).next_interrupt_enabled;

        destroy((cpu, mmu));

        assert_eq!(24, t1);
        assert_eq!(16, t2);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x4000, r2.pc());
        assert_eq!(0x0003, r3.pc());
        assert_eq!(true, ie);
        //assert_eq!(true, nie);
    }
}

#[test]
fn call_carry_ret_carry_with_carry_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_flags(Flags::C);
        (*mmu).cartridge_rom[0] = 0xDC;
        (*mmu).cartridge_rom[1] = 0x00;
        (*mmu).cartridge_rom[2] = 0x40;
        (*mmu).cartridge_rom[0x4000] = 0xD8;

        let r1 = (*cpu).registers();
        let t1 = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let t2 = (*cpu).cycle();
        let r3 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(24, t1);
        assert_eq!(20, t2);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x4000, r2.pc());
        assert_eq!(0x0003, r3.pc());
    }
}

#[test]
fn call_carry_with_carry_not_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0xDC;
        (*mmu).cartridge_rom[1] = 0x00;
        (*mmu).cartridge_rom[2] = 0x40;

        let r1 = (*cpu).registers();
        let t1 = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(12, t1);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x0003, r2.pc());
    }
}

#[test]
fn call_zero_ret_zero_with_zero_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_flags(Flags::Z);
        (*mmu).cartridge_rom[0] = 0xCC;
        (*mmu).cartridge_rom[1] = 0x00;
        (*mmu).cartridge_rom[2] = 0x40;
        (*mmu).cartridge_rom[0x4000] = 0xC8;

        let r1 = (*cpu).registers();
        let t1 = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let t2 = (*cpu).cycle();
        let r3 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(24, t1);
        assert_eq!(20, t2);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x4000, r2.pc());
        assert_eq!(0x0003, r3.pc());
    }
}

#[test]
fn call_zero_with_zero_not_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0xCC;
        (*mmu).cartridge_rom[1] = 0x00;
        (*mmu).cartridge_rom[2] = 0x40;

        let r1 = (*cpu).registers();
        let t1 = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(12, t1);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x0003, r2.pc());
    }
}

#[test]
fn call_not_carry_ret_not_carry_with_carry_not_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0xD4;
        (*mmu).cartridge_rom[1] = 0x00;
        (*mmu).cartridge_rom[2] = 0x40;
        (*mmu).cartridge_rom[0x4000] = 0xD0;

        let r1 = (*cpu).registers();
        let t1 = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let t2 = (*cpu).cycle();
        let r3 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(24, t1);
        assert_eq!(20, t2);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x4000, r2.pc());
        assert_eq!(0x0003, r3.pc());
    }
}

#[test]
fn call_not_carry_with_carry_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_flags(Flags::C);
        (*mmu).cartridge_rom[0] = 0xD4;
        (*mmu).cartridge_rom[1] = 0x00;
        (*mmu).cartridge_rom[2] = 0x40;

        let r1 = (*cpu).registers();
        let t1 = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(12, t1);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x0003, r2.pc());
    }
}

#[test]
fn call_not_zero_ret_not_zero_with_zero_not_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0xC4;
        (*mmu).cartridge_rom[1] = 0x00;
        (*mmu).cartridge_rom[2] = 0x40;
        (*mmu).cartridge_rom[0x4000] = 0xC0;

        let r1 = (*cpu).registers();
        let t1 = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let t2 = (*cpu).cycle();
        let r3 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(24, t1);
        assert_eq!(20, t2);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x4000, r2.pc());
        assert_eq!(0x0003, r3.pc());
    }
}

#[test]
fn call_not_zero_with_zero_set_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_flags(Flags::Z);
        (*mmu).cartridge_rom[0] = 0xC4;
        (*mmu).cartridge_rom[1] = 0x00;
        (*mmu).cartridge_rom[2] = 0x40;

        let r1 = (*cpu).registers();
        let t1 = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(12, t1);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x0003, r2.pc());
    }
}

#[test]
fn rst00_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0xC7;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(16, tk);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x0000, r2.pc());
    }
}

#[test]
fn rst08_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0xCF;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(16, tk);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x0008, r2.pc());
    }
}

#[test]
fn rst10_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0xD7;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(16, tk);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x0010, r2.pc());
    }
}

#[test]
fn rst18_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0xDF;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(16, tk);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x0018, r2.pc());
    }
}

#[test]
fn rst20_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0xE7;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(16, tk);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x0020, r2.pc());
    }
}

#[test]
fn rst28_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0xEF;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(16, tk);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x0028, r2.pc());
    }
}

#[test]
fn rst30_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0xF7;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(16, tk);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x0030, r2.pc());
    }
}

#[test]
fn rst38_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*mmu).cartridge_rom[0] = 0xFF;

        let r1 = (*cpu).registers();
        let tk = (*cpu).cycle();
        let r2 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(16, tk);
        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x0038, r2.pc());
    }
}

#[test]
fn push_pop_test() {
    unsafe {
        let (cpu, mmu) = build();
        (*cpu).regs.set_af(0xAAF0);
        (*cpu).regs.set_bc(0xBBCC);
        (*cpu).regs.set_de(0xDDEE);
        (*cpu).regs.set_hl(0x8811);
        (*cpu).regs.set_sp(0xFFFE);
        (*mmu).cartridge_rom[0] = 0xF5;
        (*mmu).cartridge_rom[1] = 0xC5;
        (*mmu).cartridge_rom[2] = 0xD5;
        (*mmu).cartridge_rom[3] = 0xE5;
        (*mmu).cartridge_rom[4] = 0xF1;
        (*mmu).cartridge_rom[5] = 0xC1;
        (*mmu).cartridge_rom[6] = 0xD1;
        (*mmu).cartridge_rom[7] = 0xE1;

        let r1 = (*cpu).registers();
        let t1 = (*cpu).cycle();
        let t2 = (*cpu).cycle();
        let t3 = (*cpu).cycle();
        let t4 = (*cpu).cycle();
        let r2 = (*cpu).registers();
        let t5 = (*cpu).cycle();
        let t6 = (*cpu).cycle();
        let t7 = (*cpu).cycle();
        let t8 = (*cpu).cycle();
        let r3 = (*cpu).registers();
        (*cpu).cycle();

        destroy((cpu, mmu));

        assert_eq!(16, t1);
        assert_eq!(16, t2);
        assert_eq!(16, t3);
        assert_eq!(16, t4);
        assert_eq!(12, t5);
        assert_eq!(12, t6);
        assert_eq!(12, t7);
        assert_eq!(12, t8);

        assert_eq!(0x0000, r1.pc());
        assert_eq!(0x0004, r2.pc());
        assert_eq!(0x0008, r3.pc());

        assert_eq!(0xfffe, r1.sp());
        assert_eq!(0xfff6, r2.sp());
        assert_eq!(0xfffe, r3.sp());

        assert_eq!(0x8810, r3.af());
        assert_eq!(0xDDEE, r3.bc());
        assert_eq!(0xBBCC, r3.de());
        assert_eq!(0xAAF0, r3.hl());
    }
}