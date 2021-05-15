pub mod alu;
pub mod asm;
pub mod flags;
pub mod registers;
pub mod interrupt;
pub mod interrupt_service;

#[cfg(test)]
mod tests;

use std::ptr;
use registers::Registers;
use interrupt::Interrupt;
use interrupt_service::InterruptService;
use crate::mmu::Mmu;
use crate::MemoryBus;

#[derive(Clone, Debug)]
pub struct Cpu {
    // Registers
    pub (crate) regs: Registers,

    // Interrupt Service
    pub (crate) int_svc: InterruptService,

    next_pc: u16,

    // Memory Management Unit
    pub mmu: *mut Mmu,

}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            regs: Registers::default(),
            int_svc: InterruptService::default(),
            next_pc: 0,
            mmu: ptr::null_mut(),
        }
    }
}

const IF_ADDR: u16 = 0xFF0F;
const IE_ADDR: u16 = 0xFFFF;

impl MemoryBus for Cpu {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            IF_ADDR => self.int_svc.interrupt_latched_register(),
            IE_ADDR => self.int_svc.interrupt_enabled_register(),
            _ => panic!()
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            IF_ADDR => { self.int_svc.set_interrupt_latched_register(data); },
            IE_ADDR => { self.int_svc.set_interrupt_enabled_register(data); },
            _ => panic!("BAD MAP: CPU {:04x} <= {:04x}", addr, data)
        }
    }
}

impl Cpu {
    pub fn registers(&self) -> Registers { self.regs.clone() }

    pub fn request_interrupt(&mut self, int: Interrupt) {
        self.int_svc.latch_interrupt_flags(int);
    }

    fn jump_absolute(&mut self, target: u16) {
        self.next_pc = target;
    }

    fn jump_absolute_if(&mut self, target: u16, cond: bool) -> u64 {
        if cond {
            self.next_pc = target;
            4
        } else {
            0
        }
    }

    fn jump_relative(&mut self, offset: u8) {
        self.next_pc = self.next_pc.wrapping_add((offset as i8) as u16)
    }

    fn jump_relative_if(&mut self, offset: u8, cond: bool) -> u64 {
        if cond {
            self.next_pc = self.next_pc.wrapping_add((offset as i8) as u16);
            4
        } else {
            0
        }
    }

    unsafe fn subroutine_call(&mut self, target: u16) {
        self.stack_push(self.next_pc);
        self.next_pc = target;
    }

    unsafe fn subroutine_call_if(&mut self, target: u16, cond: bool) -> u64 {
        if cond {
            self.stack_push(self.next_pc);
            self.next_pc = target;
            12
        } else {
            0
        }
    }

    unsafe fn subroutine_return(&mut self) {
        self.next_pc = self.stack_pop();
    }

    unsafe fn subroutine_return_if(&mut self, cond: bool) -> u64 {
        if cond {
            self.next_pc = self.stack_pop();
            12
        } else {
            0
        }
    }

    unsafe fn stack_push(&mut self, data: u16) {
        let [lsb, msb] = data.to_le_bytes();
        let sp = self.regs.sp();

        let sp = sp.wrapping_sub(1);
        (*self.mmu).write(sp, msb);

        let sp = sp.wrapping_sub(1);
        (*self.mmu).write(sp, lsb);

        self.regs.set_sp(sp);
    }

    unsafe fn stack_pop(&mut self) -> u16 {
        let sp = self.regs.sp();

        let lsb = (*self.mmu).read(sp);
        let sp = sp.wrapping_add(1);

        let msb = (*self.mmu).read(sp);
        let sp = sp.wrapping_add(1);

        self.regs.set_sp(sp);
        u16::from_le_bytes([lsb, msb])
    }

    pub fn cycle(&mut self) -> u64 {
        self.int_svc.interrupt_service_preamble();

        let mut ticks: u64 = unsafe {
            let pc = self.regs.pc();
            let opcode = (*self.mmu).read(pc);

            let pc = pc.wrapping_add(1);
            let imm8 = (*self.mmu).read(pc);

            let pc = pc.wrapping_add(1);
            let imm16 = u16::from_le_bytes([imm8, (*self.mmu).read(pc)]);

            //trace!("${:04x} {:<15} {:02x?}", pc, asm::disassemble(opcode, imm8, imm16), self.regs);

            self.fetch_decode_execute_store_cycle(opcode, imm8, imm16)
        };

        // HALT Handler
        let halted = self.regs.pc() == self.next_pc;
        if halted && !self.int_svc.interrupt_latched_flags().is_empty() {
            self.next_pc = self.next_pc.wrapping_add(if self.int_svc.enabled() { 1 } else { 2 });
        }

        // Execute Interruptions
        match self.int_svc.interrupt_service_routine() {
            Some(addr) => unsafe {
                self.subroutine_call(addr);
                ticks += 12;
            }
            None => { }
        }
        self.regs.set_pc(self.next_pc);

        ticks
    }

