pub mod alu;
pub mod asm;
pub mod flags;
pub mod registers;
pub mod interrupt;

use std::ptr;
use registers::Registers;
use interrupt::Interrupt;
use crate::mmu::Mmu;

#[derive(Clone, Debug)]
pub struct Cpu {
    // Registers
    r: Registers,

    // Interruption Enable Register (IE)
    // - $FFFF (Hardware IO)
    ie_reg: Interrupt,

    // Interruption Flag (IF)
    // - $FF0F (Hardware IO)
    if_reg: Interrupt,

    // Master Interruption Enable
    int_enable: bool,

    // Next Master Interruption Enabled State
    // - auxiliar flag to emulate EI/DI change after execute next instruction
    next_int_enable: bool,

    next_pc: u16,

    // Memory Management Unit
    pub mmu: *mut Mmu,

}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            r: Registers::default(),
            ie_reg: Interrupt::empty(),
            if_reg: Interrupt::empty(),
            int_enable: false,
            next_int_enable: false,
            next_pc: 0,
            mmu: ptr::null_mut(),
        }
    }
}


impl Cpu {
    pub fn interruption_flag(&self) -> u8 { self.if_reg.bits() }

    pub fn set_interruption_flag(&mut self, flag: u8) {
        self.if_reg = Interrupt::from_bits_truncate(flag);
    }

    pub fn interruption_enable_register(&self) -> u8 { self.ie_reg.bits() }

    pub fn set_interruption_enable_register(&mut self, flag: u8) {
        self.ie_reg = Interrupt::from_bits_truncate(flag);
    }

    pub fn registers(&self) -> Registers { self.r.clone() }

    fn jump_absolute(&mut self, target: u16) {
        self.next_pc = target;
    }

    fn jump_absolute_if(&mut self, target: u16, cond: bool) {
        if cond {
            self.next_pc = target;
        }
    }

    fn jump_relative(&mut self, offset: u8) {
        self.next_pc = self.next_pc.wrapping_add((offset as i8) as u16)
    }

    fn jump_relative_if(&mut self, offset: u8, cond: bool) {
        if cond {
            self.next_pc = self.next_pc.wrapping_add((offset as i8) as u16)
        }
    }

    fn subroutine_call(&mut self, target: u16) {
        self.stack_push(self.next_pc);
        self.next_pc = target;
    }

    fn subroutine_call_if(&mut self, target: u16, cond: bool) {
        if cond {
            self.stack_push(self.next_pc);
            self.next_pc = target;
        }
    }

    fn subroutine_return(&mut self) {
        self.next_pc = self.stack_pop();
    }

    fn subroutine_return_if(&mut self, cond: bool) {
        if cond {
            self.next_pc = self.stack_pop();
        }
    }

    fn stack_push(&mut self, data: u16) {
        let be_bytes = data.to_be_bytes();

        let mut sp = self.r.sp();

        unsafe {
            sp = sp.wrapping_sub(1);
            (*self.mmu).write(sp, be_bytes[0]);
            sp = sp.wrapping_sub(1);
            (*self.mmu).write(sp, be_bytes[1]);
        }

        self.r.set_sp(sp);
    }

    fn stack_pop(&mut self) -> u16 {
        let mut sp = self.r.sp();
        let lsb: u8;
        let msb: u8;

        unsafe {
            lsb = (*self.mmu).read(sp);
            sp = sp.wrapping_add(1);

            msb = (*self.mmu).read(sp);
            sp = sp.wrapping_add(1);
        }

        self.r.set_sp(sp);
        u16::from_be_bytes([msb, lsb])
    }

    fn interrupt_service_routine(&mut self) -> bool {
        let int_enable = self.int_enable;
        self.int_enable = self.next_int_enable;

        if !int_enable {
            return false;
        }

        let int = self.ie_reg & self.if_reg;

        if int.vertical_blank() {
            self.subroutine_call(0x40);
        } else if int.lcdc_status() {
            self.subroutine_call(0x48);
        } else if int.timer_overflow() {
            self.subroutine_call(0x50);
        } else if int.serial_transfer_complete() {
            self.subroutine_call(0x58);
        } else if int.high_to_low_pin10_to_pin_13() {
            self.subroutine_call(0x60);
        } else {
            return false;
        }

        self.int_enable = false;
        return true;
    }

    pub fn fetch_decode_execute_store_cycle(&mut self) -> u64 {
        if self.interrupt_service_routine() {
            return 4
        }

        let pc = self.r.pc();

        // Fetch
        let opcode : u8;
        let immediate8 : u8;
        let immediate16 : u16;

        unsafe {
            opcode = (*self.mmu).read(pc);
            immediate8 = (*self.mmu).read(pc + 1);
            immediate16 = u16::from_le_bytes([immediate8, (*self.mmu).read(pc + 2)]);
        }
        self.next_pc = pc + asm::instruction_size(opcode);

        //println!("${:04x} ${:02x} ${:02x} ${:04x} {:<15} {:02x?} {:?}", pc, opcode, immediate8, immediate16, disassemble(opcode, immediate8, immediate16), self.r, self.alu.flags);
        //if pc == 0xc7d2 {
        //    panic!();
        //}

        // Decode => Execute => Store
        match opcode {
            0x00 => {
                // NOP
            }
            0x01 => {
                // LD BC, $0000
                self.r.set_bc(immediate16);
            }
            0x02 => {
                // LD (BC), A
                unsafe {
                    (*self.mmu).write(self.r.bc(), self.r.a());
                }
            },
            0x03 => {
                // INC BC
                let bc = alu::inc16(self.r.bc());
                self.r.set_bc(bc);
            }
            0x04 => {
                // INC B
                let (flags, b) = alu::inc(self.r.flags(), self.r.b());
                self.r.set_flags(flags);
                self.r.set_b(b);
            }
            0x05 => {
                // DEC B
                let (flags, b) = alu::dec(self.r.flags(), self.r.b());
                self.r.set_flags(flags);
                self.r.set_b(b);
            }
            0x06 => {
                // LD B, $00
                self.r.set_b(immediate8);
            }
            0x07 => {
                // RLCA
                let (flags, a) = alu::rlc(self.r.a());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x08 => {
                // LD ($0000),SP
                let le_bytes = self.r.sp().to_le_bytes();
                unsafe {
                    (*self.mmu).write(immediate16, le_bytes[0]);
                    (*self.mmu).write(immediate16.wrapping_add(1), le_bytes[1]);
                }
            }
            0x09 => {
                // ADD HL, BC
                let (flags, hl) = alu::add16(self.r.flags(), self.r.hl(), self.r.bc());
                self.r.set_flags(flags);
                self.r.set_hl(hl);
            }
            0x0A => {
                // LD A, (BC)
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(self.r.bc());
                }
                self.r.set_a(data);
            }
            0x0B => {
                // DEC BC
                let bc = alu::dec16(self.r.bc());
                self.r.set_bc(bc);
            }
            0x0C => {
                // INC C
                let (flags, c) = alu::inc(self.r.flags(), self.r.c());
                self.r.set_flags(flags);
                self.r.set_c(c);
            }
            0x0D => {
                // DEC C
                let (flags, c) = alu::dec(self.r.flags(), self.r.c());
                self.r.set_flags(flags);
                self.r.set_c(c);
            }
            0x0E => {
                // LD C, $00
                self.r.set_c(immediate8);
            }
            0x0F => {
                // RRCA
                let (flags, a) = alu::rrc(self.r.a());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x10 => {
                // STOP 0
            }
            0x11 => {
                // LD DE, $0000
                self.r.set_de(immediate16);
            }
            0x12 => {
                // LD (DE), A
                unsafe {
                    (*self.mmu).write(self.r.de(), self.r.a());
                }
            }
            0x13 => {
                // INC DE
                let de = alu::inc16(self.r.de());
                self.r.set_de(de);
            }
            0x14 => {
                // INC D
                let (flags, d) = alu::inc(self.r.flags(), self.r.d());
                self.r.set_flags(flags);
                self.r.set_d(d);
            }
            0x15 => {
                // DEC D
                let (flags, d) = alu::dec(self.r.flags(), self.r.d());
                self.r.set_flags(flags);
                self.r.set_d(d);
            }
            0x16 => {
                // LD D, $00
                self.r.set_d(immediate8);
            }
            0x17 => {
                // RLA
                let (flags, a) = alu::rl(self.r.flags(), self.r.a());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x18 => {
                // JR $00
                self.jump_relative(immediate8);
            }
            0x19 => {
                // ADD HL, DE
                let (flags, hl) = alu::add16(self.r.flags(), self.r.hl(), self.r.de());
                self.r.set_flags(flags);
                self.r.set_hl(hl);
            }
            0x1A => {
                // LD A, (DE)
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(self.r.de());
                }
                self.r.set_a(data);
            }
            0x1B => {
                // DEC DE
                let de = alu::dec16(self.r.de());
                self.r.set_de(de);
            }
            0x1C => {
                // INC E
                let (flags, e) = alu::inc(self.r.flags(), self.r.e());
                self.r.set_flags(flags);
                self.r.set_e(e);
            }
            0x1D => {
                // DEC E
                let (flags, e) = alu::dec(self.r.flags(), self.r.e());
                self.r.set_flags(flags);
                self.r.set_e(e);
            }
            0x1E => {
                // LD E, $00
                self.r.set_e(immediate8);
            }
            0x1F => {
                // RRA
                let (flags, a) = alu::rr(self.r.flags(), self.r.a());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x20 => {
                // JR NZ $00
                self.jump_relative_if(immediate8, !self.r.flags().zero());
            }
            0x21 => {
                // LD HL, $0000
                self.r.set_hl(immediate16);
            }
            0x22 => {
                // LDI (HL), A
                let hl = self.r.hl();
                unsafe {
                    (*self.mmu).write(hl, self.r.a());
                }
                let hl = hl.wrapping_add(1);
                self.r.set_hl(hl);
            }
            0x23 => {
                // INC HL
                let hl = alu::inc16(self.r.hl());
                self.r.set_hl(hl);
            }
            0x24 => {
                // INC H
                let (flags, h) = alu::inc(self.r.flags(), self.r.h());
                self.r.set_flags(flags);
                self.r.set_h(h);
            }
            0x25 => {
                // DEC H
                let (flags, h) = alu::dec(self.r.flags(), self.r.h());
                self.r.set_flags(flags);
                self.r.set_h(h);
            }
            0x26 => {
                // LD H, $00
                self.r.set_h(immediate8);
            }
            0x27 => {
                // DAA
                let (flags, a) = alu::daa(self.r.flags(), self.r.a());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x28 => {
                // JR Z $00
                self.jump_relative_if(immediate8, self.r.flags().zero());
            }
            0x29 => {
                // ADD HL, HL
                let (flags, hl) = alu::add16(self.r.flags(), self.r.hl(), self.r.hl());
                self.r.set_flags(flags);
                self.r.set_hl(hl);
            }
            0x2A => {
                // LDI A, (HL)
                let addr = self.r.hl();
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(addr);
                }
                self.r.set_a(data);
                self.r.set_hl(addr.wrapping_add(1));
            }
            0x2B => {
                // DEC HL
                let hl = alu::dec16(self.r.hl());
                self.r.set_hl(hl);
            }
            0x2C => {
                // INC L
                let (flags, l) = alu::inc(self.r.flags(), self.r.l());
                self.r.set_flags(flags);
                self.r.set_l(l);
            }
            0x2D => {
                // DEC L
                let (flags, l) = alu::dec(self.r.flags(), self.r.l());
                self.r.set_flags(flags);
                self.r.set_l(l);
            }
            0x2E => {
                // LD L, $00
                self.r.set_l(immediate8);
            }
            0x2F => {
                // CPL
                let (flags, a) = alu::complement(self.r.flags(), self.r.a());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x30 => {
                // JR NC $00
                self.jump_relative_if(immediate8, !self.r.flags().carry());
            }
            0x31 => {
                // LD SP, $0000
                self.r.set_sp(immediate16);
            }
            0x32 => {
                // LDD (HL), A
                let hl = self.r.hl();
                unsafe {
                    (*self.mmu).write(hl, self.r.a());
                }
                let hl = hl.wrapping_sub(1);
                self.r.set_hl(hl);
            }
            0x33 => {
                // INC SP
                let sp = self.r.sp().wrapping_add(1);
                self.r.set_sp(sp);
            }
            0x34 => {
                // INC (HL)
                let addr = self.r.hl();
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(addr);
                }

                let (flags, data) = alu::inc(self.r.flags(), data);
                self.r.set_flags(flags);
                unsafe {
                    (*self.mmu).write(addr, data);
                }
            }
            0x35 => {
                // DEC (HL)
                let addr = self.r.hl();
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(addr);
                }

                let (flags, data) = alu::dec(self.r.flags(), data);
                self.r.set_flags(flags);
                unsafe {
                    (*self.mmu).write(addr, data);
                }
            }
            0x36 => {
                // LD (HL), $00
                unsafe {
                    (*self.mmu).write(self.r.hl(), immediate8);
                }
            }
            0x37 => {
                // SCF
                let mut flags = self.r.flags();
                flags.set_carry();

                self.r.set_flags(flags);
            }
            0x38 => {
                // JR C $00
                self.jump_relative_if(immediate8, self.r.flags().carry());
            }
            0x39 => {
                // ADD HL, SP
                let (flags, hl) = alu::add16(self.r.flags(), self.r.hl(), self.r.sp());
                self.r.set_flags(flags);
                self.r.set_hl(hl);
            }
            0x3A => {
                // LDD A, (HL)
                let addr = self.r.hl();
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(addr);
                }
                self.r.set_a(data);
                self.r.set_hl(addr.wrapping_sub(1));
            }
            0x3B => {
                // DEC SP
                self.r.set_sp(self.r.sp().wrapping_sub(1));
            }
            0x3C => {
                // INC A
                let (flags, a) = alu::inc(self.r.flags(), self.r.a());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x3D => {
                // DEC A
                let (flags, a) = alu::dec(self.r.flags(), self.r.a());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x3E => {
                // LD A, $00
                self.r.set_a(immediate8);
            }
            0x3F => {
                // CCF
                let mut flags = self.r.flags();
                flags.reset_carry();

                self.r.set_flags(flags);
            }
            0x40 => {
                // LD B, B
            }
            0x41 => {
                // LD B, C
                self.r.set_b(self.r.c());
            }
            0x42 => {
                // LD B, D
                self.r.set_b(self.r.d());
            }
            0x43 => {
                // LD B, E
                self.r.set_b(self.r.e());
            }
            0x44 => {
                // LD B, H
                self.r.set_b(self.r.h());
            }
            0x45 => {
                // LD B, L
                self.r.set_b(self.r.l());
            }
            0x46 => {
                // LD B, (HL)
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(self.r.hl());
                }
                self.r.set_b(data);
            }
            0x47 => {
                // LD B, A
                self.r.set_b(self.r.a());
            }
            0x48 => {
                // LD C, B
                self.r.set_c(self.r.b());
            }
            0x49 => {
                // LD C, C
            }
            0x4A => {
                // LD C, D
                self.r.set_c(self.r.d());
            }
            0x4B => {
                // LD C, E
                self.r.set_c(self.r.e());
            }
            0x4C => {
                // LD C, H
                self.r.set_c(self.r.h());
            }
            0x4D => {
                // LD C, L
                self.r.set_c(self.r.l());
            }
            0x4E => {
                // LD C, (HL)
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(self.r.hl());
                }
                self.r.set_c(data);
            }
            0x4F => {
                // LD C, A
                self.r.set_c(self.r.a());
            }
            0x50 => {
                // LD D, B
                self.r.set_d(self.r.b());
            }
            0x51 => {
                // LD D, C
                self.r.set_d(self.r.c());
            }
            0x52 => {
                // LD D, D
            }
            0x53 => {
                // LD D, E
                self.r.set_d(self.r.e());
            }
            0x54 => {
                // LD D, H
                self.r.set_d(self.r.h());
            }
            0x55 => {
                // LD D, L
                self.r.set_d(self.r.l());
            }
            0x56 => {
                // LD D, (HL)
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(self.r.hl());
                }
                self.r.set_d(data);
            }
            0x57 => {
                // LD D, A
                self.r.set_d(self.r.a());
            }
            0x58 => {
                // LD E, B
                self.r.set_e(self.r.b());
            }
            0x59 => {
                // LD E, C
                self.r.set_e(self.r.c());
            }
            0x5A => {
                // LD E, D
                self.r.set_e(self.r.d());
            }
            0x5B => {
                // LD E, E
            }
            0x5C => {
                // LD E, H
                self.r.set_e(self.r.h());
            }
            0x5D => {
                // LD E, L
                self.r.set_e(self.r.l());
            }
            0x5E => {
                // LD E, (HL)
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(self.r.hl());
                }
                self.r.set_e(data);
            }
            0x5F => {
                // LD E, A
                self.r.set_e(self.r.a());
            }
            0x60 => {
                // LD H, B
                self.r.set_h(self.r.b());
            }
            0x61 => {
                // LD H, C
                self.r.set_h(self.r.c());
            }
            0x62 => {
                // LD H, D
                self.r.set_h(self.r.d());
            }
            0x63 => {
                // LD H, E
                self.r.set_h(self.r.e());
            }
            0x64 => {
                // LD H, H
            }
            0x65 => {
                // LD H, L
                self.r.set_h(self.r.l());
            }
            0x66 => {
                // LD H, (HL)
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(self.r.hl());
                }
                self.r.set_h(data);
            }
            0x67 => {
                // LD H, A
                self.r.set_h(self.r.a());
            }
            0x68 => {
                // LD L, B
                self.r.set_l(self.r.b());
            }
            0x69 => {
                // LD L, C
                self.r.set_l(self.r.c());
            }
            0x6A => {
                // LD L, D
                self.r.set_l(self.r.d());
            }
            0x6B => {
                // LD L, E
                self.r.set_l(self.r.e());
            }
            0x6C => {
                // LD L, H
                self.r.set_l(self.r.h());
            }
            0x6D => {
                // LD L, L
            }
            0x6E => {
                // LD L, (HL)
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(self.r.hl());
                }
                self.r.set_l(data);
            }
            0x6F => {
                // LD L, A
                self.r.set_l(self.r.a());
            }
            0x70 => {
                // LD (HL), B
                unsafe {
                    (*self.mmu).write(self.r.hl(), self.r.b());
                }
            }
            0x71 => {
                // LD (HL), C
                unsafe {
                    (*self.mmu).write(self.r.hl(), self.r.c());
                }
            }
            0x72 => {
                // LD (HL), D
                unsafe {
                    (*self.mmu).write(self.r.hl(), self.r.d());
                }
            }
            0x73 => {
                // LD (HL), E
                unsafe {
                    (*self.mmu).write(self.r.hl(), self.r.e());
                }
            }
            0x74 => {
                // LD (HL), H
                unsafe {
                    (*self.mmu).write(self.r.hl(), self.r.h());
                }
            }
            0x75 => {
                // LD (HL), L
                unsafe {
                    (*self.mmu).write(self.r.hl(), self.r.l());
                }
            }
            0x76 => {
                // LD (HL), (HL)
                unsafe {
                    let data = (*self.mmu).read(self.r.hl());
                    (*self.mmu).write(self.r.hl(), data);
                }
            }
            0x77 => {
                // LD (HL), A
                unsafe {
                    (*self.mmu).write(self.r.hl(), self.r.a());
                }
            }
            0x78 => {
                // LD A, B
                self.r.set_a(self.r.b());
            }
            0x79 => {
                // LD A, C
                self.r.set_a(self.r.c());
            }
            0x7A => {
                // LD A, D
                self.r.set_a(self.r.d());
            }
            0x7B => {
                // LD A, E
                self.r.set_a(self.r.e());
            }
            0x7C => {
                // LD A, H
                self.r.set_a(self.r.h());
            }
            0x7D => {
                // LD A, L
                self.r.set_a(self.r.l());
            }
            0x7E => {
                // LD A, (HL)
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(self.r.hl());
                }
                self.r.set_a(data);
            }
            0x7F => {
                // LD A, A
            }
            0x80 => {
                // ADD A, B
                self.r.set_a(self.r.b());
            }
            0x81 => {
                // ADD A, C
                let (flags, a) = alu::add(self.r.a(), self.r.c());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x82 => {
                // ADD A, D
                let (flags, a) = alu::add(self.r.a(), self.r.d());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x83 => {
                // ADD A, E
                let (flags, a) = alu::add(self.r.a(), self.r.e());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x84 => {
                // ADD A, H
                let (flags, a) = alu::add(self.r.a(), self.r.h());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x85 => {
                // ADD A, L
                let (flags, a) = alu::add(self.r.a(), self.r.l());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x86 => {
                // ADD A, (HL)
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(self.r.hl());
                }
                let (flags, a) = alu::add(self.r.a(), data);
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x87 => {
                // ADD A, A
                let (flags, a) = alu::add(self.r.a(), self.r.a());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x88 => {
                // ADC A, B
                let (flags, a) = alu::adc(self.r.flags(), self.r.a(), self.r.b());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x89 => {
                // ADC A, C
                let (flags, a) = alu::adc(self.r.flags(), self.r.a(), self.r.c());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x8A => {
                // ADC A, D
                let (flags, a) = alu::adc(self.r.flags(), self.r.a(), self.r.d());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x8B => {
                // ADC A, E
                let (flags, a) = alu::adc(self.r.flags(), self.r.a(), self.r.e());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x8C => {
                // ADC A, H
                let (flags, a) = alu::adc(self.r.flags(), self.r.a(), self.r.h());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x8D => {
                // ADC A, L
                let (flags, a) = alu::adc(self.r.flags(), self.r.a(), self.r.l());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x8E => {
                // ADC A, (HL)
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(self.r.hl());
                }

                let (flags, a) = alu::adc(self.r.flags(), self.r.a(), data);
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x8F => {
                // ADC A, A
                let (flags, a) = alu::adc(self.r.flags(), self.r.a(), self.r.a());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x90 => {
                // SUB A, B
                let (flags, a) = alu::sub(self.r.a(), self.r.b());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x91 => {
                // SUB A, C
                let (flags, a) = alu::sub(self.r.a(), self.r.c());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x92 => {
                // SUB A, D
                let (flags, a) = alu::sub(self.r.a(), self.r.d());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x93 => {
                // SUB A, E
                let (flags, a) = alu::sub(self.r.a(), self.r.e());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x94 => {
                // SUB A, H
                let (flags, a) = alu::sub(self.r.a(), self.r.h());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x95 => {
                // SUB A, L
                let (flags, a) = alu::sub(self.r.a(), self.r.l());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x96 => {
                // SUB A, (HL)
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(self.r.hl());
                }

                let (flags, a) = alu::sub(self.r.a(), data);
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x97 => {
                // SUB A, A
                let (flags, a) = alu::sub(self.r.a(), self.r.a());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x98 => {
                // SBC A, B
                let (flags, a) = alu::sbc(self.r.flags(), self.r.a(), self.r.b());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x99 => {
                // SBC A, C
                let (flags, a) = alu::sbc(self.r.flags(), self.r.a(), self.r.c());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x9A => {
                // SBC A, D
                let (flags, a) = alu::sbc(self.r.flags(), self.r.a(), self.r.d());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x9B => {
                // SBC A, E
                let (flags, a) = alu::sbc(self.r.flags(), self.r.a(), self.r.e());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x9C => {
                // SBC A, H
                let (flags, a) = alu::sbc(self.r.flags(), self.r.a(), self.r.h());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x9D => {
                // SBC A, L
                let (flags, a) = alu::sbc(self.r.flags(), self.r.a(), self.r.l());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x9E => {
                // SBC A, (HL)
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(self.r.hl());
                }

                let (flags, a) = alu::sbc(self.r.flags(), self.r.a(), data);
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0x9F => {
                // SBC A, A
                let (flags, a) = alu::sbc(self.r.flags(), self.r.a(), self.r.a());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xA0 => {
                // AND A, B
                let (flags, a) = alu::and(self.r.a(), self.r.b());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xA1 => {
                // AND A, C
                let (flags, a) = alu::and(self.r.a(), self.r.c());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xA2 => {
                // AND A, D
                let (flags, a) = alu::and(self.r.a(), self.r.d());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xA3 => {
                // AND A, E
                let (flags, a) = alu::and(self.r.a(), self.r.e());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xA4 => {
                // AND A, H
                let (flags, a) = alu::and(self.r.a(), self.r.h());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xA5 => {
                // AND A, L
                let (flags, a) = alu::and(self.r.a(), self.r.l());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xA6 => {
                // AND A, (HL)
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(self.r.hl());
                }

                let (flags, a) = alu::and(self.r.a(), data);
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xA7 => {
                // AND A, A
                let (flags, a) = alu::and(self.r.a(), self.r.a());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xA8 => {
                // XOR A, B
                let (flags, a) = alu::xor(self.r.a(), self.r.b());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xA9 => {
                // XOR A, C
                let (flags, a) = alu::xor(self.r.a(), self.r.c());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xAA => {
                // XOR A, D
                let (flags, a) = alu::xor(self.r.a(), self.r.d());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xAB => {
                // XOR A, E
                let (flags, a) = alu::xor(self.r.a(), self.r.e());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xAC => {
                // XOR A, H
                let (flags, a) = alu::xor(self.r.a(), self.r.h());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xAD => {
                // XOR A, L
                let (flags, a) = alu::xor(self.r.a(), self.r.l());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xAE => {
                // XOR A, (HL)
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(self.r.hl());
                }

                let (flags, a) = alu::xor(self.r.a(), data);
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xAF => {
                // XOR A, A
                let (flags, a) = alu::xor(self.r.a(), self.r.a());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xB0 => {
                // OR A, B
                let (flags, a) = alu::or(self.r.a(), self.r.b());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xB1 => {
                // OR A, C
                let (flags, a) = alu::or(self.r.a(), self.r.c());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xB2 => {
                // OR A, D
                let (flags, a) = alu::or(self.r.a(), self.r.d());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xB3 => {
                // OR A, E
                let (flags, a) = alu::or(self.r.a(), self.r.e());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xB4 => {
                // OR A, H
                let (flags, a) = alu::or(self.r.a(), self.r.h());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xB5 => {
                // OR A, L
                let (flags, a) = alu::or(self.r.a(), self.r.l());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xB6 => {
                // OR A, (HL)
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(self.r.hl());
                }

                let (flags, a) = alu::or(self.r.a(), data);
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xB7 => {
                // OR A, A
                let (flags, a) = alu::or(self.r.a(), self.r.a());
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xB8 => {
                // CP A, B
                let (flags, _) = alu::sub(self.r.a(), self.r.b());
                self.r.set_flags(flags);
            }
            0xB9 => {
                // CP A, C
                let (flags, _) = alu::sub(self.r.a(), self.r.c());
                self.r.set_flags(flags);
            }
            0xBA => {
                // CP A, D
                let (flags, _) = alu::sub(self.r.a(), self.r.d());
                self.r.set_flags(flags);
            }
            0xBB => {
                // CP A, E
                let (flags, _) = alu::sub(self.r.a(), self.r.e());
                self.r.set_flags(flags);
            }
            0xBC => {
                // CP A, H
                let (flags, _) = alu::sub(self.r.a(), self.r.h());
                self.r.set_flags(flags);
            }
            0xBD => {
                // CP A, L
                let (flags, _) = alu::sub(self.r.a(), self.r.l());
                self.r.set_flags(flags);
            }
            0xBE => {
                // CP A, (HL)
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(self.r.hl());
                }

                let (flags, _) = alu::sub(self.r.a(), data);
                self.r.set_flags(flags);
            }
            0xBF => {
                // CP A, A
                let (flags, _) = alu::sub(self.r.a(), self.r.a());
                self.r.set_flags(flags);
            }
            0xC0 => {
                // RET NZ
                self.subroutine_return_if(!self.r.flags().zero());
            }
            0xC1 => {
                // POP BC
                let bc = self.stack_pop();
                self.r.set_bc(bc);
            }
            0xC2 => {
                // JP NZ $0000
                self.jump_absolute_if(immediate16, !self.r.flags().zero());
            }
            0xC3 => {
                // JP $0000
                self.jump_absolute(immediate16);
            }
            0xC4 => {
                // CALL NZ $0000
                self.subroutine_call_if(immediate16, !self.r.flags().zero());
            }
            0xC5 => {
                // PUSH BC
                self.stack_push(self.r.bc());
            }
            0xC6 => {
                // ADD A, $00
                let (flags, a) = alu::add(self.r.a(), immediate8);
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xC7 => {
                // RST $00
                self.subroutine_call(0x00);
            }
            0xC8 => {
                // RET Z
                self.subroutine_return_if(self.r.flags().zero());
            }
            0xC9 => {
                // RET
                self.subroutine_return();
            }
            0xCA => {
                // JP Z $0000
                self.jump_absolute_if(immediate16, self.r.flags().zero());
            }
            0xCB => {
                // PREFIX CB (Logic Instruction Extension)

                let arg = match immediate8 & 0x7 {
                    0x0 => self.r.b(),
                    0x1 => self.r.c(),
                    0x2 => self.r.d(),
                    0x3 => self.r.e(),
                    0x4 => self.r.h(),
                    0x5 => self.r.l(),
                    0x6 => {
                        let data : u8;
                        unsafe {
                            data = (*self.mmu).read(self.r.hl());
                        }
                        data
                    }
                    0x7 => self.r.a(),
                    _ => panic!()
                };

                let (flags, ret) = match immediate8 {
                    0x00..=0x07 => {
                        // RLC
                        alu::rlc(arg)
                    }
                    0x08..=0x0F => {
                        // RRC
                        alu::rrc(arg)
                    }
                    0x10..=0x17 => {
                        // RL
                        alu::rl(self.r.flags(), arg)
                    }
                    0x18..=0x1F => {
                        // RR
                        alu::rr(self.r.flags(), arg)
                    }
                    0x20..=0x27 => {
                        // SLA
                        alu::sla(arg)
                    }
                    0x28..=0x2F => {
                        // SRA
                        alu::sra(arg)
                    }
                    0x30..=0x37 => {
                        // SWAP
                        alu::nibble_swap(arg)
                    }
                    0x38..=0x3F => {
                        // SRL
                        alu::srl(arg)
                    }
                    0x40..=0x7F => {
                        // BIT
                        let bit_index = (immediate8 - 0x40) / 8;
                        let flags = alu::test_bit(self.r.flags(), arg, bit_index);
                        (flags, arg)
                    }
                    0x80..=0xBF => {
                        // RES
                        let bit_index = (immediate8 - 0x80) / 8;
                        (self.r.flags(), alu::reset_bit(arg, bit_index))
                    }
                    0xC0..=0xFF => {
                        // SET
                        let bit_index = (immediate8 - 0xC0) / 8;
                        (self.r.flags(), alu::set_bit(arg, bit_index))
                    }
                };

                self.r.set_flags(flags);

                if ret != arg {
                    match immediate8 & 0b111 {
                        0x0 => self.r.set_a(ret),
                        0x1 => self.r.set_c(ret),
                        0x2 => self.r.set_d(ret),
                        0x3 => self.r.set_e(ret),
                        0x4 => self.r.set_h(ret),
                        0x5 => self.r.set_l(ret),
                        0x6 => unsafe {
                            (*self.mmu).write(self.r.hl(), ret);
                        }
                        0x7 => self.r.set_a(ret),
                        _ => panic!()
                    }
                }
            }
            0xCC => {
                // CALL Z $0000
                self.subroutine_call_if(immediate16, self.r.flags().zero())
            }
            0xCD => {
                // CALL $0000
                self.subroutine_call(immediate16)
            }
            0xCE => {
                // ADC A, $00
                let (flags, a) = alu::adc(self.r.flags(), self.r.a(), immediate8);
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xCF => {
                // RST $08
                self.subroutine_call(0x08);
            }
            0xD0 => {
                // RET NC
                self.subroutine_return_if(!self.r.flags().carry());
            }
            0xD1 => {
                // POP DE
                let de = self.stack_pop();
                self.r.set_de(de);
            }
            0xD2 => {
                // JP NC $0000
                self.jump_absolute_if(immediate16, !self.r.flags().carry());
            }
            0xD3 => {
                // [D3] - INVALID
            }
            0xD4 => {
                // CALL NC $0000
                self.subroutine_call_if(immediate16, !self.r.flags().carry())
            }
            0xD5 => {
                // PUSH DE
                self.stack_push(self.r.de());
            }
            0xD6 => {
                // SUB A, $00
                let (flags, a) = alu::sub(self.r.a(), immediate8);
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xD7 => {
                // RST $10
                self.subroutine_call(0x10);
            }
            0xD8 => {
                // RET C
                self.subroutine_return_if(self.r.flags().carry());
            }
            0xD9 => {
                // RETI
                self.subroutine_return();
                self.int_enable = true;
            }
            0xDA => {
                // JP C $0000
                self.jump_absolute_if(immediate16, self.r.flags().carry());
            }
            0xDB => {
                // [DB] - INVALID
            }
            0xDC => {
                // CALL C $0000
                self.subroutine_call_if(immediate16, self.r.flags().carry())
            }
            0xDD => {
                // [DD] - INVALID
            }
            0xDE => {
                // SBC A, $00
                let (flags, a) = alu::sbc(self.r.flags(), self.r.a(), immediate8);
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xDF => {
                // RST $18
                self.subroutine_call(0x18);
            }
            0xE0 => {
                // LDH ($00), A
                let addr: u16 = 0xff00u16 | immediate8 as u16;
                let data = self.r.a();
                unsafe {
                    (*self.mmu).write(addr, data);
                }
            }
            0xE1 => {
                // POP HL
                let hl = self.stack_pop();
                self.r.set_hl(hl);
            }
            0xE2 => {
                // LDH (C), A
                let addr = 0xff00u16 | self.r.c() as u16;
                let data = self.r.a();
                unsafe {
                    (*self.mmu).write(addr, data);
                }
            }
            0xE3 => {
                // [E3] - INVALID
            }
            0xE4 => {
                // [E4] - INVALID
            }
            0xE5 => {
                // PUSH HL
                self.stack_push(self.r.hl());
            }
            0xE6 => {
                // AND $00
                let (flags, a) = alu::and(self.r.a(), immediate8);
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xE7 => {
                // RST $20
                self.subroutine_call(0x20);
            }
            0xE8 => {
                // ADD SP, $00
                let mut value = self.r.sp();
                if value < 0x80 {
                    value = value.wrapping_add(immediate8 as u16);
                } else {
                    value = value.wrapping_sub((0xFFu8.wrapping_sub(immediate8).wrapping_add(1)) as u16);
                }

                let mut flags = self.r.flags();
                flags.set_carry();
                flags.set_half();

                self.r.set_flags(flags);
                self.r.set_sp(value);
            }
            0xE9 => {
                // JP HL
                self.jump_absolute(self.r.hl());
            }
            0xEA => {
                // LD ($0000), A
                unsafe {
                    (*self.mmu).write(immediate16, self.r.a());
                }
            }
            0xEB => {
                // [EB] - INVALID
            }
            0xEC => {
                // [EC] - INVALID
            }
            0xED => {
                // [ED] - INVALID
            }
            0xEE => {
                // XOR $00
                let (flags, a) = alu::xor(self.r.a(), immediate8);
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xEF => {
                // RST $28
                self.subroutine_call(0x28);
            }
            0xF0 => {
                // LDH A, ($00)
                let addr: u16 = 0xff00u16 | immediate8 as u16;
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(addr);
                }
                self.r.set_a(data);
            }
            0xF1 => {
                // POP AF
                let af = self.stack_pop();
                self.r.set_af(af);
            }
            0xF2 => {
                // LD A, ($FF00+C)
                let addr = 0xff00u16 | self.r.c() as u16;
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(addr);
                }
                self.r.set_a(data);
            }
            0xF3 => {
                // DI
                self.next_int_enable = false;
            }
            0xF4 => {
                // [F4] - INVALID
            }
            0xF5 => {
                // PUSH AF
                self.stack_push(self.r.af());
            }
            0xF6 => {
                // OR $00
                let (flags, a) = alu::or(self.r.a(), immediate8);
                self.r.set_flags(flags);
                self.r.set_a(a);
            }
            0xF7 => {
                // RST $30
                self.subroutine_call(0x30);
            }
            0xF8 => {
                // LD HL,SP+$00
                let mut value = self.r.sp();
                if value < 0x80 {
                    value = value.wrapping_add(immediate8 as u16);
                } else {
                    value = value.wrapping_sub((0xFFu8.wrapping_sub(immediate8).wrapping_add(1)) as u16);
                }
                let mut flags = self.r.flags();
                flags.set_carry();
                flags.set_half();

                self.r.set_flags(flags);
                self.r.set_hl(value);
            }
            0xF9 => {
                // LD SP, HL
                self.r.set_sp(self.r.hl());
            }
            0xFA => {
                // LD A, ($0000)
                let data : u8;
                unsafe {
                    data = (*self.mmu).read(immediate16);
                }
                self.r.set_a(data);
            }
            0xFB => {
                // EI
                self.next_int_enable = true;
            }
            0xFC => {
                // [FC] - INVALID;
            }
            0xFD => {
                // [FD] - INVALID;
            }
            0xFE => {
                // CP $00
                let (flags, _) = alu::sub(self.r.a(), immediate8);
                self.r.set_flags(flags);
            }
            0xFF => {
                // RST $38
                self.subroutine_call(0x38);
            }
        }

        self.r.set_pc(self.next_pc);
        asm::instruction_ticks(opcode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn nop_test() {
        unsafe {
            let (cpu, mmu) = build();
            let r1 = (*cpu).registers();
            let tk = (*cpu).fetch_decode_execute_store_cycle();
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
    fn ld_bc_d16_test() {
        unsafe {
            let (cpu, mmu) = build();
            (*mmu).cartridge_rom[0] = 0x01;
            (*mmu).cartridge_rom[1] = 0xEF;
            (*mmu).cartridge_rom[2] = 0xBE;

            let r1 = (*cpu).registers();
            let tk = (*cpu).fetch_decode_execute_store_cycle();
            let r2 = (*cpu).registers();

            destroy((cpu, mmu));

            assert_eq!(12, tk);
            assert_eq!(0, r1.pc());
            assert_eq!(3, r2.pc());
            assert_eq!(0x0000, r1.bc());
            assert_eq!(0xBEEF, r2.bc());

            let mut rr = r2.clone();
            rr.set_pc(r1.pc());
            rr.set_bc(r1.bc());
            assert_eq!(rr, r1);
        }
    }
}