    unsafe fn fetch_decode_execute_store_cycle(&mut self, opcode: u8, imm8: u8, imm16: u16) -> u64 {
        self.next_pc = self.regs.pc() + asm::instruction_size(opcode);
        let mut ticks = asm::instruction_ticks(opcode);

        // Decode => Execute => Store
        match opcode {
            0x00 => {
                // NOP
            }
            0x01 => {
                // LD BC, $0000
                self.regs.set_bc(imm16);
            }
            0x02 => {
                // LD (BC), A
                (*self.mmu).write(self.regs.bc(), self.regs.a());
            },
            0x03 => {
                // INC BC
                let bc = alu::inc16(self.regs.bc());
                self.regs.set_bc(bc);
            }
            0x04 => {
                // INC B
                let (flags, b) = alu::inc(self.regs.flags(), self.regs.b());
                self.regs.set_flags(flags);
                self.regs.set_b(b);
            }
            0x05 => {
                // DEC B
                let (flags, b) = alu::dec(self.regs.flags(), self.regs.b());
                self.regs.set_flags(flags);
                self.regs.set_b(b);
            }
            0x06 => {
                // LD B, $00
                self.regs.set_b(imm8);
            }
            0x07 => {
                // RLCA
                let (flags, a) = alu::rlc(self.regs.a());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x08 => {
                // LD ($0000),SP
                let [lsb, msb] = self.regs.sp().to_le_bytes();
                (*self.mmu).write(imm16, lsb);
                (*self.mmu).write(imm16.wrapping_add(1), msb);
            }
            0x09 => {
                // ADD HL, BC
                let (flags, hl) = alu::add16(self.regs.flags(), self.regs.hl(), self.regs.bc());
                self.regs.set_flags(flags);
                self.regs.set_hl(hl);
            }
            0x0A => {
                // LD A, (BC)
                let data: u8 = (*self.mmu).read(self.regs.bc());
                self.regs.set_a(data);
            }
            0x0B => {
                // DEC BC
                let bc = alu::dec16(self.regs.bc());
                self.regs.set_bc(bc);
            }
            0x0C => {
                // INC C
                let (flags, c) = alu::inc(self.regs.flags(), self.regs.c());
                self.regs.set_flags(flags);
                self.regs.set_c(c);
            }
            0x0D => {
                // DEC C
                let (flags, c) = alu::dec(self.regs.flags(), self.regs.c());
                self.regs.set_flags(flags);
                self.regs.set_c(c);
            }
            0x0E => {
                // LD C, $00
                self.regs.set_c(imm8);
            }
            0x0F => {
                // RRCA
                let (flags, a) = alu::rrc(self.regs.a());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x10 => {
                // STOP 0
            }
            0x11 => {
                // LD DE, $0000
                self.regs.set_de(imm16);
            }
            0x12 => {
                // LD (DE), A
                (*self.mmu).write(self.regs.de(), self.regs.a());
            }
            0x13 => {
                // INC DE
                let de = alu::inc16(self.regs.de());
                self.regs.set_de(de);
            }
            0x14 => {
                // INC D
                let (flags, d) = alu::inc(self.regs.flags(), self.regs.d());
                self.regs.set_flags(flags);
                self.regs.set_d(d);
            }
            0x15 => {
                // DEC D
                let (flags, d) = alu::dec(self.regs.flags(), self.regs.d());
                self.regs.set_flags(flags);
                self.regs.set_d(d);
            }
            0x16 => {
                // LD D, $00
                self.regs.set_d(imm8);
            }
            0x17 => {
                // RLA
                let (flags, a) = alu::rl(self.regs.flags(), self.regs.a());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x18 => {
                // JR $00
                self.jump_relative(imm8);
            }
            0x19 => {
                // ADD HL, DE
                let (flags, hl) = alu::add16(self.regs.flags(), self.regs.hl(), self.regs.de());
                self.regs.set_flags(flags);
                self.regs.set_hl(hl);
            }
            0x1A => {
                // LD A, (DE)
                let data: u8 = (*self.mmu).read(self.regs.de());
                self.regs.set_a(data);
            }
            0x1B => {
                // DEC DE
                let de = alu::dec16(self.regs.de());
                self.regs.set_de(de);
            }
            0x1C => {
                // INC E
                let (flags, e) = alu::inc(self.regs.flags(), self.regs.e());
                self.regs.set_flags(flags);
                self.regs.set_e(e);
            }
            0x1D => {
                // DEC E
                let (flags, e) = alu::dec(self.regs.flags(), self.regs.e());
                self.regs.set_flags(flags);
                self.regs.set_e(e);
            }
            0x1E => {
                // LD E, $00
                self.regs.set_e(imm8);
            }
            0x1F => {
                // RRA
                let (flags, a) = alu::rr(self.regs.flags(), self.regs.a());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x20 => {
                // JR NZ $00
                ticks += self.jump_relative_if(imm8, !self.regs.flags().zero());
            }
            0x21 => {
                // LD HL, $0000
                self.regs.set_hl(imm16);
            }
            0x22 => {
                // LDI (HL), A
                let hl = self.regs.hl();
                (*self.mmu).write(hl, self.regs.a());
                let hl = hl.wrapping_add(1);
                self.regs.set_hl(hl);
            }
            0x23 => {
                // INC HL
                let hl = alu::inc16(self.regs.hl());
                self.regs.set_hl(hl);
            }
            0x24 => {
                // INC H
                let (flags, h) = alu::inc(self.regs.flags(), self.regs.h());
                self.regs.set_flags(flags);
                self.regs.set_h(h);
            }
            0x25 => {
                // DEC H
                let (flags, h) = alu::dec(self.regs.flags(), self.regs.h());
                self.regs.set_flags(flags);
                self.regs.set_h(h);
            }
            0x26 => {
                // LD H, $00
                self.regs.set_h(imm8);
            }
            0x27 => {
                // DAA
                let (flags, a) = alu::daa(self.regs.flags(), self.regs.a());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x28 => {
                // JR Z $00
                ticks += self.jump_relative_if(imm8, self.regs.flags().zero());
            }
            0x29 => {
                // ADD HL, HL
                let (flags, hl) = alu::add16(self.regs.flags(), self.regs.hl(), self.regs.hl());
                self.regs.set_flags(flags);
                self.regs.set_hl(hl);
            }
            0x2A => {
                // LDI A, (HL)
                let addr = self.regs.hl();
                let data = (*self.mmu).read(addr);
                self.regs.set_a(data);
                self.regs.set_hl(addr.wrapping_add(1));
            }
            0x2B => {
                // DEC HL
                let hl = alu::dec16(self.regs.hl());
                self.regs.set_hl(hl);
            }
            0x2C => {
                // INC L
                let (flags, l) = alu::inc(self.regs.flags(), self.regs.l());
                self.regs.set_flags(flags);
                self.regs.set_l(l);
            }
            0x2D => {
                // DEC L
                let (flags, l) = alu::dec(self.regs.flags(), self.regs.l());
                self.regs.set_flags(flags);
                self.regs.set_l(l);
            }
            0x2E => {
                // LD L, $00
                self.regs.set_l(imm8);
            }
            0x2F => {
                // CPL
                let (flags, a) = alu::complement(self.regs.flags(), self.regs.a());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x30 => {
                // JR NC $00
                ticks += self.jump_relative_if(imm8, !self.regs.flags().carry());
            }
            0x31 => {
                // LD SP, $0000
                self.regs.set_sp(imm16);
            }
            0x32 => {
                // LDD (HL), A
                let hl = self.regs.hl();
                (*self.mmu).write(hl, self.regs.a());
                let hl = hl.wrapping_sub(1);
                self.regs.set_hl(hl);
            }
            0x33 => {
                // INC SP
                let sp = self.regs.sp().wrapping_add(1);
                self.regs.set_sp(sp);
            }
            0x34 => {
                // INC (HL)
                let addr = self.regs.hl();
                let data = (*self.mmu).read(addr);

                let (flags, data) = alu::inc(self.regs.flags(), data);
                self.regs.set_flags(flags);
                (*self.mmu).write(addr, data);
            }
            0x35 => {
                // DEC (HL)
                let addr = self.regs.hl();
                let data = (*self.mmu).read(addr);

                let (flags, data) = alu::dec(self.regs.flags(), data);
                self.regs.set_flags(flags);
                (*self.mmu).write(addr, data);
            }
            0x36 => {
                // LD (HL), $00
                (*self.mmu).write(self.regs.hl(), imm8);
            }
            0x37 => {
                // SCF
                let mut flags = self.regs.flags();
                flags.set_carry();

                self.regs.set_flags(flags);
            }
            0x38 => {
                // JR C $00
                ticks += self.jump_relative_if(imm8, self.regs.flags().carry());
            }
            0x39 => {
                // ADD HL, SP
                let (flags, hl) = alu::add16(self.regs.flags(), self.regs.hl(), self.regs.sp());
                self.regs.set_flags(flags);
                self.regs.set_hl(hl);
            }
            0x3A => {
                // LDD A, (HL)
                let addr = self.regs.hl();
                let data = (*self.mmu).read(addr);
                self.regs.set_a(data);
                self.regs.set_hl(addr.wrapping_sub(1));
            }
            0x3B => {
                // DEC SP
                self.regs.set_sp(self.regs.sp().wrapping_sub(1));
            }
            0x3C => {
                // INC A
                let (flags, a) = alu::inc(self.regs.flags(), self.regs.a());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x3D => {
                // DEC A
                let (flags, a) = alu::dec(self.regs.flags(), self.regs.a());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x3E => {
                // LD A, $00
                self.regs.set_a(imm8);
            }
            0x3F => {
                // CCF
                let mut flags = self.regs.flags();
                flags.reset_carry();

                self.regs.set_flags(flags);
            }
            0x40 => {
                // LD B, B
            }
            0x41 => {
                // LD B, C
                self.regs.set_b(self.regs.c());
            }
            0x42 => {
                // LD B, D
                self.regs.set_b(self.regs.d());
            }
            0x43 => {
                // LD B, E
                self.regs.set_b(self.regs.e());
            }
            0x44 => {
                // LD B, H
                self.regs.set_b(self.regs.h());
            }
            0x45 => {
                // LD B, L
                self.regs.set_b(self.regs.l());
            }
            0x46 => {
                // LD B, (HL)
                let data = (*self.mmu).read(self.regs.hl());
                self.regs.set_b(data);
            }
            0x47 => {
                // LD B, A
                self.regs.set_b(self.regs.a());
            }
            0x48 => {
                // LD C, B
                self.regs.set_c(self.regs.b());
            }
            0x49 => {
                // LD C, C
            }
            0x4A => {
                // LD C, D
                self.regs.set_c(self.regs.d());
            }
            0x4B => {
                // LD C, E
                self.regs.set_c(self.regs.e());
            }
            0x4C => {
                // LD C, H
                self.regs.set_c(self.regs.h());
            }
            0x4D => {
                // LD C, L
                self.regs.set_c(self.regs.l());
            }
            0x4E => {
                // LD C, (HL)
                let data = (*self.mmu).read(self.regs.hl());
                self.regs.set_c(data);
            }
            0x4F => {
                // LD C, A
                self.regs.set_c(self.regs.a());
            }
            0x50 => {
                // LD D, B
                self.regs.set_d(self.regs.b());
            }
            0x51 => {
                // LD D, C
                self.regs.set_d(self.regs.c());
            }
            0x52 => {
                // LD D, D
            }
            0x53 => {
                // LD D, E
                self.regs.set_d(self.regs.e());
            }
            0x54 => {
                // LD D, H
                self.regs.set_d(self.regs.h());
            }
            0x55 => {
                // LD D, L
                self.regs.set_d(self.regs.l());
            }
            0x56 => {
                // LD D, (HL)
                let data = (*self.mmu).read(self.regs.hl());
                self.regs.set_d(data);
            }
            0x57 => {
                // LD D, A
                self.regs.set_d(self.regs.a());
            }
            0x58 => {
                // LD E, B
                self.regs.set_e(self.regs.b());
            }
            0x59 => {
                // LD E, C
                self.regs.set_e(self.regs.c());
            }
            0x5A => {
                // LD E, D
                self.regs.set_e(self.regs.d());
            }
            0x5B => {
                // LD E, E
            }
            0x5C => {
                // LD E, H
                self.regs.set_e(self.regs.h());
            }
            0x5D => {
                // LD E, L
                self.regs.set_e(self.regs.l());
            }
            0x5E => {
                // LD E, (HL)
                let data = (*self.mmu).read(self.regs.hl());
                self.regs.set_e(data);
            }
            0x5F => {
                // LD E, A
                self.regs.set_e(self.regs.a());
            }
            0x60 => {
                // LD H, B
                self.regs.set_h(self.regs.b());
            }
            0x61 => {
                // LD H, C
                self.regs.set_h(self.regs.c());
            }
            0x62 => {
                // LD H, D
                self.regs.set_h(self.regs.d());
            }
            0x63 => {
                // LD H, E
                self.regs.set_h(self.regs.e());
            }
            0x64 => {
                // LD H, H
            }
            0x65 => {
                // LD H, L
                self.regs.set_h(self.regs.l());
            }
            0x66 => {
                // LD H, (HL)
                let data = (*self.mmu).read(self.regs.hl());
                self.regs.set_h(data);
            }
            0x67 => {
                // LD H, A
                self.regs.set_h(self.regs.a());
            }
            0x68 => {
                // LD L, B
                self.regs.set_l(self.regs.b());
            }
            0x69 => {
                // LD L, C
                self.regs.set_l(self.regs.c());
            }
            0x6A => {
                // LD L, D
                self.regs.set_l(self.regs.d());
            }
            0x6B => {
                // LD L, E
                self.regs.set_l(self.regs.e());
            }
            0x6C => {
                // LD L, H
                self.regs.set_l(self.regs.h());
            }
            0x6D => {
                // LD L, L
            }
            0x6E => {
                // LD L, (HL)
                let data = (*self.mmu).read(self.regs.hl());
                self.regs.set_l(data);
            }
            0x6F => {
                // LD L, A
                self.regs.set_l(self.regs.a());
            }
            0x70 => {
                // LD (HL), B
                (*self.mmu).write(self.regs.hl(), self.regs.b());
            }
            0x71 => {
                // LD (HL), C
                (*self.mmu).write(self.regs.hl(), self.regs.c());
            }
            0x72 => {
                // LD (HL), D
                (*self.mmu).write(self.regs.hl(), self.regs.d());
            }
            0x73 => {
                // LD (HL), E
                (*self.mmu).write(self.regs.hl(), self.regs.e());
            }
            0x74 => {
                // LD (HL), H
                (*self.mmu).write(self.regs.hl(), self.regs.h());
            }
            0x75 => {
                // LD (HL), L
                (*self.mmu).write(self.regs.hl(), self.regs.l());
            }
            0x76 => {
                // HALT
                self.next_pc = self.regs.pc();
            }
            0x77 => {
                // LD (HL), A
                (*self.mmu).write(self.regs.hl(), self.regs.a());
            }
            0x78 => {
                // LD A, B
                self.regs.set_a(self.regs.b());
            }
            0x79 => {
                // LD A, C
                self.regs.set_a(self.regs.c());
            }
            0x7A => {
                // LD A, D
                self.regs.set_a(self.regs.d());
            }
            0x7B => {
                // LD A, E
                self.regs.set_a(self.regs.e());
            }
            0x7C => {
                // LD A, H
                self.regs.set_a(self.regs.h());
            }
            0x7D => {
                // LD A, L
                self.regs.set_a(self.regs.l());
            }
            0x7E => {
                // LD A, (HL)
                let data = (*self.mmu).read(self.regs.hl());
                self.regs.set_a(data);
            }
            0x7F => {
                // LD A, A
            }
            0x80 => {
                // ADD A, B
                let (flags, a) = alu::add(self.regs.a(), self.regs.b());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x81 => {
                // ADD A, C
                let (flags, a) = alu::add(self.regs.a(), self.regs.c());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x82 => {
                // ADD A, D
                let (flags, a) = alu::add(self.regs.a(), self.regs.d());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x83 => {
                // ADD A, E
                let (flags, a) = alu::add(self.regs.a(), self.regs.e());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x84 => {
                // ADD A, H
                let (flags, a) = alu::add(self.regs.a(), self.regs.h());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x85 => {
                // ADD A, L
                let (flags, a) = alu::add(self.regs.a(), self.regs.l());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x86 => {
                // ADD A, (HL)
                let data = (*self.mmu).read(self.regs.hl());
                let (flags, a) = alu::add(self.regs.a(), data);
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x87 => {
                // ADD A, A
                let (flags, a) = alu::add(self.regs.a(), self.regs.a());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x88 => {
                // ADC A, B
                let (flags, a) = alu::adc(self.regs.flags(), self.regs.a(), self.regs.b());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x89 => {
                // ADC A, C
                let (flags, a) = alu::adc(self.regs.flags(), self.regs.a(), self.regs.c());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x8A => {
                // ADC A, D
                let (flags, a) = alu::adc(self.regs.flags(), self.regs.a(), self.regs.d());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x8B => {
                // ADC A, E
                let (flags, a) = alu::adc(self.regs.flags(), self.regs.a(), self.regs.e());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x8C => {
                // ADC A, H
                let (flags, a) = alu::adc(self.regs.flags(), self.regs.a(), self.regs.h());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x8D => {
                // ADC A, L
                let (flags, a) = alu::adc(self.regs.flags(), self.regs.a(), self.regs.l());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x8E => {
                // ADC A, (HL)
                let data = (*self.mmu).read(self.regs.hl());
                let (flags, a) = alu::adc(self.regs.flags(), self.regs.a(), data);
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x8F => {
                // ADC A, A
                let (flags, a) = alu::adc(self.regs.flags(), self.regs.a(), self.regs.a());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x90 => {
                // SUB A, B
                let (flags, a) = alu::sub(self.regs.a(), self.regs.b());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x91 => {
                // SUB A, C
                let (flags, a) = alu::sub(self.regs.a(), self.regs.c());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x92 => {
                // SUB A, D
                let (flags, a) = alu::sub(self.regs.a(), self.regs.d());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x93 => {
                // SUB A, E
                let (flags, a) = alu::sub(self.regs.a(), self.regs.e());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x94 => {
                // SUB A, H
                let (flags, a) = alu::sub(self.regs.a(), self.regs.h());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x95 => {
                // SUB A, L
                let (flags, a) = alu::sub(self.regs.a(), self.regs.l());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x96 => {
                // SUB A, (HL)
                let data = (*self.mmu).read(self.regs.hl());
                let (flags, a) = alu::sub(self.regs.a(), data);
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x97 => {
                // SUB A, A
                let (flags, a) = alu::sub(self.regs.a(), self.regs.a());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x98 => {
                // SBC A, B
                let (flags, a) = alu::sbc(self.regs.flags(), self.regs.a(), self.regs.b());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x99 => {
                // SBC A, C
                let (flags, a) = alu::sbc(self.regs.flags(), self.regs.a(), self.regs.c());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x9A => {
                // SBC A, D
                let (flags, a) = alu::sbc(self.regs.flags(), self.regs.a(), self.regs.d());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x9B => {
                // SBC A, E
                let (flags, a) = alu::sbc(self.regs.flags(), self.regs.a(), self.regs.e());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x9C => {
                // SBC A, H
                let (flags, a) = alu::sbc(self.regs.flags(), self.regs.a(), self.regs.h());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x9D => {
                // SBC A, L
                let (flags, a) = alu::sbc(self.regs.flags(), self.regs.a(), self.regs.l());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x9E => {
                // SBC A, (HL)
                let data = (*self.mmu).read(self.regs.hl());
                let (flags, a) = alu::sbc(self.regs.flags(), self.regs.a(), data);
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0x9F => {
                // SBC A, A
                let (flags, a) = alu::sbc(self.regs.flags(), self.regs.a(), self.regs.a());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xA0 => {
                // AND A, B
                let (flags, a) = alu::and(self.regs.a(), self.regs.b());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xA1 => {
                // AND A, C
                let (flags, a) = alu::and(self.regs.a(), self.regs.c());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xA2 => {
                // AND A, D
                let (flags, a) = alu::and(self.regs.a(), self.regs.d());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xA3 => {
                // AND A, E
                let (flags, a) = alu::and(self.regs.a(), self.regs.e());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xA4 => {
                // AND A, H
                let (flags, a) = alu::and(self.regs.a(), self.regs.h());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xA5 => {
                // AND A, L
                let (flags, a) = alu::and(self.regs.a(), self.regs.l());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xA6 => {
                // AND A, (HL)
                let data = (*self.mmu).read(self.regs.hl());
                let (flags, a) = alu::and(self.regs.a(), data);
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xA7 => {
                // AND A, A
                let (flags, a) = alu::and(self.regs.a(), self.regs.a());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xA8 => {
                // XOR A, B
                let (flags, a) = alu::xor(self.regs.a(), self.regs.b());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xA9 => {
                // XOR A, C
                let (flags, a) = alu::xor(self.regs.a(), self.regs.c());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xAA => {
                // XOR A, D
                let (flags, a) = alu::xor(self.regs.a(), self.regs.d());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xAB => {
                // XOR A, E
                let (flags, a) = alu::xor(self.regs.a(), self.regs.e());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xAC => {
                // XOR A, H
                let (flags, a) = alu::xor(self.regs.a(), self.regs.h());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xAD => {
                // XOR A, L
                let (flags, a) = alu::xor(self.regs.a(), self.regs.l());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xAE => {
                // XOR A, (HL)
                let data = (*self.mmu).read(self.regs.hl());
                let (flags, a) = alu::xor(self.regs.a(), data);
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xAF => {
                // XOR A, A
                let (flags, a) = alu::xor(self.regs.a(), self.regs.a());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xB0 => {
                // OR A, B
                let (flags, a) = alu::or(self.regs.a(), self.regs.b());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xB1 => {
                // OR A, C
                let (flags, a) = alu::or(self.regs.a(), self.regs.c());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xB2 => {
                // OR A, D
                let (flags, a) = alu::or(self.regs.a(), self.regs.d());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xB3 => {
                // OR A, E
                let (flags, a) = alu::or(self.regs.a(), self.regs.e());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xB4 => {
                // OR A, H
                let (flags, a) = alu::or(self.regs.a(), self.regs.h());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xB5 => {
                // OR A, L
                let (flags, a) = alu::or(self.regs.a(), self.regs.l());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xB6 => {
                // OR A, (HL)
                let data = (*self.mmu).read(self.regs.hl());
                let (flags, a) = alu::or(self.regs.a(), data);
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xB7 => {
                // OR A, A
                let (flags, a) = alu::or(self.regs.a(), self.regs.a());
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xB8 => {
                // CP A, B
                let (flags, _) = alu::sub(self.regs.a(), self.regs.b());
                self.regs.set_flags(flags);
            }
            0xB9 => {
                // CP A, C
                let (flags, _) = alu::sub(self.regs.a(), self.regs.c());
                self.regs.set_flags(flags);
            }
            0xBA => {
                // CP A, D
                let (flags, _) = alu::sub(self.regs.a(), self.regs.d());
                self.regs.set_flags(flags);
            }
            0xBB => {
                // CP A, E
                let (flags, _) = alu::sub(self.regs.a(), self.regs.e());
                self.regs.set_flags(flags);
            }
            0xBC => {
                // CP A, H
                let (flags, _) = alu::sub(self.regs.a(), self.regs.h());
                self.regs.set_flags(flags);
            }
            0xBD => {
                // CP A, L
                let (flags, _) = alu::sub(self.regs.a(), self.regs.l());
                self.regs.set_flags(flags);
            }
            0xBE => {
                // CP A, (HL)
                let data = (*self.mmu).read(self.regs.hl());
                let (flags, _) = alu::sub(self.regs.a(), data);
                self.regs.set_flags(flags);
            }
            0xBF => {
                // CP A, A
                let (flags, _) = alu::sub(self.regs.a(), self.regs.a());
                self.regs.set_flags(flags);
            }
            0xC0 => {
                // RET NZ
                ticks += self.subroutine_return_if(!self.regs.flags().zero());
            }
            0xC1 => {
                // POP BC
                let bc = self.stack_pop();
                self.regs.set_bc(bc);
            }
            0xC2 => {
                // JP NZ $0000
                ticks += self.jump_absolute_if(imm16, !self.regs.flags().zero());
            }
            0xC3 => {
                // JP $0000
                self.jump_absolute(imm16);
            }
            0xC4 => {
                // CALL NZ $0000
                ticks += self.subroutine_call_if(imm16, !self.regs.flags().zero());
            }
            0xC5 => {
                // PUSH BC
                self.stack_push(self.regs.bc());
            }
            0xC6 => {
                // ADD A, $00
                let (flags, a) = alu::add(self.regs.a(), imm8);
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xC7 => {
                // RST $00
                self.subroutine_call(0x00);
            }
            0xC8 => {
                // RET Z
                ticks += self.subroutine_return_if(self.regs.flags().zero());
            }
            0xC9 => {
                // RET
                self.subroutine_return();
            }
            0xCA => {
                // JP Z $0000
                ticks += self.jump_absolute_if(imm16, self.regs.flags().zero());
            }
            0xCB => {
                // PREFIX CB (Logic Instruction Extension)

                let arg = match imm8 & 0x7 {
                    0x0 => self.regs.b(),
                    0x1 => self.regs.c(),
                    0x2 => self.regs.d(),
                    0x3 => self.regs.e(),
                    0x4 => self.regs.h(),
                    0x5 => self.regs.l(),
                    0x6 => (*self.mmu).read(self.regs.hl()),
                    0x7 => self.regs.a(),
                    _ => panic!()
                };

                let (flags, ret) = match imm8 {
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
                        alu::rl(self.regs.flags(), arg)
                    }
                    0x18..=0x1F => {
                        // RR
                        alu::rr(self.regs.flags(), arg)
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
                        let bit_index = (imm8 - 0x40) / 8;
                        let flags = alu::test_bit(self.regs.flags(), arg, bit_index);
                        (flags, arg)
                    }
                    0x80..=0xBF => {
                        // RES
                        let bit_index = (imm8 - 0x80) / 8;
                        (self.regs.flags(), alu::reset_bit(arg, bit_index))
                    }
                    0xC0..=0xFF => {
                        // SET
                        let bit_index = (imm8 - 0xC0) / 8;
                        (self.regs.flags(), alu::set_bit(arg, bit_index))
                    }
                };

                self.regs.set_flags(flags);

                if ret != arg {
                    match imm8 & 0b111 {
                        0x0 => self.regs.set_b(ret),
                        0x1 => self.regs.set_c(ret),
                        0x2 => self.regs.set_d(ret),
                        0x3 => self.regs.set_e(ret),
                        0x4 => self.regs.set_h(ret),
                        0x5 => self.regs.set_l(ret),
                        0x6 => (*self.mmu).write(self.regs.hl(), ret),
                        0x7 => self.regs.set_a(ret),
                        _ => panic!()
                    }
                }
            }
            0xCC => {
                // CALL Z $0000
                ticks += self.subroutine_call_if(imm16, self.regs.flags().zero());
            }
            0xCD => {
                // CALL $0000
                self.subroutine_call(imm16);
            }
            0xCE => {
                // ADC A, $00
                let (flags, a) = alu::adc(self.regs.flags(), self.regs.a(), imm8);
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xCF => {
                // RST $08
                self.subroutine_call(0x08);
            }
            0xD0 => {
                // RET NC
                ticks += self.subroutine_return_if(!self.regs.flags().carry());
            }
            0xD1 => {
                // POP DE
                let de = self.stack_pop();
                self.regs.set_de(de);
            }
            0xD2 => {
                // JP NC $0000
                ticks += self.jump_absolute_if(imm16, !self.regs.flags().carry());
            }
            0xD3 => {
                // [D3] - INVALID
            }
            0xD4 => {
                // CALL NC $0000
                ticks += self.subroutine_call_if(imm16, !self.regs.flags().carry());
            }
            0xD5 => {
                // PUSH DE
                self.stack_push(self.regs.de());
            }
            0xD6 => {
                // SUB A, $00
                let (flags, a) = alu::sub(self.regs.a(), imm8);
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xD7 => {
                // RST $10
                self.subroutine_call(0x10);
            }
            0xD8 => {
                // RET C
                ticks += self.subroutine_return_if(self.regs.flags().carry());
            }
            0xD9 => {
                // RETI
                self.int_svc.enable_interrupt();
                self.subroutine_return();
            }
            0xDA => {
                // JP C $0000
                ticks += self.jump_absolute_if(imm16, self.regs.flags().carry());
            }
            0xDB => {
                // [DB] - INVALID
            }
            0xDC => {
                // CALL C $0000
                ticks += self.subroutine_call_if(imm16, self.regs.flags().carry())
            }
            0xDD => {
                // [DD] - INVALID
            }
            0xDE => {
                // SBC A, $00
                let (flags, a) = alu::sbc(self.regs.flags(), self.regs.a(), imm8);
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xDF => {
                // RST $18
                self.subroutine_call(0x18);
            }
            0xE0 => {
                // LDH ($00), A
                let addr: u16 = 0xff00u16 | imm8 as u16;
                let data = self.regs.a();
                (*self.mmu).write(addr, data);
            }
            0xE1 => {
                // POP HL
                let hl = self.stack_pop();
                self.regs.set_hl(hl);
            }
            0xE2 => {
                // LDH (C), A
                let addr = 0xff00u16 | self.regs.c() as u16;
                let data = self.regs.a();
                (*self.mmu).write(addr, data);
            }
            0xE3 => {
                // [E3] - INVALID
            }
            0xE4 => {
                // [E4] - INVALID
            }
            0xE5 => {
                // PUSH HL
                self.stack_push(self.regs.hl());
            }
            0xE6 => {
                // AND $00
                let (flags, a) = alu::and(self.regs.a(), imm8);
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xE7 => {
                // RST $20
                self.subroutine_call(0x20);
            }
            0xE8 => {
                // ADD SP, $00
                let (flags, sp) = alu::add16_with_s8(self.regs.sp(), imm8);
                self.regs.set_flags(flags);
                self.regs.set_sp(sp);
            }
            0xE9 => {
                // JP HL
                self.jump_absolute(self.regs.hl());
            }
            0xEA => {
                // LD ($0000), A
                (*self.mmu).write(imm16, self.regs.a());
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
                let (flags, a) = alu::xor(self.regs.a(), imm8);
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xEF => {
                // RST $28
                self.subroutine_call(0x28);
            }
            0xF0 => {
                // LDH A, ($00)
                let addr: u16 = 0xff00u16 | imm8 as u16;
                let data = (*self.mmu).read(addr);
                self.regs.set_a(data);
            }
            0xF1 => {
                // POP AF
                let af = self.stack_pop();
                self.regs.set_af(af);
            }
            0xF2 => {
                // LD A, ($FF00+C)
                let addr = 0xff00u16 | self.regs.c() as u16;
                let data = (*self.mmu).read(addr);
                self.regs.set_a(data);
            }
            0xF3 => {
                // DI
                self.int_svc.disable_interrupt();
            }
            0xF4 => {
                // [F4] - INVALID
            }
            0xF5 => {
                // PUSH AF
                self.stack_push(self.regs.af());
            }
            0xF6 => {
                // OR $00
                let (flags, a) = alu::or(self.regs.a(), imm8);
                self.regs.set_flags(flags);
                self.regs.set_a(a);
            }
            0xF7 => {
                // RST $30
                self.subroutine_call(0x30);
            }
            0xF8 => {
                // LD HL,SP+$00
                let (flags, hl) = alu::add16_with_s8(self.regs.sp(), imm8);
                self.regs.set_flags(flags);
                self.regs.set_hl(hl);
            }
            0xF9 => {
                // LD SP, HL
                self.regs.set_sp(self.regs.hl());
            }
            0xFA => {
                // LD A, ($0000)
                let data = (*self.mmu).read(imm16);
                self.regs.set_a(data);
            }
            0xFB => {
                // EI
                self.int_svc.enable_interrupt();
            }
            0xFC => {
                // [FC] - INVALID;
            }
            0xFD => {
                // [FD] - INVALID;
            }
            0xFE => {
                // CP $00
                let (flags, _) = alu::sub(self.regs.a(), imm8);
                self.regs.set_flags(flags);
            }
            0xFF => {
                // RST $38
                self.subroutine_call(0x38);
            }
        }

        ticks
    }
}
