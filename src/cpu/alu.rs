use super::flags::Flags;

pub fn add16_with_s8(acc: u16, arg: u8) -> (Flags, u16) {
    let arg: u16 = (arg as i8) as u16;

    let (_, half) = acc.wrapping_shl(4).overflowing_add(arg.wrapping_shl(4));
    let (acc, carry) = acc.overflowing_add(arg);

    let mut flags = Flags::empty();
    flags.set_half_if(half);
    flags.set_carry_if(carry);

    (flags, acc)
}

/// Add arg to acc
///
/// Flags Affected:
/// - Z: Not Affected
/// - N: Reset
/// - H: Set if carry from bit 11
/// - C: Set if carry from bit 15
pub fn add16(mut flags: Flags, acc: u16, arg: u16) -> (Flags, u16) {
    let (_, half) = acc.wrapping_shl(4).overflowing_add(arg.wrapping_shl(4));
    let (acc, carry) = acc.overflowing_add(arg);

    flags.reset_sub();
    flags.set_half_if(half);
    flags.set_carry_if(carry);

    (flags, acc)
}

/// Increment acc by one
///
/// Flags Affected:
/// - Z: Not affected
/// - N: Not affected
/// - H: Not affected
/// - C: Not affected
pub fn inc16(acc: u16) -> u16 {
    acc.wrapping_add(1)
}

/// Increment acc by one
///
/// Flags Affected:
/// - Z: Not affected
/// - N: Not affected
/// - H: Not affected
/// - C: Not affected
pub fn dec16(acc: u16) -> u16 {
    acc.wrapping_sub(1)
}

/// Add arg to acc
///
/// Flags Affected:
/// - Z: Set if result is zero
/// - N: Reset
/// - H: Set if carry from bit 3
/// - C: Set if carry from bit 7
pub fn add(acc: u8, arg: u8) -> (Flags, u8) {
    let (_, half) = acc.wrapping_shl(4).overflowing_add(arg.wrapping_shl(4));
    let (acc, carry) = acc.overflowing_add(arg);

    let mut flags = Flags::empty();
    flags.set_zero_if(acc == 0);
    flags.set_half_if(half);
    flags.set_carry_if(carry);

    (flags, acc)
}

/// Subtract arg from acc
///
/// Flags Affected:
/// - Z: Set if result is zero
/// - N: Set
/// - H: Set if borrow from bit 4
/// - C: Set if no borrow
pub fn sub(acc: u8, arg: u8) -> (Flags, u8) {
    let (_, half) = acc.wrapping_shr(4).overflowing_sub(arg.wrapping_shr(4));
    let (acc, carry) = acc.overflowing_sub(arg);

    let mut flags = Flags::N;
    flags.set_zero_if(acc == 0);
    flags.set_half_if(half);
    flags.set_carry_if(carry);

    (flags, acc)
}

/// Add (arg + carry) to acc
///
/// Flags Affected:
/// - Z: Set if result is zero
/// - N: Reset
/// - H: Set if carry from bit 3
/// - C: Set if carry from bit 7
pub fn adc(mut flags: Flags, acc: u8, arg: u8) -> (Flags, u8) {
    if !flags.carry() {
        return add(acc, arg)
    }

    let (aux, half1) = arg.wrapping_shl(4).overflowing_add(0x10);
    let (_, half2) = acc.wrapping_shl(4).overflowing_add(aux);

    let (aux, carry1) = arg.overflowing_add(1);
    let (acc, carry2) = acc.overflowing_add(aux);

    flags.set_zero_if(acc == 0);
    flags.reset_sub();
    flags.set_half_if(half1 || half2);
    flags.set_carry_if(carry1 || carry2);

    (flags, acc)
}

/// Subtract (arg + carry) from acc
///
/// Flags Affected:
/// - Z: Set if result is zero
/// - N: Set
/// - H: Set if borrow from bit 4
/// - C: Set if no borrow
pub fn sbc(mut flags: Flags, acc: u8, arg: u8) -> (Flags, u8) {
    if !flags.carry() {
        return sub(acc, arg);
    }

    let arg = arg.wrapping_add(1);
    let half = (acc & 0xf0) < (arg & 0xf0);
    let (acc, carry) = acc.overflowing_sub(arg);

    flags.set_zero_if(acc == 0);
    flags.set_sub();
    flags.set_half_if(half);
    flags.set_carry_if(carry);

    (flags, acc)
}

/// Increment acc by one
///
/// Flags Affected:
/// - Z: Set if result is zero
/// - N: Reset
/// - H: Set if carry from bit 3
/// - C: Not affected
pub fn inc(mut flags: Flags, mut acc: u8) -> (Flags, u8) {
    flags.set_zero_if(acc == 0xff);
    flags.reset_sub();
    flags.set_half_if(acc & 0xf == 0xf);

    acc = acc.wrapping_add(1);

    (flags, acc)
}

/// Decrement acc by one
///
/// Flags Affected:
/// - Z: Set if result is zero
/// - N: Set
/// - H: Set if no borrow from bit 4
/// - C: Not affected
pub fn dec(mut flags: Flags, mut acc: u8) -> (Flags, u8) {
    flags.set_zero_if(acc == 1);
    flags.set_sub();
    flags.set_half_if(acc & 0xf == 0);

    acc = acc.wrapping_sub(1);

    (flags, acc)
}

/// Logical AND of acc with arg
///
/// Flags Affected:
/// - Z: Set if result is zero
/// - N: Reset
/// - H: Set
/// - C: Reset
pub fn and( mut acc: u8, arg: u8) -> (Flags, u8) {
    acc = acc & arg;

    let mut flags = Flags::H;
    flags.set_zero_if(acc == 0);

    (flags, acc)
}

/// Logical OR of acc with arg
///
/// Flags Affected:
/// - Z: Set if result is zero
/// - N: Reset
/// - H: Reset
/// - C: Reset
pub fn or(mut acc: u8, arg: u8) -> (Flags, u8) {
    acc = acc | arg;

    let flags = if acc == 0 { Flags::Z } else { Flags::empty() };

    (flags, acc)
}

/// Logical eXclusive OR (XOR) of acc with arg
///
/// Flags Affected:
/// - Z: Set if result is zero
/// - N: Reset
/// - H: Reset
/// - C: Reset
pub fn xor(mut acc: u8, arg: u8) -> (Flags, u8) {
    acc = acc ^ arg;

    let flags = if acc == 0 { Flags::Z } else { Flags::empty() };

    (flags, acc)
}

/// Rotates acc to the left with bit 7 being moved to bit 0 and also stored into the carry.
///
/// Flags Affected:
/// - Z - Set if result is zero
/// - N - Reset
/// - H - Reset
/// - C - Contains old bit 7
pub fn rlc(mut acc: u8) -> (Flags, u8) {
    acc = acc.rotate_left(1);

    let mut flags = Flags::empty();
    flags.set_zero_if(acc == 0);
    flags.set_carry_if(acc & 0x01 != 0);

    (flags, acc)
}

/// Rotates acc to the right with bit 0 moved to bit 7 and also stored into the carry.
///
/// Flags Affected:
/// - Z: Set if result is zero
/// - N: Reset
/// - H: Reset
/// - C: Contains old bit 0
pub fn rrc(mut acc: u8) -> (Flags, u8) {
    acc = acc.rotate_right(1);

    let mut flags= Flags::empty();
    flags.set_zero_if(acc == 0);
    flags.set_carry_if(acc & 0x80 != 0);

    (flags, acc)
}

/// Rotates acc to the left with the carry's value put into bit 0 and bit 7 is put into the carry.
///
/// Flags Affected:
/// - Z: Set if result is zero
/// - N: Reset
/// - H: Reset
/// - C: Contains old bit 7
pub fn rl(mut flags: Flags, mut acc: u8) -> (Flags, u8) {
    let carry = acc & 0x80 != 0;
    acc = acc.rotate_left(1) & !(1 << 0) | if flags.carry() { 1 << 0 } else { 0 };

    flags.set_zero_if(acc == 0);
    flags.reset_sub();
    flags.reset_half();
    flags.set_carry_if(carry);

    (flags, acc)
}

/// Rotates acc to the right with the carry put in bit 7 and bit 0 put into the carry.
/// 
/// Flags Affected:
/// - Z: Set if result is zero
/// - N: Reset
/// - H: Reset
/// - C: Contains old bit 0
pub fn rr(mut flags: Flags, mut acc: u8) -> (Flags, u8) {
    let carry = acc & 0x01 != 0;
    acc = acc.rotate_right(1) & !(1 << 7) |  if flags.carry() { 1 << 7 } else { 0 };

    flags.set_zero_if(acc == 0);
    flags.reset_sub();
    flags.reset_half();
    flags.set_carry_if(carry);

    (flags, acc)
}

/// Shift acc to the left with bit 0 set to 0 and bit 7 into the carry.
///
/// Flags Affected:
/// - Z: Set if result is zero
/// - N: Reset
/// - H: Reset
/// - C: Contains old bit 7
pub fn sla(mut acc: u8) -> (Flags, u8) {
    let carry = acc & 1 << 7 != 0;
    acc = acc.wrapping_shl(1);

    let mut flags= Flags::empty();
    flags.set_zero_if(acc == 0);
    flags.set_carry_if(carry);

    (flags, acc)
}

/// Shift acc to the right without changing bit 7 and put bit 0 into the carry.
///
/// Flags Affected:
/// - Z: Set if result is zero
/// - N: Reset
/// - H: Reset
/// - C: Contains old bit 0
pub fn sra(mut acc: u8) -> (Flags, u8) {
    let carry = acc & 1 << 0 != 0;
    acc = (acc & 0x80) | acc.wrapping_shr(1);

    let mut flags= Flags::empty();
    flags.set_zero_if(acc == 0);
    flags.set_carry_if(carry);

    (flags, acc)
}

/// Shift acc to the right with 0 put in bit 7 and put bit 0 into the carry.
///
/// Flags Affected:
/// - Z: Set if result is zero
/// - N: Reset
/// - H: Reset
/// - C: Contains old bit 0
pub fn srl( mut acc: u8) -> (Flags, u8) {
    let carry = acc & 0x01 != 0;
    acc = acc.wrapping_shr(1);

    let mut flags= Flags::empty();
    flags.set_zero_if(acc == 0);
    flags.set_carry_if(carry);

    (flags, acc)
}

/// Swap upper and lower nibbles of acc
///
/// Flags Affected:
/// - Z: Set if result is zero
/// - N: Reset
/// - H: Reset
/// - C: Reset
pub fn nibble_swap(mut acc: u8) -> (Flags, u8) {
    acc = acc.wrapping_shl(4) | acc.wrapping_shr(4);

    let mut flags= Flags::empty();
    flags.set_zero_if(acc == 0);

    (flags, acc)
}

/// Complement acc [Flip all bits]
///
/// Flags Affected:
/// - Z: Not affected
/// - N: Set
/// - H: Set
/// - C: Not affected
pub fn complement(mut flags: Flags, mut acc: u8) -> (Flags, u8) {
    acc = !acc;

    flags.set_sub();
    flags.set_half();

    (flags, acc)
}

/// Test bit
///
/// Flags Affected:
/// - Z: Set if bit b of register r is 0
/// - N: Reset
/// - H: Set
/// - C: Not affected
pub fn test_bit(mut flags: Flags, acc: u8, bit_index: u8) -> Flags {
    assert!(bit_index < 8);

    flags.set_zero_if(acc & (1 << bit_index) == 0);
    flags.reset_sub();
    flags.set_half();

    flags
}

/// Set bit to 1
///
/// Flags Affected: NONE
pub fn set_bit(acc: u8, bit_index: u8) -> u8 {
    assert!(bit_index < 8);
    acc | (1 << bit_index)
}

/// Reset bit to 0
///
/// Flags Affected: NONE
pub fn reset_bit(acc: u8, bit_index: u8) -> u8 {
    assert!(bit_index < 8);
    acc & !(1 << bit_index)
}

/// Decimal Adjust acc to obtain the bcd representation
///
/// - Z: Set if register acc is zero. 
/// - N: Not affected.
/// - H: Reset.
/// - C: Set or reset according to operation.
pub fn daa(mut flags: Flags, mut acc: u8) -> (Flags, u8) {
    if !flags.sub() {
        // after an addition, adjust if (half-)carry occurred or if result is out of bounds
        if flags.carry() || acc > 0x99 {
            acc = acc.wrapping_add(0x60);
            flags.set_carry();
        }
        if flags.half() || (acc & 0x0f) > 0x09 {
            acc = acc.wrapping_add(0x06);
        }
    }
    else {
        // after a subtraction, only adjust if (half-)carry occurred
        if flags.carry() {
            acc = acc.wrapping_sub(0x60);
        }
        if flags.half() {
            acc = acc.wrapping_sub(0x06);
        }
    }
    flags.set_zero_if(acc == 0);
    flags.reset_half();

    (flags, acc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_exaustive_test() {
        for acc in 0u8..255u8 {
            for arg in 0u8..255u8 {
                let half = ((acc & 0xf) + (arg & 0xf)) > 0xf;
                let (expected, carry) = acc.overflowing_add(arg);

                let (flags, acc) = add(acc, arg);
                assert_eq!(expected, acc);

                assert_eq!(acc == 0, flags.zero());
                assert_eq!(false, flags.sub());
                assert_eq!(half, flags.half());
                assert_eq!(carry, flags.carry());
            }
        }
    }

    #[test]
    fn adc_exaustive_test() {
        for flags in 0u8..15u8 {
            for acc in 0u8..255u8 {
                for arg in 0u8..255u8 {
                    let flags = Flags::from(flags << 4);

                    let c = if flags.carry() { 1 } else { 0 };
                    let half = ((acc & 0xf) + (arg & 0xf) + c) > 0xf;

                    let expected = acc.wrapping_add(arg).wrapping_add(c);
                    let carry = expected < acc;

                    let (flags, acc) = adc(flags, acc, arg);
                    assert_eq!(expected, acc);

                    assert_eq!(acc == 0, flags.zero());
                    assert_eq!(false, flags.sub());
                    assert_eq!(half, flags.half());
                    assert_eq!(carry, flags.carry());
                }
            }
        }
    }

    #[test]
    fn sub_exaustive_test() {
        for acc in 0u8..255u8 {
            for arg in 0u8..255u8 {
                let half = (acc & 0xf0) < (arg & 0xf0);
                let (expected, carry) = acc.overflowing_sub(arg);

                let (flags, acc) = sub(acc, arg);

                assert_eq!(expected, acc);

                assert_eq!(acc == 0, flags.zero());
                assert_eq!(true, flags.sub());
                assert_eq!(half, flags.half());
                assert_eq!(carry, flags.carry());
            }
        }
    }

    #[test]
    fn sbc_exaustive_test() {
        for flags in 0u8..15u8 {
            for acc in 0u8..255u8 {
                for arg in 0u8..255u8 {
                    let flags = Flags::from(flags << 4);

                    let c : u8= if flags.carry() { 1 } else { 0 };
                    let half = (acc & 0xf0) < (arg.wrapping_add(c) & 0xf0);

                    let expected = acc.wrapping_sub(arg).wrapping_sub(c);
                    let carry = expected > acc;

                    let (flags, acc) = sbc(flags, acc, arg);

                    assert_eq!(expected, acc);

                    assert_eq!(acc == 0, flags.zero());
                    assert_eq!(true, flags.sub());
                    assert_eq!(half, flags.half());
                    assert_eq!(carry, flags.carry());
                }
            }
        }
    }

    #[test]
    fn inc_exaustive_test() {
        for flags in 0u8..15u8 {
            for acc in 0u8..255u8 {
                let flags = Flags::from(flags << 4);
                let carry = flags.carry();
                let half = (acc & 0xf) == 0xf;
                let expected = acc.wrapping_add(1);

                let (flags, acc) = inc(flags, acc);

                assert_eq!(expected, acc);
                assert_eq!(acc == 0, flags.zero());
                assert_eq!(false, flags.sub());
                assert_eq!(half, flags.half());
                assert_eq!(carry, flags.carry());
            }
        }
    }

    #[test]
    fn dec_exaustive_test() {
        for flags in 0u8..15u8 {
            for acc in 0u8..255u8 {
                let flags = Flags::from(flags << 4);
                let carry = flags.carry();
                let half = (acc & 0xf) == 0;
                let expected = acc.wrapping_sub(1);

                let (flags, acc) = dec(flags, acc);

                assert_eq!(expected, acc);
                assert_eq!(acc == 0, flags.zero());
                assert_eq!(true, flags.sub());
                assert_eq!(half, flags.half());
                assert_eq!(carry, flags.carry());
            }
        }
    }

    #[test]
    fn and_exaustive_test() {
        for acc in 0u8..255u8 {
            for arg in 0u8..255u8 {
                let expected = acc & arg;

                let (flags, acc) = and(acc, arg);

                assert_eq!(expected, acc);
                assert_eq!(acc == 0, flags.zero());
                assert_eq!(false, flags.sub());
                assert_eq!(true, flags.half());
                assert_eq!(false, flags.carry());
            }
        }
    }

    #[test]
    fn or_exaustive_test() {
        for acc in 0u8..255u8 {
            for arg in 0u8..255u8 {
                let expected = acc | arg;
                let (flags, acc) = or(acc, arg);

                assert_eq!(expected, acc);
                assert_eq!(acc == 0, flags.zero());
                assert_eq!(false, flags.sub());
                assert_eq!(false, flags.half());
                assert_eq!(false, flags.carry());
            }
        }
    }

    #[test]
    fn xor_exaustive_test() {
        for acc in 0u8..255u8 {
            for arg in 0u8..255u8 {
                let expected = acc ^ arg;
                let (flags, acc) = xor(acc, arg);

                assert_eq!(expected, acc);
                assert_eq!(acc == 0, flags.zero());
                assert_eq!(false, flags.sub());
                assert_eq!(false, flags.half());
                assert_eq!(false, flags.carry());
            }
        }
    }

    #[test]
    fn rlc_exaustive_test() {
        for acc in 0u8..255u8 {
            let carry = (acc & 0x80) != 0;
            let expected = acc.rotate_left(1);

            let (flags, acc) = rlc(acc);

            assert_eq!(expected, acc);
            assert_eq!(acc == 0, flags.zero());
            assert_eq!(false, flags.sub());
            assert_eq!(false, flags.half());
            assert_eq!(carry, flags.carry());
        }
    }

    #[test]
    fn rrc_exaustive_test() {
        for acc in 0u8..255u8 {
            let carry = (acc & 0x01) != 0;
            let expected = acc.rotate_right(1);

            let (flags, acc) = rrc(acc);

            assert_eq!(expected, acc);
            assert_eq!(acc == 0, flags.zero());
            assert_eq!(false, flags.sub());
            assert_eq!(false, flags.half());
            assert_eq!(carry, flags.carry());
        }
    }

    #[test]
    fn rl_exaustive_test() {
        for flags in 0u8..15u8 {
            for acc in 0u8..255u8 {
                let flags = Flags::from(flags << 4);
                let carry = (acc & 0x80) != 0;
                let expected = acc.wrapping_shl(1) + if flags.carry() { 1 } else { 0 };
                let (flags, acc) = rl(flags, acc);

                assert_eq!(expected, acc);
                assert_eq!(acc == 0, flags.zero());
                assert_eq!(false, flags.sub());
                assert_eq!(false, flags.half());
                assert_eq!(carry, flags.carry());
            }
        }
    }

    #[test]
    fn rr_exaustive_test() {
        for flags in 0u8..15u8 {
            for acc in 0u8..255u8 {
                let flags = Flags::from(flags << 4);
                let carry = (acc & 0x01) != 0;
                let expected = acc.wrapping_shr(1) + if flags.carry() { 1 << 7 } else { 0 };

                let (flags, acc) = rr(flags, acc);

                assert_eq!(expected, acc);
                assert_eq!(acc == 0, flags.zero());
                assert_eq!(false, flags.sub());
                assert_eq!(false, flags.half());
                assert_eq!(carry, flags.carry());
            }
        }
    }

    #[test]
    fn sla_exaustive_test() {
        for acc in 0u8..255u8 {
            let carry = acc & 0x80 != 0;
            let expected = acc.wrapping_shl(1);

            let (flags, acc) = sla(acc);

            assert_eq!(expected, acc);
            assert_eq!(acc == 0, flags.zero());
            assert_eq!(false, flags.sub());
            assert_eq!(false, flags.half());
            assert_eq!(carry, flags.carry());
        }
    }

    #[test]
    fn sra_exaustive_test() {
        for acc in 0u8..255u8 {
            let carry = acc & 0x01 != 0;
            let expected = (acc & 0x80) | acc.wrapping_shr(1);

            let (flags, acc) = sra(acc);

            assert_eq!(expected, acc);
            assert_eq!(acc == 0, flags.zero());
            assert_eq!(false, flags.sub());
            assert_eq!(false, flags.half());
            assert_eq!(carry, flags.carry());
        }
    }

    #[test]
    fn nibble_swap_exaustive_test() {
        for acc in 0u8..255u8 {
            let expected = acc.rotate_left(4);

            let (flags, acc) = nibble_swap(acc);

            assert_eq!(expected, acc);
            assert_eq!(acc == 0, flags.zero());
            assert_eq!(false, flags.sub());
            assert_eq!(false, flags.half());
            assert_eq!(false, flags.carry());
        }
    }

    #[test]
    fn complement_exaustive_test() {
        for flags in 0u8..15u8 {
            for acc in 0u8..255u8 {
                let flags = Flags::from(flags << 4);
                let zero = flags.zero();
                let carry = flags.carry();
                let expected = !acc;

                let (flags, acc) = complement(flags, acc);

                assert_eq!(expected, acc);
                assert_eq!(zero, flags.zero());
                assert_eq!(true, flags.sub());
                assert_eq!(true, flags.half());
                assert_eq!(carry, flags.carry());
            }
        }
    }

    #[test]
    fn test_bit_exaustive_test() {
        for flags in 0u8..15u8 {
            for acc in 0u8..255u8 {
                for bit in 0u8..7u8
                {
                    let flags = Flags::from(flags << 4);
                    let carry = flags.carry();

                    let flags = test_bit(flags, acc, bit);

                    assert_eq!(acc & (1 << bit) == 0, flags.zero());
                    assert_eq!(false, flags.sub());
                    assert_eq!(true, flags.half());
                    assert_eq!(carry, flags.carry());
                }
            }
        }
    }

    #[test]
    fn set_bit_exaustive_test() {
        for acc in 0u8..255u8 {
            for bit in 0u8..7u8
            {
                let ret = set_bit(acc, bit);
                assert_eq!(true, (ret & 1 << bit) != 0);
                assert_eq!(0, (ret & !(1 << bit) ^ (acc & !(1 << bit))));
            }
        }
    }

    #[test]
    fn reset_bit_exaustive_test() {
        for acc in 0u8..255u8 {
            for bit in 0u8..7u8
            {
                let ret = reset_bit(acc, bit);
                assert_eq!(false, (ret & 1 << bit) != 0);
                assert_eq!(0, (ret & !(1 << bit) ^ (acc & !(1 << bit))));
            }
        }
    }

    #[test]
    fn alu_daa_exaustive_test() {
        for sample in DAA_MAP.iter() {
            let (beg_a, beg_f, end_a, end_f) = sample;
            let beg_f = Flags::from(beg_f << 4);
            let end_f = Flags::from(end_f << 4);
    
            let (flags, acc) = daa(beg_f, *beg_a);
            assert_eq!(*end_a, acc);
            assert_eq!(end_f, flags);
        }
    }

    // (A, F, A', F') where A => A' and F => F' after DAA.
    // https://raw.githubusercontent.com/Gekkio/mooneye-gb/master/tests/acceptance/instr/daa.s
    const DAA_MAP: [(u8, u8, u8, u8); 2048] = [
        (0x00, 0b0000, 0x00, 0b1000),
        (0x00, 0b0001, 0x60, 0b0001),
        (0x00, 0b0010, 0x06, 0b0000),
        (0x00, 0b0011, 0x66, 0b0001),
        (0x00, 0b0100, 0x00, 0b1100),
        (0x00, 0b0101, 0xa0, 0b0101),
        (0x00, 0b0110, 0xfa, 0b0100),
        (0x00, 0b0111, 0x9a, 0b0101),
        (0x00, 0b1000, 0x00, 0b1000),
        (0x00, 0b1001, 0x60, 0b0001),
        (0x00, 0b1010, 0x06, 0b0000),
        (0x00, 0b1011, 0x66, 0b0001),
        (0x00, 0b1100, 0x00, 0b1100),
        (0x00, 0b1101, 0xa0, 0b0101),
        (0x00, 0b1110, 0xfa, 0b0100),
        (0x00, 0b1111, 0x9a, 0b0101),
        (0x01, 0b0000, 0x01, 0b0000),
        (0x01, 0b0001, 0x61, 0b0001),
        (0x01, 0b0010, 0x07, 0b0000),
        (0x01, 0b0011, 0x67, 0b0001),
        (0x01, 0b0100, 0x01, 0b0100),
        (0x01, 0b0101, 0xa1, 0b0101),
        (0x01, 0b0110, 0xfb, 0b0100),
        (0x01, 0b0111, 0x9b, 0b0101),
        (0x01, 0b1000, 0x01, 0b0000),
        (0x01, 0b1001, 0x61, 0b0001),
        (0x01, 0b1010, 0x07, 0b0000),
        (0x01, 0b1011, 0x67, 0b0001),
        (0x01, 0b1100, 0x01, 0b0100),
        (0x01, 0b1101, 0xa1, 0b0101),
        (0x01, 0b1110, 0xfb, 0b0100),
        (0x01, 0b1111, 0x9b, 0b0101),
        (0x02, 0b0000, 0x02, 0b0000),
        (0x02, 0b0001, 0x62, 0b0001),
        (0x02, 0b0010, 0x08, 0b0000),
        (0x02, 0b0011, 0x68, 0b0001),
        (0x02, 0b0100, 0x02, 0b0100),
        (0x02, 0b0101, 0xa2, 0b0101),
        (0x02, 0b0110, 0xfc, 0b0100),
        (0x02, 0b0111, 0x9c, 0b0101),
        (0x02, 0b1000, 0x02, 0b0000),
        (0x02, 0b1001, 0x62, 0b0001),
        (0x02, 0b1010, 0x08, 0b0000),
        (0x02, 0b1011, 0x68, 0b0001),
        (0x02, 0b1100, 0x02, 0b0100),
        (0x02, 0b1101, 0xa2, 0b0101),
        (0x02, 0b1110, 0xfc, 0b0100),
        (0x02, 0b1111, 0x9c, 0b0101),
        (0x03, 0b0000, 0x03, 0b0000),
        (0x03, 0b0001, 0x63, 0b0001),
        (0x03, 0b0010, 0x09, 0b0000),
        (0x03, 0b0011, 0x69, 0b0001),
        (0x03, 0b0100, 0x03, 0b0100),
        (0x03, 0b0101, 0xa3, 0b0101),
        (0x03, 0b0110, 0xfd, 0b0100),
        (0x03, 0b0111, 0x9d, 0b0101),
        (0x03, 0b1000, 0x03, 0b0000),
        (0x03, 0b1001, 0x63, 0b0001),
        (0x03, 0b1010, 0x09, 0b0000),
        (0x03, 0b1011, 0x69, 0b0001),
        (0x03, 0b1100, 0x03, 0b0100),
        (0x03, 0b1101, 0xa3, 0b0101),
        (0x03, 0b1110, 0xfd, 0b0100),
        (0x03, 0b1111, 0x9d, 0b0101),
        (0x04, 0b0000, 0x04, 0b0000),
        (0x04, 0b0001, 0x64, 0b0001),
        (0x04, 0b0010, 0x0a, 0b0000),
        (0x04, 0b0011, 0x6a, 0b0001),
        (0x04, 0b0100, 0x04, 0b0100),
        (0x04, 0b0101, 0xa4, 0b0101),
        (0x04, 0b0110, 0xfe, 0b0100),
        (0x04, 0b0111, 0x9e, 0b0101),
        (0x04, 0b1000, 0x04, 0b0000),
        (0x04, 0b1001, 0x64, 0b0001),
        (0x04, 0b1010, 0x0a, 0b0000),
        (0x04, 0b1011, 0x6a, 0b0001),
        (0x04, 0b1100, 0x04, 0b0100),
        (0x04, 0b1101, 0xa4, 0b0101),
        (0x04, 0b1110, 0xfe, 0b0100),
        (0x04, 0b1111, 0x9e, 0b0101),
        (0x05, 0b0000, 0x05, 0b0000),
        (0x05, 0b0001, 0x65, 0b0001),
        (0x05, 0b0010, 0x0b, 0b0000),
        (0x05, 0b0011, 0x6b, 0b0001),
        (0x05, 0b0100, 0x05, 0b0100),
        (0x05, 0b0101, 0xa5, 0b0101),
        (0x05, 0b0110, 0xff, 0b0100),
        (0x05, 0b0111, 0x9f, 0b0101),
        (0x05, 0b1000, 0x05, 0b0000),
        (0x05, 0b1001, 0x65, 0b0001),
        (0x05, 0b1010, 0x0b, 0b0000),
        (0x05, 0b1011, 0x6b, 0b0001),
        (0x05, 0b1100, 0x05, 0b0100),
        (0x05, 0b1101, 0xa5, 0b0101),
        (0x05, 0b1110, 0xff, 0b0100),
        (0x05, 0b1111, 0x9f, 0b0101),
        (0x06, 0b0000, 0x06, 0b0000),
        (0x06, 0b0001, 0x66, 0b0001),
        (0x06, 0b0010, 0x0c, 0b0000),
        (0x06, 0b0011, 0x6c, 0b0001),
        (0x06, 0b0100, 0x06, 0b0100),
        (0x06, 0b0101, 0xa6, 0b0101),
        (0x06, 0b0110, 0x00, 0b1100),
        (0x06, 0b0111, 0xa0, 0b0101),
        (0x06, 0b1000, 0x06, 0b0000),
        (0x06, 0b1001, 0x66, 0b0001),
        (0x06, 0b1010, 0x0c, 0b0000),
        (0x06, 0b1011, 0x6c, 0b0001),
        (0x06, 0b1100, 0x06, 0b0100),
        (0x06, 0b1101, 0xa6, 0b0101),
        (0x06, 0b1110, 0x00, 0b1100),
        (0x06, 0b1111, 0xa0, 0b0101),
        (0x07, 0b0000, 0x07, 0b0000),
        (0x07, 0b0001, 0x67, 0b0001),
        (0x07, 0b0010, 0x0d, 0b0000),
        (0x07, 0b0011, 0x6d, 0b0001),
        (0x07, 0b0100, 0x07, 0b0100),
        (0x07, 0b0101, 0xa7, 0b0101),
        (0x07, 0b0110, 0x01, 0b0100),
        (0x07, 0b0111, 0xa1, 0b0101),
        (0x07, 0b1000, 0x07, 0b0000),
        (0x07, 0b1001, 0x67, 0b0001),
        (0x07, 0b1010, 0x0d, 0b0000),
        (0x07, 0b1011, 0x6d, 0b0001),
        (0x07, 0b1100, 0x07, 0b0100),
        (0x07, 0b1101, 0xa7, 0b0101),
        (0x07, 0b1110, 0x01, 0b0100),
        (0x07, 0b1111, 0xa1, 0b0101),
        (0x08, 0b0000, 0x08, 0b0000),
        (0x08, 0b0001, 0x68, 0b0001),
        (0x08, 0b0010, 0x0e, 0b0000),
        (0x08, 0b0011, 0x6e, 0b0001),
        (0x08, 0b0100, 0x08, 0b0100),
        (0x08, 0b0101, 0xa8, 0b0101),
        (0x08, 0b0110, 0x02, 0b0100),
        (0x08, 0b0111, 0xa2, 0b0101),
        (0x08, 0b1000, 0x08, 0b0000),
        (0x08, 0b1001, 0x68, 0b0001),
        (0x08, 0b1010, 0x0e, 0b0000),
        (0x08, 0b1011, 0x6e, 0b0001),
        (0x08, 0b1100, 0x08, 0b0100),
        (0x08, 0b1101, 0xa8, 0b0101),
        (0x08, 0b1110, 0x02, 0b0100),
        (0x08, 0b1111, 0xa2, 0b0101),
        (0x09, 0b0000, 0x09, 0b0000),
        (0x09, 0b0001, 0x69, 0b0001),
        (0x09, 0b0010, 0x0f, 0b0000),
        (0x09, 0b0011, 0x6f, 0b0001),
        (0x09, 0b0100, 0x09, 0b0100),
        (0x09, 0b0101, 0xa9, 0b0101),
        (0x09, 0b0110, 0x03, 0b0100),
        (0x09, 0b0111, 0xa3, 0b0101),
        (0x09, 0b1000, 0x09, 0b0000),
        (0x09, 0b1001, 0x69, 0b0001),
        (0x09, 0b1010, 0x0f, 0b0000),
        (0x09, 0b1011, 0x6f, 0b0001),
        (0x09, 0b1100, 0x09, 0b0100),
        (0x09, 0b1101, 0xa9, 0b0101),
        (0x09, 0b1110, 0x03, 0b0100),
        (0x09, 0b1111, 0xa3, 0b0101),
        (0x0a, 0b0000, 0x10, 0b0000),
        (0x0a, 0b0001, 0x70, 0b0001),
        (0x0a, 0b0010, 0x10, 0b0000),
        (0x0a, 0b0011, 0x70, 0b0001),
        (0x0a, 0b0100, 0x0a, 0b0100),
        (0x0a, 0b0101, 0xaa, 0b0101),
        (0x0a, 0b0110, 0x04, 0b0100),
        (0x0a, 0b0111, 0xa4, 0b0101),
        (0x0a, 0b1000, 0x10, 0b0000),
        (0x0a, 0b1001, 0x70, 0b0001),
        (0x0a, 0b1010, 0x10, 0b0000),
        (0x0a, 0b1011, 0x70, 0b0001),
        (0x0a, 0b1100, 0x0a, 0b0100),
        (0x0a, 0b1101, 0xaa, 0b0101),
        (0x0a, 0b1110, 0x04, 0b0100),
        (0x0a, 0b1111, 0xa4, 0b0101),
        (0x0b, 0b0000, 0x11, 0b0000),
        (0x0b, 0b0001, 0x71, 0b0001),
        (0x0b, 0b0010, 0x11, 0b0000),
        (0x0b, 0b0011, 0x71, 0b0001),
        (0x0b, 0b0100, 0x0b, 0b0100),
        (0x0b, 0b0101, 0xab, 0b0101),
        (0x0b, 0b0110, 0x05, 0b0100),
        (0x0b, 0b0111, 0xa5, 0b0101),
        (0x0b, 0b1000, 0x11, 0b0000),
        (0x0b, 0b1001, 0x71, 0b0001),
        (0x0b, 0b1010, 0x11, 0b0000),
        (0x0b, 0b1011, 0x71, 0b0001),
        (0x0b, 0b1100, 0x0b, 0b0100),
        (0x0b, 0b1101, 0xab, 0b0101),
        (0x0b, 0b1110, 0x05, 0b0100),
        (0x0b, 0b1111, 0xa5, 0b0101),
        (0x0c, 0b0000, 0x12, 0b0000),
        (0x0c, 0b0001, 0x72, 0b0001),
        (0x0c, 0b0010, 0x12, 0b0000),
        (0x0c, 0b0011, 0x72, 0b0001),
        (0x0c, 0b0100, 0x0c, 0b0100),
        (0x0c, 0b0101, 0xac, 0b0101),
        (0x0c, 0b0110, 0x06, 0b0100),
        (0x0c, 0b0111, 0xa6, 0b0101),
        (0x0c, 0b1000, 0x12, 0b0000),
        (0x0c, 0b1001, 0x72, 0b0001),
        (0x0c, 0b1010, 0x12, 0b0000),
        (0x0c, 0b1011, 0x72, 0b0001),
        (0x0c, 0b1100, 0x0c, 0b0100),
        (0x0c, 0b1101, 0xac, 0b0101),
        (0x0c, 0b1110, 0x06, 0b0100),
        (0x0c, 0b1111, 0xa6, 0b0101),
        (0x0d, 0b0000, 0x13, 0b0000),
        (0x0d, 0b0001, 0x73, 0b0001),
        (0x0d, 0b0010, 0x13, 0b0000),
        (0x0d, 0b0011, 0x73, 0b0001),
        (0x0d, 0b0100, 0x0d, 0b0100),
        (0x0d, 0b0101, 0xad, 0b0101),
        (0x0d, 0b0110, 0x07, 0b0100),
        (0x0d, 0b0111, 0xa7, 0b0101),
        (0x0d, 0b1000, 0x13, 0b0000),
        (0x0d, 0b1001, 0x73, 0b0001),
        (0x0d, 0b1010, 0x13, 0b0000),
        (0x0d, 0b1011, 0x73, 0b0001),
        (0x0d, 0b1100, 0x0d, 0b0100),
        (0x0d, 0b1101, 0xad, 0b0101),
        (0x0d, 0b1110, 0x07, 0b0100),
        (0x0d, 0b1111, 0xa7, 0b0101),
        (0x0e, 0b0000, 0x14, 0b0000),
        (0x0e, 0b0001, 0x74, 0b0001),
        (0x0e, 0b0010, 0x14, 0b0000),
        (0x0e, 0b0011, 0x74, 0b0001),
        (0x0e, 0b0100, 0x0e, 0b0100),
        (0x0e, 0b0101, 0xae, 0b0101),
        (0x0e, 0b0110, 0x08, 0b0100),
        (0x0e, 0b0111, 0xa8, 0b0101),
        (0x0e, 0b1000, 0x14, 0b0000),
        (0x0e, 0b1001, 0x74, 0b0001),
        (0x0e, 0b1010, 0x14, 0b0000),
        (0x0e, 0b1011, 0x74, 0b0001),
        (0x0e, 0b1100, 0x0e, 0b0100),
        (0x0e, 0b1101, 0xae, 0b0101),
        (0x0e, 0b1110, 0x08, 0b0100),
        (0x0e, 0b1111, 0xa8, 0b0101),
        (0x0f, 0b0000, 0x15, 0b0000),
        (0x0f, 0b0001, 0x75, 0b0001),
        (0x0f, 0b0010, 0x15, 0b0000),
        (0x0f, 0b0011, 0x75, 0b0001),
        (0x0f, 0b0100, 0x0f, 0b0100),
        (0x0f, 0b0101, 0xaf, 0b0101),
        (0x0f, 0b0110, 0x09, 0b0100),
        (0x0f, 0b0111, 0xa9, 0b0101),
        (0x0f, 0b1000, 0x15, 0b0000),
        (0x0f, 0b1001, 0x75, 0b0001),
        (0x0f, 0b1010, 0x15, 0b0000),
        (0x0f, 0b1011, 0x75, 0b0001),
        (0x0f, 0b1100, 0x0f, 0b0100),
        (0x0f, 0b1101, 0xaf, 0b0101),
        (0x0f, 0b1110, 0x09, 0b0100),
        (0x0f, 0b1111, 0xa9, 0b0101),
        (0x10, 0b0000, 0x10, 0b0000),
        (0x10, 0b0001, 0x70, 0b0001),
        (0x10, 0b0010, 0x16, 0b0000),
        (0x10, 0b0011, 0x76, 0b0001),
        (0x10, 0b0100, 0x10, 0b0100),
        (0x10, 0b0101, 0xb0, 0b0101),
        (0x10, 0b0110, 0x0a, 0b0100),
        (0x10, 0b0111, 0xaa, 0b0101),
        (0x10, 0b1000, 0x10, 0b0000),
        (0x10, 0b1001, 0x70, 0b0001),
        (0x10, 0b1010, 0x16, 0b0000),
        (0x10, 0b1011, 0x76, 0b0001),
        (0x10, 0b1100, 0x10, 0b0100),
        (0x10, 0b1101, 0xb0, 0b0101),
        (0x10, 0b1110, 0x0a, 0b0100),
        (0x10, 0b1111, 0xaa, 0b0101),
        (0x11, 0b0000, 0x11, 0b0000),
        (0x11, 0b0001, 0x71, 0b0001),
        (0x11, 0b0010, 0x17, 0b0000),
        (0x11, 0b0011, 0x77, 0b0001),
        (0x11, 0b0100, 0x11, 0b0100),
        (0x11, 0b0101, 0xb1, 0b0101),
        (0x11, 0b0110, 0x0b, 0b0100),
        (0x11, 0b0111, 0xab, 0b0101),
        (0x11, 0b1000, 0x11, 0b0000),
        (0x11, 0b1001, 0x71, 0b0001),
        (0x11, 0b1010, 0x17, 0b0000),
        (0x11, 0b1011, 0x77, 0b0001),
        (0x11, 0b1100, 0x11, 0b0100),
        (0x11, 0b1101, 0xb1, 0b0101),
        (0x11, 0b1110, 0x0b, 0b0100),
        (0x11, 0b1111, 0xab, 0b0101),
        (0x12, 0b0000, 0x12, 0b0000),
        (0x12, 0b0001, 0x72, 0b0001),
        (0x12, 0b0010, 0x18, 0b0000),
        (0x12, 0b0011, 0x78, 0b0001),
        (0x12, 0b0100, 0x12, 0b0100),
        (0x12, 0b0101, 0xb2, 0b0101),
        (0x12, 0b0110, 0x0c, 0b0100),
        (0x12, 0b0111, 0xac, 0b0101),
        (0x12, 0b1000, 0x12, 0b0000),
        (0x12, 0b1001, 0x72, 0b0001),
        (0x12, 0b1010, 0x18, 0b0000),
        (0x12, 0b1011, 0x78, 0b0001),
        (0x12, 0b1100, 0x12, 0b0100),
        (0x12, 0b1101, 0xb2, 0b0101),
        (0x12, 0b1110, 0x0c, 0b0100),
        (0x12, 0b1111, 0xac, 0b0101),
        (0x13, 0b0000, 0x13, 0b0000),
        (0x13, 0b0001, 0x73, 0b0001),
        (0x13, 0b0010, 0x19, 0b0000),
        (0x13, 0b0011, 0x79, 0b0001),
        (0x13, 0b0100, 0x13, 0b0100),
        (0x13, 0b0101, 0xb3, 0b0101),
        (0x13, 0b0110, 0x0d, 0b0100),
        (0x13, 0b0111, 0xad, 0b0101),
        (0x13, 0b1000, 0x13, 0b0000),
        (0x13, 0b1001, 0x73, 0b0001),
        (0x13, 0b1010, 0x19, 0b0000),
        (0x13, 0b1011, 0x79, 0b0001),
        (0x13, 0b1100, 0x13, 0b0100),
        (0x13, 0b1101, 0xb3, 0b0101),
        (0x13, 0b1110, 0x0d, 0b0100),
        (0x13, 0b1111, 0xad, 0b0101),
        (0x14, 0b0000, 0x14, 0b0000),
        (0x14, 0b0001, 0x74, 0b0001),
        (0x14, 0b0010, 0x1a, 0b0000),
        (0x14, 0b0011, 0x7a, 0b0001),
        (0x14, 0b0100, 0x14, 0b0100),
        (0x14, 0b0101, 0xb4, 0b0101),
        (0x14, 0b0110, 0x0e, 0b0100),
        (0x14, 0b0111, 0xae, 0b0101),
        (0x14, 0b1000, 0x14, 0b0000),
        (0x14, 0b1001, 0x74, 0b0001),
        (0x14, 0b1010, 0x1a, 0b0000),
        (0x14, 0b1011, 0x7a, 0b0001),
        (0x14, 0b1100, 0x14, 0b0100),
        (0x14, 0b1101, 0xb4, 0b0101),
        (0x14, 0b1110, 0x0e, 0b0100),
        (0x14, 0b1111, 0xae, 0b0101),
        (0x15, 0b0000, 0x15, 0b0000),
        (0x15, 0b0001, 0x75, 0b0001),
        (0x15, 0b0010, 0x1b, 0b0000),
        (0x15, 0b0011, 0x7b, 0b0001),
        (0x15, 0b0100, 0x15, 0b0100),
        (0x15, 0b0101, 0xb5, 0b0101),
        (0x15, 0b0110, 0x0f, 0b0100),
        (0x15, 0b0111, 0xaf, 0b0101),
        (0x15, 0b1000, 0x15, 0b0000),
        (0x15, 0b1001, 0x75, 0b0001),
        (0x15, 0b1010, 0x1b, 0b0000),
        (0x15, 0b1011, 0x7b, 0b0001),
        (0x15, 0b1100, 0x15, 0b0100),
        (0x15, 0b1101, 0xb5, 0b0101),
        (0x15, 0b1110, 0x0f, 0b0100),
        (0x15, 0b1111, 0xaf, 0b0101),
        (0x16, 0b0000, 0x16, 0b0000),
        (0x16, 0b0001, 0x76, 0b0001),
        (0x16, 0b0010, 0x1c, 0b0000),
        (0x16, 0b0011, 0x7c, 0b0001),
        (0x16, 0b0100, 0x16, 0b0100),
        (0x16, 0b0101, 0xb6, 0b0101),
        (0x16, 0b0110, 0x10, 0b0100),
        (0x16, 0b0111, 0xb0, 0b0101),
        (0x16, 0b1000, 0x16, 0b0000),
        (0x16, 0b1001, 0x76, 0b0001),
        (0x16, 0b1010, 0x1c, 0b0000),
        (0x16, 0b1011, 0x7c, 0b0001),
        (0x16, 0b1100, 0x16, 0b0100),
        (0x16, 0b1101, 0xb6, 0b0101),
        (0x16, 0b1110, 0x10, 0b0100),
        (0x16, 0b1111, 0xb0, 0b0101),
        (0x17, 0b0000, 0x17, 0b0000),
        (0x17, 0b0001, 0x77, 0b0001),
        (0x17, 0b0010, 0x1d, 0b0000),
        (0x17, 0b0011, 0x7d, 0b0001),
        (0x17, 0b0100, 0x17, 0b0100),
        (0x17, 0b0101, 0xb7, 0b0101),
        (0x17, 0b0110, 0x11, 0b0100),
        (0x17, 0b0111, 0xb1, 0b0101),
        (0x17, 0b1000, 0x17, 0b0000),
        (0x17, 0b1001, 0x77, 0b0001),
        (0x17, 0b1010, 0x1d, 0b0000),
        (0x17, 0b1011, 0x7d, 0b0001),
        (0x17, 0b1100, 0x17, 0b0100),
        (0x17, 0b1101, 0xb7, 0b0101),
        (0x17, 0b1110, 0x11, 0b0100),
        (0x17, 0b1111, 0xb1, 0b0101),
        (0x18, 0b0000, 0x18, 0b0000),
        (0x18, 0b0001, 0x78, 0b0001),
        (0x18, 0b0010, 0x1e, 0b0000),
        (0x18, 0b0011, 0x7e, 0b0001),
        (0x18, 0b0100, 0x18, 0b0100),
        (0x18, 0b0101, 0xb8, 0b0101),
        (0x18, 0b0110, 0x12, 0b0100),
        (0x18, 0b0111, 0xb2, 0b0101),
        (0x18, 0b1000, 0x18, 0b0000),
        (0x18, 0b1001, 0x78, 0b0001),
        (0x18, 0b1010, 0x1e, 0b0000),
        (0x18, 0b1011, 0x7e, 0b0001),
        (0x18, 0b1100, 0x18, 0b0100),
        (0x18, 0b1101, 0xb8, 0b0101),
        (0x18, 0b1110, 0x12, 0b0100),
        (0x18, 0b1111, 0xb2, 0b0101),
        (0x19, 0b0000, 0x19, 0b0000),
        (0x19, 0b0001, 0x79, 0b0001),
        (0x19, 0b0010, 0x1f, 0b0000),
        (0x19, 0b0011, 0x7f, 0b0001),
        (0x19, 0b0100, 0x19, 0b0100),
        (0x19, 0b0101, 0xb9, 0b0101),
        (0x19, 0b0110, 0x13, 0b0100),
        (0x19, 0b0111, 0xb3, 0b0101),
        (0x19, 0b1000, 0x19, 0b0000),
        (0x19, 0b1001, 0x79, 0b0001),
        (0x19, 0b1010, 0x1f, 0b0000),
        (0x19, 0b1011, 0x7f, 0b0001),
        (0x19, 0b1100, 0x19, 0b0100),
        (0x19, 0b1101, 0xb9, 0b0101),
        (0x19, 0b1110, 0x13, 0b0100),
        (0x19, 0b1111, 0xb3, 0b0101),
        (0x1a, 0b0000, 0x20, 0b0000),
        (0x1a, 0b0001, 0x80, 0b0001),
        (0x1a, 0b0010, 0x20, 0b0000),
        (0x1a, 0b0011, 0x80, 0b0001),
        (0x1a, 0b0100, 0x1a, 0b0100),
        (0x1a, 0b0101, 0xba, 0b0101),
        (0x1a, 0b0110, 0x14, 0b0100),
        (0x1a, 0b0111, 0xb4, 0b0101),
        (0x1a, 0b1000, 0x20, 0b0000),
        (0x1a, 0b1001, 0x80, 0b0001),
        (0x1a, 0b1010, 0x20, 0b0000),
        (0x1a, 0b1011, 0x80, 0b0001),
        (0x1a, 0b1100, 0x1a, 0b0100),
        (0x1a, 0b1101, 0xba, 0b0101),
        (0x1a, 0b1110, 0x14, 0b0100),
        (0x1a, 0b1111, 0xb4, 0b0101),
        (0x1b, 0b0000, 0x21, 0b0000),
        (0x1b, 0b0001, 0x81, 0b0001),
        (0x1b, 0b0010, 0x21, 0b0000),
        (0x1b, 0b0011, 0x81, 0b0001),
        (0x1b, 0b0100, 0x1b, 0b0100),
        (0x1b, 0b0101, 0xbb, 0b0101),
        (0x1b, 0b0110, 0x15, 0b0100),
        (0x1b, 0b0111, 0xb5, 0b0101),
        (0x1b, 0b1000, 0x21, 0b0000),
        (0x1b, 0b1001, 0x81, 0b0001),
        (0x1b, 0b1010, 0x21, 0b0000),
        (0x1b, 0b1011, 0x81, 0b0001),
        (0x1b, 0b1100, 0x1b, 0b0100),
        (0x1b, 0b1101, 0xbb, 0b0101),
        (0x1b, 0b1110, 0x15, 0b0100),
        (0x1b, 0b1111, 0xb5, 0b0101),
        (0x1c, 0b0000, 0x22, 0b0000),
        (0x1c, 0b0001, 0x82, 0b0001),
        (0x1c, 0b0010, 0x22, 0b0000),
        (0x1c, 0b0011, 0x82, 0b0001),
        (0x1c, 0b0100, 0x1c, 0b0100),
        (0x1c, 0b0101, 0xbc, 0b0101),
        (0x1c, 0b0110, 0x16, 0b0100),
        (0x1c, 0b0111, 0xb6, 0b0101),
        (0x1c, 0b1000, 0x22, 0b0000),
        (0x1c, 0b1001, 0x82, 0b0001),
        (0x1c, 0b1010, 0x22, 0b0000),
        (0x1c, 0b1011, 0x82, 0b0001),
        (0x1c, 0b1100, 0x1c, 0b0100),
        (0x1c, 0b1101, 0xbc, 0b0101),
        (0x1c, 0b1110, 0x16, 0b0100),
        (0x1c, 0b1111, 0xb6, 0b0101),
        (0x1d, 0b0000, 0x23, 0b0000),
        (0x1d, 0b0001, 0x83, 0b0001),
        (0x1d, 0b0010, 0x23, 0b0000),
        (0x1d, 0b0011, 0x83, 0b0001),
        (0x1d, 0b0100, 0x1d, 0b0100),
        (0x1d, 0b0101, 0xbd, 0b0101),
        (0x1d, 0b0110, 0x17, 0b0100),
        (0x1d, 0b0111, 0xb7, 0b0101),
        (0x1d, 0b1000, 0x23, 0b0000),
        (0x1d, 0b1001, 0x83, 0b0001),
        (0x1d, 0b1010, 0x23, 0b0000),
        (0x1d, 0b1011, 0x83, 0b0001),
        (0x1d, 0b1100, 0x1d, 0b0100),
        (0x1d, 0b1101, 0xbd, 0b0101),
        (0x1d, 0b1110, 0x17, 0b0100),
        (0x1d, 0b1111, 0xb7, 0b0101),
        (0x1e, 0b0000, 0x24, 0b0000),
        (0x1e, 0b0001, 0x84, 0b0001),
        (0x1e, 0b0010, 0x24, 0b0000),
        (0x1e, 0b0011, 0x84, 0b0001),
        (0x1e, 0b0100, 0x1e, 0b0100),
        (0x1e, 0b0101, 0xbe, 0b0101),
        (0x1e, 0b0110, 0x18, 0b0100),
        (0x1e, 0b0111, 0xb8, 0b0101),
        (0x1e, 0b1000, 0x24, 0b0000),
        (0x1e, 0b1001, 0x84, 0b0001),
        (0x1e, 0b1010, 0x24, 0b0000),
        (0x1e, 0b1011, 0x84, 0b0001),
        (0x1e, 0b1100, 0x1e, 0b0100),
        (0x1e, 0b1101, 0xbe, 0b0101),
        (0x1e, 0b1110, 0x18, 0b0100),
        (0x1e, 0b1111, 0xb8, 0b0101),
        (0x1f, 0b0000, 0x25, 0b0000),
        (0x1f, 0b0001, 0x85, 0b0001),
        (0x1f, 0b0010, 0x25, 0b0000),
        (0x1f, 0b0011, 0x85, 0b0001),
        (0x1f, 0b0100, 0x1f, 0b0100),
        (0x1f, 0b0101, 0xbf, 0b0101),
        (0x1f, 0b0110, 0x19, 0b0100),
        (0x1f, 0b0111, 0xb9, 0b0101),
        (0x1f, 0b1000, 0x25, 0b0000),
        (0x1f, 0b1001, 0x85, 0b0001),
        (0x1f, 0b1010, 0x25, 0b0000),
        (0x1f, 0b1011, 0x85, 0b0001),
        (0x1f, 0b1100, 0x1f, 0b0100),
        (0x1f, 0b1101, 0xbf, 0b0101),
        (0x1f, 0b1110, 0x19, 0b0100),
        (0x1f, 0b1111, 0xb9, 0b0101),
        (0x20, 0b0000, 0x20, 0b0000),
        (0x20, 0b0001, 0x80, 0b0001),
        (0x20, 0b0010, 0x26, 0b0000),
        (0x20, 0b0011, 0x86, 0b0001),
        (0x20, 0b0100, 0x20, 0b0100),
        (0x20, 0b0101, 0xc0, 0b0101),
        (0x20, 0b0110, 0x1a, 0b0100),
        (0x20, 0b0111, 0xba, 0b0101),
        (0x20, 0b1000, 0x20, 0b0000),
        (0x20, 0b1001, 0x80, 0b0001),
        (0x20, 0b1010, 0x26, 0b0000),
        (0x20, 0b1011, 0x86, 0b0001),
        (0x20, 0b1100, 0x20, 0b0100),
        (0x20, 0b1101, 0xc0, 0b0101),
        (0x20, 0b1110, 0x1a, 0b0100),
        (0x20, 0b1111, 0xba, 0b0101),
        (0x21, 0b0000, 0x21, 0b0000),
        (0x21, 0b0001, 0x81, 0b0001),
        (0x21, 0b0010, 0x27, 0b0000),
        (0x21, 0b0011, 0x87, 0b0001),
        (0x21, 0b0100, 0x21, 0b0100),
        (0x21, 0b0101, 0xc1, 0b0101),
        (0x21, 0b0110, 0x1b, 0b0100),
        (0x21, 0b0111, 0xbb, 0b0101),
        (0x21, 0b1000, 0x21, 0b0000),
        (0x21, 0b1001, 0x81, 0b0001),
        (0x21, 0b1010, 0x27, 0b0000),
        (0x21, 0b1011, 0x87, 0b0001),
        (0x21, 0b1100, 0x21, 0b0100),
        (0x21, 0b1101, 0xc1, 0b0101),
        (0x21, 0b1110, 0x1b, 0b0100),
        (0x21, 0b1111, 0xbb, 0b0101),
        (0x22, 0b0000, 0x22, 0b0000),
        (0x22, 0b0001, 0x82, 0b0001),
        (0x22, 0b0010, 0x28, 0b0000),
        (0x22, 0b0011, 0x88, 0b0001),
        (0x22, 0b0100, 0x22, 0b0100),
        (0x22, 0b0101, 0xc2, 0b0101),
        (0x22, 0b0110, 0x1c, 0b0100),
        (0x22, 0b0111, 0xbc, 0b0101),
        (0x22, 0b1000, 0x22, 0b0000),
        (0x22, 0b1001, 0x82, 0b0001),
        (0x22, 0b1010, 0x28, 0b0000),
        (0x22, 0b1011, 0x88, 0b0001),
        (0x22, 0b1100, 0x22, 0b0100),
        (0x22, 0b1101, 0xc2, 0b0101),
        (0x22, 0b1110, 0x1c, 0b0100),
        (0x22, 0b1111, 0xbc, 0b0101),
        (0x23, 0b0000, 0x23, 0b0000),
        (0x23, 0b0001, 0x83, 0b0001),
        (0x23, 0b0010, 0x29, 0b0000),
        (0x23, 0b0011, 0x89, 0b0001),
        (0x23, 0b0100, 0x23, 0b0100),
        (0x23, 0b0101, 0xc3, 0b0101),
        (0x23, 0b0110, 0x1d, 0b0100),
        (0x23, 0b0111, 0xbd, 0b0101),
        (0x23, 0b1000, 0x23, 0b0000),
        (0x23, 0b1001, 0x83, 0b0001),
        (0x23, 0b1010, 0x29, 0b0000),
        (0x23, 0b1011, 0x89, 0b0001),
        (0x23, 0b1100, 0x23, 0b0100),
        (0x23, 0b1101, 0xc3, 0b0101),
        (0x23, 0b1110, 0x1d, 0b0100),
        (0x23, 0b1111, 0xbd, 0b0101),
        (0x24, 0b0000, 0x24, 0b0000),
        (0x24, 0b0001, 0x84, 0b0001),
        (0x24, 0b0010, 0x2a, 0b0000),
        (0x24, 0b0011, 0x8a, 0b0001),
        (0x24, 0b0100, 0x24, 0b0100),
        (0x24, 0b0101, 0xc4, 0b0101),
        (0x24, 0b0110, 0x1e, 0b0100),
        (0x24, 0b0111, 0xbe, 0b0101),
        (0x24, 0b1000, 0x24, 0b0000),
        (0x24, 0b1001, 0x84, 0b0001),
        (0x24, 0b1010, 0x2a, 0b0000),
        (0x24, 0b1011, 0x8a, 0b0001),
        (0x24, 0b1100, 0x24, 0b0100),
        (0x24, 0b1101, 0xc4, 0b0101),
        (0x24, 0b1110, 0x1e, 0b0100),
        (0x24, 0b1111, 0xbe, 0b0101),
        (0x25, 0b0000, 0x25, 0b0000),
        (0x25, 0b0001, 0x85, 0b0001),
        (0x25, 0b0010, 0x2b, 0b0000),
        (0x25, 0b0011, 0x8b, 0b0001),
        (0x25, 0b0100, 0x25, 0b0100),
        (0x25, 0b0101, 0xc5, 0b0101),
        (0x25, 0b0110, 0x1f, 0b0100),
        (0x25, 0b0111, 0xbf, 0b0101),
        (0x25, 0b1000, 0x25, 0b0000),
        (0x25, 0b1001, 0x85, 0b0001),
        (0x25, 0b1010, 0x2b, 0b0000),
        (0x25, 0b1011, 0x8b, 0b0001),
        (0x25, 0b1100, 0x25, 0b0100),
        (0x25, 0b1101, 0xc5, 0b0101),
        (0x25, 0b1110, 0x1f, 0b0100),
        (0x25, 0b1111, 0xbf, 0b0101),
        (0x26, 0b0000, 0x26, 0b0000),
        (0x26, 0b0001, 0x86, 0b0001),
        (0x26, 0b0010, 0x2c, 0b0000),
        (0x26, 0b0011, 0x8c, 0b0001),
        (0x26, 0b0100, 0x26, 0b0100),
        (0x26, 0b0101, 0xc6, 0b0101),
        (0x26, 0b0110, 0x20, 0b0100),
        (0x26, 0b0111, 0xc0, 0b0101),
        (0x26, 0b1000, 0x26, 0b0000),
        (0x26, 0b1001, 0x86, 0b0001),
        (0x26, 0b1010, 0x2c, 0b0000),
        (0x26, 0b1011, 0x8c, 0b0001),
        (0x26, 0b1100, 0x26, 0b0100),
        (0x26, 0b1101, 0xc6, 0b0101),
        (0x26, 0b1110, 0x20, 0b0100),
        (0x26, 0b1111, 0xc0, 0b0101),
        (0x27, 0b0000, 0x27, 0b0000),
        (0x27, 0b0001, 0x87, 0b0001),
        (0x27, 0b0010, 0x2d, 0b0000),
        (0x27, 0b0011, 0x8d, 0b0001),
        (0x27, 0b0100, 0x27, 0b0100),
        (0x27, 0b0101, 0xc7, 0b0101),
        (0x27, 0b0110, 0x21, 0b0100),
        (0x27, 0b0111, 0xc1, 0b0101),
        (0x27, 0b1000, 0x27, 0b0000),
        (0x27, 0b1001, 0x87, 0b0001),
        (0x27, 0b1010, 0x2d, 0b0000),
        (0x27, 0b1011, 0x8d, 0b0001),
        (0x27, 0b1100, 0x27, 0b0100),
        (0x27, 0b1101, 0xc7, 0b0101),
        (0x27, 0b1110, 0x21, 0b0100),
        (0x27, 0b1111, 0xc1, 0b0101),
        (0x28, 0b0000, 0x28, 0b0000),
        (0x28, 0b0001, 0x88, 0b0001),
        (0x28, 0b0010, 0x2e, 0b0000),
        (0x28, 0b0011, 0x8e, 0b0001),
        (0x28, 0b0100, 0x28, 0b0100),
        (0x28, 0b0101, 0xc8, 0b0101),
        (0x28, 0b0110, 0x22, 0b0100),
        (0x28, 0b0111, 0xc2, 0b0101),
        (0x28, 0b1000, 0x28, 0b0000),
        (0x28, 0b1001, 0x88, 0b0001),
        (0x28, 0b1010, 0x2e, 0b0000),
        (0x28, 0b1011, 0x8e, 0b0001),
        (0x28, 0b1100, 0x28, 0b0100),
        (0x28, 0b1101, 0xc8, 0b0101),
        (0x28, 0b1110, 0x22, 0b0100),
        (0x28, 0b1111, 0xc2, 0b0101),
        (0x29, 0b0000, 0x29, 0b0000),
        (0x29, 0b0001, 0x89, 0b0001),
        (0x29, 0b0010, 0x2f, 0b0000),
        (0x29, 0b0011, 0x8f, 0b0001),
        (0x29, 0b0100, 0x29, 0b0100),
        (0x29, 0b0101, 0xc9, 0b0101),
        (0x29, 0b0110, 0x23, 0b0100),
        (0x29, 0b0111, 0xc3, 0b0101),
        (0x29, 0b1000, 0x29, 0b0000),
        (0x29, 0b1001, 0x89, 0b0001),
        (0x29, 0b1010, 0x2f, 0b0000),
        (0x29, 0b1011, 0x8f, 0b0001),
        (0x29, 0b1100, 0x29, 0b0100),
        (0x29, 0b1101, 0xc9, 0b0101),
        (0x29, 0b1110, 0x23, 0b0100),
        (0x29, 0b1111, 0xc3, 0b0101),
        (0x2a, 0b0000, 0x30, 0b0000),
        (0x2a, 0b0001, 0x90, 0b0001),
        (0x2a, 0b0010, 0x30, 0b0000),
        (0x2a, 0b0011, 0x90, 0b0001),
        (0x2a, 0b0100, 0x2a, 0b0100),
        (0x2a, 0b0101, 0xca, 0b0101),
        (0x2a, 0b0110, 0x24, 0b0100),
        (0x2a, 0b0111, 0xc4, 0b0101),
        (0x2a, 0b1000, 0x30, 0b0000),
        (0x2a, 0b1001, 0x90, 0b0001),
        (0x2a, 0b1010, 0x30, 0b0000),
        (0x2a, 0b1011, 0x90, 0b0001),
        (0x2a, 0b1100, 0x2a, 0b0100),
        (0x2a, 0b1101, 0xca, 0b0101),
        (0x2a, 0b1110, 0x24, 0b0100),
        (0x2a, 0b1111, 0xc4, 0b0101),
        (0x2b, 0b0000, 0x31, 0b0000),
        (0x2b, 0b0001, 0x91, 0b0001),
        (0x2b, 0b0010, 0x31, 0b0000),
        (0x2b, 0b0011, 0x91, 0b0001),
        (0x2b, 0b0100, 0x2b, 0b0100),
        (0x2b, 0b0101, 0xcb, 0b0101),
        (0x2b, 0b0110, 0x25, 0b0100),
        (0x2b, 0b0111, 0xc5, 0b0101),
        (0x2b, 0b1000, 0x31, 0b0000),
        (0x2b, 0b1001, 0x91, 0b0001),
        (0x2b, 0b1010, 0x31, 0b0000),
        (0x2b, 0b1011, 0x91, 0b0001),
        (0x2b, 0b1100, 0x2b, 0b0100),
        (0x2b, 0b1101, 0xcb, 0b0101),
        (0x2b, 0b1110, 0x25, 0b0100),
        (0x2b, 0b1111, 0xc5, 0b0101),
        (0x2c, 0b0000, 0x32, 0b0000),
        (0x2c, 0b0001, 0x92, 0b0001),
        (0x2c, 0b0010, 0x32, 0b0000),
        (0x2c, 0b0011, 0x92, 0b0001),
        (0x2c, 0b0100, 0x2c, 0b0100),
        (0x2c, 0b0101, 0xcc, 0b0101),
        (0x2c, 0b0110, 0x26, 0b0100),
        (0x2c, 0b0111, 0xc6, 0b0101),
        (0x2c, 0b1000, 0x32, 0b0000),
        (0x2c, 0b1001, 0x92, 0b0001),
        (0x2c, 0b1010, 0x32, 0b0000),
        (0x2c, 0b1011, 0x92, 0b0001),
        (0x2c, 0b1100, 0x2c, 0b0100),
        (0x2c, 0b1101, 0xcc, 0b0101),
        (0x2c, 0b1110, 0x26, 0b0100),
        (0x2c, 0b1111, 0xc6, 0b0101),
        (0x2d, 0b0000, 0x33, 0b0000),
        (0x2d, 0b0001, 0x93, 0b0001),
        (0x2d, 0b0010, 0x33, 0b0000),
        (0x2d, 0b0011, 0x93, 0b0001),
        (0x2d, 0b0100, 0x2d, 0b0100),
        (0x2d, 0b0101, 0xcd, 0b0101),
        (0x2d, 0b0110, 0x27, 0b0100),
        (0x2d, 0b0111, 0xc7, 0b0101),
        (0x2d, 0b1000, 0x33, 0b0000),
        (0x2d, 0b1001, 0x93, 0b0001),
        (0x2d, 0b1010, 0x33, 0b0000),
        (0x2d, 0b1011, 0x93, 0b0001),
        (0x2d, 0b1100, 0x2d, 0b0100),
        (0x2d, 0b1101, 0xcd, 0b0101),
        (0x2d, 0b1110, 0x27, 0b0100),
        (0x2d, 0b1111, 0xc7, 0b0101),
        (0x2e, 0b0000, 0x34, 0b0000),
        (0x2e, 0b0001, 0x94, 0b0001),
        (0x2e, 0b0010, 0x34, 0b0000),
        (0x2e, 0b0011, 0x94, 0b0001),
        (0x2e, 0b0100, 0x2e, 0b0100),
        (0x2e, 0b0101, 0xce, 0b0101),
        (0x2e, 0b0110, 0x28, 0b0100),
        (0x2e, 0b0111, 0xc8, 0b0101),
        (0x2e, 0b1000, 0x34, 0b0000),
        (0x2e, 0b1001, 0x94, 0b0001),
        (0x2e, 0b1010, 0x34, 0b0000),
        (0x2e, 0b1011, 0x94, 0b0001),
        (0x2e, 0b1100, 0x2e, 0b0100),
        (0x2e, 0b1101, 0xce, 0b0101),
        (0x2e, 0b1110, 0x28, 0b0100),
        (0x2e, 0b1111, 0xc8, 0b0101),
        (0x2f, 0b0000, 0x35, 0b0000),
        (0x2f, 0b0001, 0x95, 0b0001),
        (0x2f, 0b0010, 0x35, 0b0000),
        (0x2f, 0b0011, 0x95, 0b0001),
        (0x2f, 0b0100, 0x2f, 0b0100),
        (0x2f, 0b0101, 0xcf, 0b0101),
        (0x2f, 0b0110, 0x29, 0b0100),
        (0x2f, 0b0111, 0xc9, 0b0101),
        (0x2f, 0b1000, 0x35, 0b0000),
        (0x2f, 0b1001, 0x95, 0b0001),
        (0x2f, 0b1010, 0x35, 0b0000),
        (0x2f, 0b1011, 0x95, 0b0001),
        (0x2f, 0b1100, 0x2f, 0b0100),
        (0x2f, 0b1101, 0xcf, 0b0101),
        (0x2f, 0b1110, 0x29, 0b0100),
        (0x2f, 0b1111, 0xc9, 0b0101),
        (0x30, 0b0000, 0x30, 0b0000),
        (0x30, 0b0001, 0x90, 0b0001),
        (0x30, 0b0010, 0x36, 0b0000),
        (0x30, 0b0011, 0x96, 0b0001),
        (0x30, 0b0100, 0x30, 0b0100),
        (0x30, 0b0101, 0xd0, 0b0101),
        (0x30, 0b0110, 0x2a, 0b0100),
        (0x30, 0b0111, 0xca, 0b0101),
        (0x30, 0b1000, 0x30, 0b0000),
        (0x30, 0b1001, 0x90, 0b0001),
        (0x30, 0b1010, 0x36, 0b0000),
        (0x30, 0b1011, 0x96, 0b0001),
        (0x30, 0b1100, 0x30, 0b0100),
        (0x30, 0b1101, 0xd0, 0b0101),
        (0x30, 0b1110, 0x2a, 0b0100),
        (0x30, 0b1111, 0xca, 0b0101),
        (0x31, 0b0000, 0x31, 0b0000),
        (0x31, 0b0001, 0x91, 0b0001),
        (0x31, 0b0010, 0x37, 0b0000),
        (0x31, 0b0011, 0x97, 0b0001),
        (0x31, 0b0100, 0x31, 0b0100),
        (0x31, 0b0101, 0xd1, 0b0101),
        (0x31, 0b0110, 0x2b, 0b0100),
        (0x31, 0b0111, 0xcb, 0b0101),
        (0x31, 0b1000, 0x31, 0b0000),
        (0x31, 0b1001, 0x91, 0b0001),
        (0x31, 0b1010, 0x37, 0b0000),
        (0x31, 0b1011, 0x97, 0b0001),
        (0x31, 0b1100, 0x31, 0b0100),
        (0x31, 0b1101, 0xd1, 0b0101),
        (0x31, 0b1110, 0x2b, 0b0100),
        (0x31, 0b1111, 0xcb, 0b0101),
        (0x32, 0b0000, 0x32, 0b0000),
        (0x32, 0b0001, 0x92, 0b0001),
        (0x32, 0b0010, 0x38, 0b0000),
        (0x32, 0b0011, 0x98, 0b0001),
        (0x32, 0b0100, 0x32, 0b0100),
        (0x32, 0b0101, 0xd2, 0b0101),
        (0x32, 0b0110, 0x2c, 0b0100),
        (0x32, 0b0111, 0xcc, 0b0101),
        (0x32, 0b1000, 0x32, 0b0000),
        (0x32, 0b1001, 0x92, 0b0001),
        (0x32, 0b1010, 0x38, 0b0000),
        (0x32, 0b1011, 0x98, 0b0001),
        (0x32, 0b1100, 0x32, 0b0100),
        (0x32, 0b1101, 0xd2, 0b0101),
        (0x32, 0b1110, 0x2c, 0b0100),
        (0x32, 0b1111, 0xcc, 0b0101),
        (0x33, 0b0000, 0x33, 0b0000),
        (0x33, 0b0001, 0x93, 0b0001),
        (0x33, 0b0010, 0x39, 0b0000),
        (0x33, 0b0011, 0x99, 0b0001),
        (0x33, 0b0100, 0x33, 0b0100),
        (0x33, 0b0101, 0xd3, 0b0101),
        (0x33, 0b0110, 0x2d, 0b0100),
        (0x33, 0b0111, 0xcd, 0b0101),
        (0x33, 0b1000, 0x33, 0b0000),
        (0x33, 0b1001, 0x93, 0b0001),
        (0x33, 0b1010, 0x39, 0b0000),
        (0x33, 0b1011, 0x99, 0b0001),
        (0x33, 0b1100, 0x33, 0b0100),
        (0x33, 0b1101, 0xd3, 0b0101),
        (0x33, 0b1110, 0x2d, 0b0100),
        (0x33, 0b1111, 0xcd, 0b0101),
        (0x34, 0b0000, 0x34, 0b0000),
        (0x34, 0b0001, 0x94, 0b0001),
        (0x34, 0b0010, 0x3a, 0b0000),
        (0x34, 0b0011, 0x9a, 0b0001),
        (0x34, 0b0100, 0x34, 0b0100),
        (0x34, 0b0101, 0xd4, 0b0101),
        (0x34, 0b0110, 0x2e, 0b0100),
        (0x34, 0b0111, 0xce, 0b0101),
        (0x34, 0b1000, 0x34, 0b0000),
        (0x34, 0b1001, 0x94, 0b0001),
        (0x34, 0b1010, 0x3a, 0b0000),
        (0x34, 0b1011, 0x9a, 0b0001),
        (0x34, 0b1100, 0x34, 0b0100),
        (0x34, 0b1101, 0xd4, 0b0101),
        (0x34, 0b1110, 0x2e, 0b0100),
        (0x34, 0b1111, 0xce, 0b0101),
        (0x35, 0b0000, 0x35, 0b0000),
        (0x35, 0b0001, 0x95, 0b0001),
        (0x35, 0b0010, 0x3b, 0b0000),
        (0x35, 0b0011, 0x9b, 0b0001),
        (0x35, 0b0100, 0x35, 0b0100),
        (0x35, 0b0101, 0xd5, 0b0101),
        (0x35, 0b0110, 0x2f, 0b0100),
        (0x35, 0b0111, 0xcf, 0b0101),
        (0x35, 0b1000, 0x35, 0b0000),
        (0x35, 0b1001, 0x95, 0b0001),
        (0x35, 0b1010, 0x3b, 0b0000),
        (0x35, 0b1011, 0x9b, 0b0001),
        (0x35, 0b1100, 0x35, 0b0100),
        (0x35, 0b1101, 0xd5, 0b0101),
        (0x35, 0b1110, 0x2f, 0b0100),
        (0x35, 0b1111, 0xcf, 0b0101),
        (0x36, 0b0000, 0x36, 0b0000),
        (0x36, 0b0001, 0x96, 0b0001),
        (0x36, 0b0010, 0x3c, 0b0000),
        (0x36, 0b0011, 0x9c, 0b0001),
        (0x36, 0b0100, 0x36, 0b0100),
        (0x36, 0b0101, 0xd6, 0b0101),
        (0x36, 0b0110, 0x30, 0b0100),
        (0x36, 0b0111, 0xd0, 0b0101),
        (0x36, 0b1000, 0x36, 0b0000),
        (0x36, 0b1001, 0x96, 0b0001),
        (0x36, 0b1010, 0x3c, 0b0000),
        (0x36, 0b1011, 0x9c, 0b0001),
        (0x36, 0b1100, 0x36, 0b0100),
        (0x36, 0b1101, 0xd6, 0b0101),
        (0x36, 0b1110, 0x30, 0b0100),
        (0x36, 0b1111, 0xd0, 0b0101),
        (0x37, 0b0000, 0x37, 0b0000),
        (0x37, 0b0001, 0x97, 0b0001),
        (0x37, 0b0010, 0x3d, 0b0000),
        (0x37, 0b0011, 0x9d, 0b0001),
        (0x37, 0b0100, 0x37, 0b0100),
        (0x37, 0b0101, 0xd7, 0b0101),
        (0x37, 0b0110, 0x31, 0b0100),
        (0x37, 0b0111, 0xd1, 0b0101),
        (0x37, 0b1000, 0x37, 0b0000),
        (0x37, 0b1001, 0x97, 0b0001),
        (0x37, 0b1010, 0x3d, 0b0000),
        (0x37, 0b1011, 0x9d, 0b0001),
        (0x37, 0b1100, 0x37, 0b0100),
        (0x37, 0b1101, 0xd7, 0b0101),
        (0x37, 0b1110, 0x31, 0b0100),
        (0x37, 0b1111, 0xd1, 0b0101),
        (0x38, 0b0000, 0x38, 0b0000),
        (0x38, 0b0001, 0x98, 0b0001),
        (0x38, 0b0010, 0x3e, 0b0000),
        (0x38, 0b0011, 0x9e, 0b0001),
        (0x38, 0b0100, 0x38, 0b0100),
        (0x38, 0b0101, 0xd8, 0b0101),
        (0x38, 0b0110, 0x32, 0b0100),
        (0x38, 0b0111, 0xd2, 0b0101),
        (0x38, 0b1000, 0x38, 0b0000),
        (0x38, 0b1001, 0x98, 0b0001),
        (0x38, 0b1010, 0x3e, 0b0000),
        (0x38, 0b1011, 0x9e, 0b0001),
        (0x38, 0b1100, 0x38, 0b0100),
        (0x38, 0b1101, 0xd8, 0b0101),
        (0x38, 0b1110, 0x32, 0b0100),
        (0x38, 0b1111, 0xd2, 0b0101),
        (0x39, 0b0000, 0x39, 0b0000),
        (0x39, 0b0001, 0x99, 0b0001),
        (0x39, 0b0010, 0x3f, 0b0000),
        (0x39, 0b0011, 0x9f, 0b0001),
        (0x39, 0b0100, 0x39, 0b0100),
        (0x39, 0b0101, 0xd9, 0b0101),
        (0x39, 0b0110, 0x33, 0b0100),
        (0x39, 0b0111, 0xd3, 0b0101),
        (0x39, 0b1000, 0x39, 0b0000),
        (0x39, 0b1001, 0x99, 0b0001),
        (0x39, 0b1010, 0x3f, 0b0000),
        (0x39, 0b1011, 0x9f, 0b0001),
        (0x39, 0b1100, 0x39, 0b0100),
        (0x39, 0b1101, 0xd9, 0b0101),
        (0x39, 0b1110, 0x33, 0b0100),
        (0x39, 0b1111, 0xd3, 0b0101),
        (0x3a, 0b0000, 0x40, 0b0000),
        (0x3a, 0b0001, 0xa0, 0b0001),
        (0x3a, 0b0010, 0x40, 0b0000),
        (0x3a, 0b0011, 0xa0, 0b0001),
        (0x3a, 0b0100, 0x3a, 0b0100),
        (0x3a, 0b0101, 0xda, 0b0101),
        (0x3a, 0b0110, 0x34, 0b0100),
        (0x3a, 0b0111, 0xd4, 0b0101),
        (0x3a, 0b1000, 0x40, 0b0000),
        (0x3a, 0b1001, 0xa0, 0b0001),
        (0x3a, 0b1010, 0x40, 0b0000),
        (0x3a, 0b1011, 0xa0, 0b0001),
        (0x3a, 0b1100, 0x3a, 0b0100),
        (0x3a, 0b1101, 0xda, 0b0101),
        (0x3a, 0b1110, 0x34, 0b0100),
        (0x3a, 0b1111, 0xd4, 0b0101),
        (0x3b, 0b0000, 0x41, 0b0000),
        (0x3b, 0b0001, 0xa1, 0b0001),
        (0x3b, 0b0010, 0x41, 0b0000),
        (0x3b, 0b0011, 0xa1, 0b0001),
        (0x3b, 0b0100, 0x3b, 0b0100),
        (0x3b, 0b0101, 0xdb, 0b0101),
        (0x3b, 0b0110, 0x35, 0b0100),
        (0x3b, 0b0111, 0xd5, 0b0101),
        (0x3b, 0b1000, 0x41, 0b0000),
        (0x3b, 0b1001, 0xa1, 0b0001),
        (0x3b, 0b1010, 0x41, 0b0000),
        (0x3b, 0b1011, 0xa1, 0b0001),
        (0x3b, 0b1100, 0x3b, 0b0100),
        (0x3b, 0b1101, 0xdb, 0b0101),
        (0x3b, 0b1110, 0x35, 0b0100),
        (0x3b, 0b1111, 0xd5, 0b0101),
        (0x3c, 0b0000, 0x42, 0b0000),
        (0x3c, 0b0001, 0xa2, 0b0001),
        (0x3c, 0b0010, 0x42, 0b0000),
        (0x3c, 0b0011, 0xa2, 0b0001),
        (0x3c, 0b0100, 0x3c, 0b0100),
        (0x3c, 0b0101, 0xdc, 0b0101),
        (0x3c, 0b0110, 0x36, 0b0100),
        (0x3c, 0b0111, 0xd6, 0b0101),
        (0x3c, 0b1000, 0x42, 0b0000),
        (0x3c, 0b1001, 0xa2, 0b0001),
        (0x3c, 0b1010, 0x42, 0b0000),
        (0x3c, 0b1011, 0xa2, 0b0001),
        (0x3c, 0b1100, 0x3c, 0b0100),
        (0x3c, 0b1101, 0xdc, 0b0101),
        (0x3c, 0b1110, 0x36, 0b0100),
        (0x3c, 0b1111, 0xd6, 0b0101),
        (0x3d, 0b0000, 0x43, 0b0000),
        (0x3d, 0b0001, 0xa3, 0b0001),
        (0x3d, 0b0010, 0x43, 0b0000),
        (0x3d, 0b0011, 0xa3, 0b0001),
        (0x3d, 0b0100, 0x3d, 0b0100),
        (0x3d, 0b0101, 0xdd, 0b0101),
        (0x3d, 0b0110, 0x37, 0b0100),
        (0x3d, 0b0111, 0xd7, 0b0101),
        (0x3d, 0b1000, 0x43, 0b0000),
        (0x3d, 0b1001, 0xa3, 0b0001),
        (0x3d, 0b1010, 0x43, 0b0000),
        (0x3d, 0b1011, 0xa3, 0b0001),
        (0x3d, 0b1100, 0x3d, 0b0100),
        (0x3d, 0b1101, 0xdd, 0b0101),
        (0x3d, 0b1110, 0x37, 0b0100),
        (0x3d, 0b1111, 0xd7, 0b0101),
        (0x3e, 0b0000, 0x44, 0b0000),
        (0x3e, 0b0001, 0xa4, 0b0001),
        (0x3e, 0b0010, 0x44, 0b0000),
        (0x3e, 0b0011, 0xa4, 0b0001),
        (0x3e, 0b0100, 0x3e, 0b0100),
        (0x3e, 0b0101, 0xde, 0b0101),
        (0x3e, 0b0110, 0x38, 0b0100),
        (0x3e, 0b0111, 0xd8, 0b0101),
        (0x3e, 0b1000, 0x44, 0b0000),
        (0x3e, 0b1001, 0xa4, 0b0001),
        (0x3e, 0b1010, 0x44, 0b0000),
        (0x3e, 0b1011, 0xa4, 0b0001),
        (0x3e, 0b1100, 0x3e, 0b0100),
        (0x3e, 0b1101, 0xde, 0b0101),
        (0x3e, 0b1110, 0x38, 0b0100),
        (0x3e, 0b1111, 0xd8, 0b0101),
        (0x3f, 0b0000, 0x45, 0b0000),
        (0x3f, 0b0001, 0xa5, 0b0001),
        (0x3f, 0b0010, 0x45, 0b0000),
        (0x3f, 0b0011, 0xa5, 0b0001),
        (0x3f, 0b0100, 0x3f, 0b0100),
        (0x3f, 0b0101, 0xdf, 0b0101),
        (0x3f, 0b0110, 0x39, 0b0100),
        (0x3f, 0b0111, 0xd9, 0b0101),
        (0x3f, 0b1000, 0x45, 0b0000),
        (0x3f, 0b1001, 0xa5, 0b0001),
        (0x3f, 0b1010, 0x45, 0b0000),
        (0x3f, 0b1011, 0xa5, 0b0001),
        (0x3f, 0b1100, 0x3f, 0b0100),
        (0x3f, 0b1101, 0xdf, 0b0101),
        (0x3f, 0b1110, 0x39, 0b0100),
        (0x3f, 0b1111, 0xd9, 0b0101),
        (0x40, 0b0000, 0x40, 0b0000),
        (0x40, 0b0001, 0xa0, 0b0001),
        (0x40, 0b0010, 0x46, 0b0000),
        (0x40, 0b0011, 0xa6, 0b0001),
        (0x40, 0b0100, 0x40, 0b0100),
        (0x40, 0b0101, 0xe0, 0b0101),
        (0x40, 0b0110, 0x3a, 0b0100),
        (0x40, 0b0111, 0xda, 0b0101),
        (0x40, 0b1000, 0x40, 0b0000),
        (0x40, 0b1001, 0xa0, 0b0001),
        (0x40, 0b1010, 0x46, 0b0000),
        (0x40, 0b1011, 0xa6, 0b0001),
        (0x40, 0b1100, 0x40, 0b0100),
        (0x40, 0b1101, 0xe0, 0b0101),
        (0x40, 0b1110, 0x3a, 0b0100),
        (0x40, 0b1111, 0xda, 0b0101),
        (0x41, 0b0000, 0x41, 0b0000),
        (0x41, 0b0001, 0xa1, 0b0001),
        (0x41, 0b0010, 0x47, 0b0000),
        (0x41, 0b0011, 0xa7, 0b0001),
        (0x41, 0b0100, 0x41, 0b0100),
        (0x41, 0b0101, 0xe1, 0b0101),
        (0x41, 0b0110, 0x3b, 0b0100),
        (0x41, 0b0111, 0xdb, 0b0101),
        (0x41, 0b1000, 0x41, 0b0000),
        (0x41, 0b1001, 0xa1, 0b0001),
        (0x41, 0b1010, 0x47, 0b0000),
        (0x41, 0b1011, 0xa7, 0b0001),
        (0x41, 0b1100, 0x41, 0b0100),
        (0x41, 0b1101, 0xe1, 0b0101),
        (0x41, 0b1110, 0x3b, 0b0100),
        (0x41, 0b1111, 0xdb, 0b0101),
        (0x42, 0b0000, 0x42, 0b0000),
        (0x42, 0b0001, 0xa2, 0b0001),
        (0x42, 0b0010, 0x48, 0b0000),
        (0x42, 0b0011, 0xa8, 0b0001),
        (0x42, 0b0100, 0x42, 0b0100),
        (0x42, 0b0101, 0xe2, 0b0101),
        (0x42, 0b0110, 0x3c, 0b0100),
        (0x42, 0b0111, 0xdc, 0b0101),
        (0x42, 0b1000, 0x42, 0b0000),
        (0x42, 0b1001, 0xa2, 0b0001),
        (0x42, 0b1010, 0x48, 0b0000),
        (0x42, 0b1011, 0xa8, 0b0001),
        (0x42, 0b1100, 0x42, 0b0100),
        (0x42, 0b1101, 0xe2, 0b0101),
        (0x42, 0b1110, 0x3c, 0b0100),
        (0x42, 0b1111, 0xdc, 0b0101),
        (0x43, 0b0000, 0x43, 0b0000),
        (0x43, 0b0001, 0xa3, 0b0001),
        (0x43, 0b0010, 0x49, 0b0000),
        (0x43, 0b0011, 0xa9, 0b0001),
        (0x43, 0b0100, 0x43, 0b0100),
        (0x43, 0b0101, 0xe3, 0b0101),
        (0x43, 0b0110, 0x3d, 0b0100),
        (0x43, 0b0111, 0xdd, 0b0101),
        (0x43, 0b1000, 0x43, 0b0000),
        (0x43, 0b1001, 0xa3, 0b0001),
        (0x43, 0b1010, 0x49, 0b0000),
        (0x43, 0b1011, 0xa9, 0b0001),
        (0x43, 0b1100, 0x43, 0b0100),
        (0x43, 0b1101, 0xe3, 0b0101),
        (0x43, 0b1110, 0x3d, 0b0100),
        (0x43, 0b1111, 0xdd, 0b0101),
        (0x44, 0b0000, 0x44, 0b0000),
        (0x44, 0b0001, 0xa4, 0b0001),
        (0x44, 0b0010, 0x4a, 0b0000),
        (0x44, 0b0011, 0xaa, 0b0001),
        (0x44, 0b0100, 0x44, 0b0100),
        (0x44, 0b0101, 0xe4, 0b0101),
        (0x44, 0b0110, 0x3e, 0b0100),
        (0x44, 0b0111, 0xde, 0b0101),
        (0x44, 0b1000, 0x44, 0b0000),
        (0x44, 0b1001, 0xa4, 0b0001),
        (0x44, 0b1010, 0x4a, 0b0000),
        (0x44, 0b1011, 0xaa, 0b0001),
        (0x44, 0b1100, 0x44, 0b0100),
        (0x44, 0b1101, 0xe4, 0b0101),
        (0x44, 0b1110, 0x3e, 0b0100),
        (0x44, 0b1111, 0xde, 0b0101),
        (0x45, 0b0000, 0x45, 0b0000),
        (0x45, 0b0001, 0xa5, 0b0001),
        (0x45, 0b0010, 0x4b, 0b0000),
        (0x45, 0b0011, 0xab, 0b0001),
        (0x45, 0b0100, 0x45, 0b0100),
        (0x45, 0b0101, 0xe5, 0b0101),
        (0x45, 0b0110, 0x3f, 0b0100),
        (0x45, 0b0111, 0xdf, 0b0101),
        (0x45, 0b1000, 0x45, 0b0000),
        (0x45, 0b1001, 0xa5, 0b0001),
        (0x45, 0b1010, 0x4b, 0b0000),
        (0x45, 0b1011, 0xab, 0b0001),
        (0x45, 0b1100, 0x45, 0b0100),
        (0x45, 0b1101, 0xe5, 0b0101),
        (0x45, 0b1110, 0x3f, 0b0100),
        (0x45, 0b1111, 0xdf, 0b0101),
        (0x46, 0b0000, 0x46, 0b0000),
        (0x46, 0b0001, 0xa6, 0b0001),
        (0x46, 0b0010, 0x4c, 0b0000),
        (0x46, 0b0011, 0xac, 0b0001),
        (0x46, 0b0100, 0x46, 0b0100),
        (0x46, 0b0101, 0xe6, 0b0101),
        (0x46, 0b0110, 0x40, 0b0100),
        (0x46, 0b0111, 0xe0, 0b0101),
        (0x46, 0b1000, 0x46, 0b0000),
        (0x46, 0b1001, 0xa6, 0b0001),
        (0x46, 0b1010, 0x4c, 0b0000),
        (0x46, 0b1011, 0xac, 0b0001),
        (0x46, 0b1100, 0x46, 0b0100),
        (0x46, 0b1101, 0xe6, 0b0101),
        (0x46, 0b1110, 0x40, 0b0100),
        (0x46, 0b1111, 0xe0, 0b0101),
        (0x47, 0b0000, 0x47, 0b0000),
        (0x47, 0b0001, 0xa7, 0b0001),
        (0x47, 0b0010, 0x4d, 0b0000),
        (0x47, 0b0011, 0xad, 0b0001),
        (0x47, 0b0100, 0x47, 0b0100),
        (0x47, 0b0101, 0xe7, 0b0101),
        (0x47, 0b0110, 0x41, 0b0100),
        (0x47, 0b0111, 0xe1, 0b0101),
        (0x47, 0b1000, 0x47, 0b0000),
        (0x47, 0b1001, 0xa7, 0b0001),
        (0x47, 0b1010, 0x4d, 0b0000),
        (0x47, 0b1011, 0xad, 0b0001),
        (0x47, 0b1100, 0x47, 0b0100),
        (0x47, 0b1101, 0xe7, 0b0101),
        (0x47, 0b1110, 0x41, 0b0100),
        (0x47, 0b1111, 0xe1, 0b0101),
        (0x48, 0b0000, 0x48, 0b0000),
        (0x48, 0b0001, 0xa8, 0b0001),
        (0x48, 0b0010, 0x4e, 0b0000),
        (0x48, 0b0011, 0xae, 0b0001),
        (0x48, 0b0100, 0x48, 0b0100),
        (0x48, 0b0101, 0xe8, 0b0101),
        (0x48, 0b0110, 0x42, 0b0100),
        (0x48, 0b0111, 0xe2, 0b0101),
        (0x48, 0b1000, 0x48, 0b0000),
        (0x48, 0b1001, 0xa8, 0b0001),
        (0x48, 0b1010, 0x4e, 0b0000),
        (0x48, 0b1011, 0xae, 0b0001),
        (0x48, 0b1100, 0x48, 0b0100),
        (0x48, 0b1101, 0xe8, 0b0101),
        (0x48, 0b1110, 0x42, 0b0100),
        (0x48, 0b1111, 0xe2, 0b0101),
        (0x49, 0b0000, 0x49, 0b0000),
        (0x49, 0b0001, 0xa9, 0b0001),
        (0x49, 0b0010, 0x4f, 0b0000),
        (0x49, 0b0011, 0xaf, 0b0001),
        (0x49, 0b0100, 0x49, 0b0100),
        (0x49, 0b0101, 0xe9, 0b0101),
        (0x49, 0b0110, 0x43, 0b0100),
        (0x49, 0b0111, 0xe3, 0b0101),
        (0x49, 0b1000, 0x49, 0b0000),
        (0x49, 0b1001, 0xa9, 0b0001),
        (0x49, 0b1010, 0x4f, 0b0000),
        (0x49, 0b1011, 0xaf, 0b0001),
        (0x49, 0b1100, 0x49, 0b0100),
        (0x49, 0b1101, 0xe9, 0b0101),
        (0x49, 0b1110, 0x43, 0b0100),
        (0x49, 0b1111, 0xe3, 0b0101),
        (0x4a, 0b0000, 0x50, 0b0000),
        (0x4a, 0b0001, 0xb0, 0b0001),
        (0x4a, 0b0010, 0x50, 0b0000),
        (0x4a, 0b0011, 0xb0, 0b0001),
        (0x4a, 0b0100, 0x4a, 0b0100),
        (0x4a, 0b0101, 0xea, 0b0101),
        (0x4a, 0b0110, 0x44, 0b0100),
        (0x4a, 0b0111, 0xe4, 0b0101),
        (0x4a, 0b1000, 0x50, 0b0000),
        (0x4a, 0b1001, 0xb0, 0b0001),
        (0x4a, 0b1010, 0x50, 0b0000),
        (0x4a, 0b1011, 0xb0, 0b0001),
        (0x4a, 0b1100, 0x4a, 0b0100),
        (0x4a, 0b1101, 0xea, 0b0101),
        (0x4a, 0b1110, 0x44, 0b0100),
        (0x4a, 0b1111, 0xe4, 0b0101),
        (0x4b, 0b0000, 0x51, 0b0000),
        (0x4b, 0b0001, 0xb1, 0b0001),
        (0x4b, 0b0010, 0x51, 0b0000),
        (0x4b, 0b0011, 0xb1, 0b0001),
        (0x4b, 0b0100, 0x4b, 0b0100),
        (0x4b, 0b0101, 0xeb, 0b0101),
        (0x4b, 0b0110, 0x45, 0b0100),
        (0x4b, 0b0111, 0xe5, 0b0101),
        (0x4b, 0b1000, 0x51, 0b0000),
        (0x4b, 0b1001, 0xb1, 0b0001),
        (0x4b, 0b1010, 0x51, 0b0000),
        (0x4b, 0b1011, 0xb1, 0b0001),
        (0x4b, 0b1100, 0x4b, 0b0100),
        (0x4b, 0b1101, 0xeb, 0b0101),
        (0x4b, 0b1110, 0x45, 0b0100),
        (0x4b, 0b1111, 0xe5, 0b0101),
        (0x4c, 0b0000, 0x52, 0b0000),
        (0x4c, 0b0001, 0xb2, 0b0001),
        (0x4c, 0b0010, 0x52, 0b0000),
        (0x4c, 0b0011, 0xb2, 0b0001),
        (0x4c, 0b0100, 0x4c, 0b0100),
        (0x4c, 0b0101, 0xec, 0b0101),
        (0x4c, 0b0110, 0x46, 0b0100),
        (0x4c, 0b0111, 0xe6, 0b0101),
        (0x4c, 0b1000, 0x52, 0b0000),
        (0x4c, 0b1001, 0xb2, 0b0001),
        (0x4c, 0b1010, 0x52, 0b0000),
        (0x4c, 0b1011, 0xb2, 0b0001),
        (0x4c, 0b1100, 0x4c, 0b0100),
        (0x4c, 0b1101, 0xec, 0b0101),
        (0x4c, 0b1110, 0x46, 0b0100),
        (0x4c, 0b1111, 0xe6, 0b0101),
        (0x4d, 0b0000, 0x53, 0b0000),
        (0x4d, 0b0001, 0xb3, 0b0001),
        (0x4d, 0b0010, 0x53, 0b0000),
        (0x4d, 0b0011, 0xb3, 0b0001),
        (0x4d, 0b0100, 0x4d, 0b0100),
        (0x4d, 0b0101, 0xed, 0b0101),
        (0x4d, 0b0110, 0x47, 0b0100),
        (0x4d, 0b0111, 0xe7, 0b0101),
        (0x4d, 0b1000, 0x53, 0b0000),
        (0x4d, 0b1001, 0xb3, 0b0001),
        (0x4d, 0b1010, 0x53, 0b0000),
        (0x4d, 0b1011, 0xb3, 0b0001),
        (0x4d, 0b1100, 0x4d, 0b0100),
        (0x4d, 0b1101, 0xed, 0b0101),
        (0x4d, 0b1110, 0x47, 0b0100),
        (0x4d, 0b1111, 0xe7, 0b0101),
        (0x4e, 0b0000, 0x54, 0b0000),
        (0x4e, 0b0001, 0xb4, 0b0001),
        (0x4e, 0b0010, 0x54, 0b0000),
        (0x4e, 0b0011, 0xb4, 0b0001),
        (0x4e, 0b0100, 0x4e, 0b0100),
        (0x4e, 0b0101, 0xee, 0b0101),
        (0x4e, 0b0110, 0x48, 0b0100),
        (0x4e, 0b0111, 0xe8, 0b0101),
        (0x4e, 0b1000, 0x54, 0b0000),
        (0x4e, 0b1001, 0xb4, 0b0001),
        (0x4e, 0b1010, 0x54, 0b0000),
        (0x4e, 0b1011, 0xb4, 0b0001),
        (0x4e, 0b1100, 0x4e, 0b0100),
        (0x4e, 0b1101, 0xee, 0b0101),
        (0x4e, 0b1110, 0x48, 0b0100),
        (0x4e, 0b1111, 0xe8, 0b0101),
        (0x4f, 0b0000, 0x55, 0b0000),
        (0x4f, 0b0001, 0xb5, 0b0001),
        (0x4f, 0b0010, 0x55, 0b0000),
        (0x4f, 0b0011, 0xb5, 0b0001),
        (0x4f, 0b0100, 0x4f, 0b0100),
        (0x4f, 0b0101, 0xef, 0b0101),
        (0x4f, 0b0110, 0x49, 0b0100),
        (0x4f, 0b0111, 0xe9, 0b0101),
        (0x4f, 0b1000, 0x55, 0b0000),
        (0x4f, 0b1001, 0xb5, 0b0001),
        (0x4f, 0b1010, 0x55, 0b0000),
        (0x4f, 0b1011, 0xb5, 0b0001),
        (0x4f, 0b1100, 0x4f, 0b0100),
        (0x4f, 0b1101, 0xef, 0b0101),
        (0x4f, 0b1110, 0x49, 0b0100),
        (0x4f, 0b1111, 0xe9, 0b0101),
        (0x50, 0b0000, 0x50, 0b0000),
        (0x50, 0b0001, 0xb0, 0b0001),
        (0x50, 0b0010, 0x56, 0b0000),
        (0x50, 0b0011, 0xb6, 0b0001),
        (0x50, 0b0100, 0x50, 0b0100),
        (0x50, 0b0101, 0xf0, 0b0101),
        (0x50, 0b0110, 0x4a, 0b0100),
        (0x50, 0b0111, 0xea, 0b0101),
        (0x50, 0b1000, 0x50, 0b0000),
        (0x50, 0b1001, 0xb0, 0b0001),
        (0x50, 0b1010, 0x56, 0b0000),
        (0x50, 0b1011, 0xb6, 0b0001),
        (0x50, 0b1100, 0x50, 0b0100),
        (0x50, 0b1101, 0xf0, 0b0101),
        (0x50, 0b1110, 0x4a, 0b0100),
        (0x50, 0b1111, 0xea, 0b0101),
        (0x51, 0b0000, 0x51, 0b0000),
        (0x51, 0b0001, 0xb1, 0b0001),
        (0x51, 0b0010, 0x57, 0b0000),
        (0x51, 0b0011, 0xb7, 0b0001),
        (0x51, 0b0100, 0x51, 0b0100),
        (0x51, 0b0101, 0xf1, 0b0101),
        (0x51, 0b0110, 0x4b, 0b0100),
        (0x51, 0b0111, 0xeb, 0b0101),
        (0x51, 0b1000, 0x51, 0b0000),
        (0x51, 0b1001, 0xb1, 0b0001),
        (0x51, 0b1010, 0x57, 0b0000),
        (0x51, 0b1011, 0xb7, 0b0001),
        (0x51, 0b1100, 0x51, 0b0100),
        (0x51, 0b1101, 0xf1, 0b0101),
        (0x51, 0b1110, 0x4b, 0b0100),
        (0x51, 0b1111, 0xeb, 0b0101),
        (0x52, 0b0000, 0x52, 0b0000),
        (0x52, 0b0001, 0xb2, 0b0001),
        (0x52, 0b0010, 0x58, 0b0000),
        (0x52, 0b0011, 0xb8, 0b0001),
        (0x52, 0b0100, 0x52, 0b0100),
        (0x52, 0b0101, 0xf2, 0b0101),
        (0x52, 0b0110, 0x4c, 0b0100),
        (0x52, 0b0111, 0xec, 0b0101),
        (0x52, 0b1000, 0x52, 0b0000),
        (0x52, 0b1001, 0xb2, 0b0001),
        (0x52, 0b1010, 0x58, 0b0000),
        (0x52, 0b1011, 0xb8, 0b0001),
        (0x52, 0b1100, 0x52, 0b0100),
        (0x52, 0b1101, 0xf2, 0b0101),
        (0x52, 0b1110, 0x4c, 0b0100),
        (0x52, 0b1111, 0xec, 0b0101),
        (0x53, 0b0000, 0x53, 0b0000),
        (0x53, 0b0001, 0xb3, 0b0001),
        (0x53, 0b0010, 0x59, 0b0000),
        (0x53, 0b0011, 0xb9, 0b0001),
        (0x53, 0b0100, 0x53, 0b0100),
        (0x53, 0b0101, 0xf3, 0b0101),
        (0x53, 0b0110, 0x4d, 0b0100),
        (0x53, 0b0111, 0xed, 0b0101),
        (0x53, 0b1000, 0x53, 0b0000),
        (0x53, 0b1001, 0xb3, 0b0001),
        (0x53, 0b1010, 0x59, 0b0000),
        (0x53, 0b1011, 0xb9, 0b0001),
        (0x53, 0b1100, 0x53, 0b0100),
        (0x53, 0b1101, 0xf3, 0b0101),
        (0x53, 0b1110, 0x4d, 0b0100),
        (0x53, 0b1111, 0xed, 0b0101),
        (0x54, 0b0000, 0x54, 0b0000),
        (0x54, 0b0001, 0xb4, 0b0001),
        (0x54, 0b0010, 0x5a, 0b0000),
        (0x54, 0b0011, 0xba, 0b0001),
        (0x54, 0b0100, 0x54, 0b0100),
        (0x54, 0b0101, 0xf4, 0b0101),
        (0x54, 0b0110, 0x4e, 0b0100),
        (0x54, 0b0111, 0xee, 0b0101),
        (0x54, 0b1000, 0x54, 0b0000),
        (0x54, 0b1001, 0xb4, 0b0001),
        (0x54, 0b1010, 0x5a, 0b0000),
        (0x54, 0b1011, 0xba, 0b0001),
        (0x54, 0b1100, 0x54, 0b0100),
        (0x54, 0b1101, 0xf4, 0b0101),
        (0x54, 0b1110, 0x4e, 0b0100),
        (0x54, 0b1111, 0xee, 0b0101),
        (0x55, 0b0000, 0x55, 0b0000),
        (0x55, 0b0001, 0xb5, 0b0001),
        (0x55, 0b0010, 0x5b, 0b0000),
        (0x55, 0b0011, 0xbb, 0b0001),
        (0x55, 0b0100, 0x55, 0b0100),
        (0x55, 0b0101, 0xf5, 0b0101),
        (0x55, 0b0110, 0x4f, 0b0100),
        (0x55, 0b0111, 0xef, 0b0101),
        (0x55, 0b1000, 0x55, 0b0000),
        (0x55, 0b1001, 0xb5, 0b0001),
        (0x55, 0b1010, 0x5b, 0b0000),
        (0x55, 0b1011, 0xbb, 0b0001),
        (0x55, 0b1100, 0x55, 0b0100),
        (0x55, 0b1101, 0xf5, 0b0101),
        (0x55, 0b1110, 0x4f, 0b0100),
        (0x55, 0b1111, 0xef, 0b0101),
        (0x56, 0b0000, 0x56, 0b0000),
        (0x56, 0b0001, 0xb6, 0b0001),
        (0x56, 0b0010, 0x5c, 0b0000),
        (0x56, 0b0011, 0xbc, 0b0001),
        (0x56, 0b0100, 0x56, 0b0100),
        (0x56, 0b0101, 0xf6, 0b0101),
        (0x56, 0b0110, 0x50, 0b0100),
        (0x56, 0b0111, 0xf0, 0b0101),
        (0x56, 0b1000, 0x56, 0b0000),
        (0x56, 0b1001, 0xb6, 0b0001),
        (0x56, 0b1010, 0x5c, 0b0000),
        (0x56, 0b1011, 0xbc, 0b0001),
        (0x56, 0b1100, 0x56, 0b0100),
        (0x56, 0b1101, 0xf6, 0b0101),
        (0x56, 0b1110, 0x50, 0b0100),
        (0x56, 0b1111, 0xf0, 0b0101),
        (0x57, 0b0000, 0x57, 0b0000),
        (0x57, 0b0001, 0xb7, 0b0001),
        (0x57, 0b0010, 0x5d, 0b0000),
        (0x57, 0b0011, 0xbd, 0b0001),
        (0x57, 0b0100, 0x57, 0b0100),
        (0x57, 0b0101, 0xf7, 0b0101),
        (0x57, 0b0110, 0x51, 0b0100),
        (0x57, 0b0111, 0xf1, 0b0101),
        (0x57, 0b1000, 0x57, 0b0000),
        (0x57, 0b1001, 0xb7, 0b0001),
        (0x57, 0b1010, 0x5d, 0b0000),
        (0x57, 0b1011, 0xbd, 0b0001),
        (0x57, 0b1100, 0x57, 0b0100),
        (0x57, 0b1101, 0xf7, 0b0101),
        (0x57, 0b1110, 0x51, 0b0100),
        (0x57, 0b1111, 0xf1, 0b0101),
        (0x58, 0b0000, 0x58, 0b0000),
        (0x58, 0b0001, 0xb8, 0b0001),
        (0x58, 0b0010, 0x5e, 0b0000),
        (0x58, 0b0011, 0xbe, 0b0001),
        (0x58, 0b0100, 0x58, 0b0100),
        (0x58, 0b0101, 0xf8, 0b0101),
        (0x58, 0b0110, 0x52, 0b0100),
        (0x58, 0b0111, 0xf2, 0b0101),
        (0x58, 0b1000, 0x58, 0b0000),
        (0x58, 0b1001, 0xb8, 0b0001),
        (0x58, 0b1010, 0x5e, 0b0000),
        (0x58, 0b1011, 0xbe, 0b0001),
        (0x58, 0b1100, 0x58, 0b0100),
        (0x58, 0b1101, 0xf8, 0b0101),
        (0x58, 0b1110, 0x52, 0b0100),
        (0x58, 0b1111, 0xf2, 0b0101),
        (0x59, 0b0000, 0x59, 0b0000),
        (0x59, 0b0001, 0xb9, 0b0001),
        (0x59, 0b0010, 0x5f, 0b0000),
        (0x59, 0b0011, 0xbf, 0b0001),
        (0x59, 0b0100, 0x59, 0b0100),
        (0x59, 0b0101, 0xf9, 0b0101),
        (0x59, 0b0110, 0x53, 0b0100),
        (0x59, 0b0111, 0xf3, 0b0101),
        (0x59, 0b1000, 0x59, 0b0000),
        (0x59, 0b1001, 0xb9, 0b0001),
        (0x59, 0b1010, 0x5f, 0b0000),
        (0x59, 0b1011, 0xbf, 0b0001),
        (0x59, 0b1100, 0x59, 0b0100),
        (0x59, 0b1101, 0xf9, 0b0101),
        (0x59, 0b1110, 0x53, 0b0100),
        (0x59, 0b1111, 0xf3, 0b0101),
        (0x5a, 0b0000, 0x60, 0b0000),
        (0x5a, 0b0001, 0xc0, 0b0001),
        (0x5a, 0b0010, 0x60, 0b0000),
        (0x5a, 0b0011, 0xc0, 0b0001),
        (0x5a, 0b0100, 0x5a, 0b0100),
        (0x5a, 0b0101, 0xfa, 0b0101),
        (0x5a, 0b0110, 0x54, 0b0100),
        (0x5a, 0b0111, 0xf4, 0b0101),
        (0x5a, 0b1000, 0x60, 0b0000),
        (0x5a, 0b1001, 0xc0, 0b0001),
        (0x5a, 0b1010, 0x60, 0b0000),
        (0x5a, 0b1011, 0xc0, 0b0001),
        (0x5a, 0b1100, 0x5a, 0b0100),
        (0x5a, 0b1101, 0xfa, 0b0101),
        (0x5a, 0b1110, 0x54, 0b0100),
        (0x5a, 0b1111, 0xf4, 0b0101),
        (0x5b, 0b0000, 0x61, 0b0000),
        (0x5b, 0b0001, 0xc1, 0b0001),
        (0x5b, 0b0010, 0x61, 0b0000),
        (0x5b, 0b0011, 0xc1, 0b0001),
        (0x5b, 0b0100, 0x5b, 0b0100),
        (0x5b, 0b0101, 0xfb, 0b0101),
        (0x5b, 0b0110, 0x55, 0b0100),
        (0x5b, 0b0111, 0xf5, 0b0101),
        (0x5b, 0b1000, 0x61, 0b0000),
        (0x5b, 0b1001, 0xc1, 0b0001),
        (0x5b, 0b1010, 0x61, 0b0000),
        (0x5b, 0b1011, 0xc1, 0b0001),
        (0x5b, 0b1100, 0x5b, 0b0100),
        (0x5b, 0b1101, 0xfb, 0b0101),
        (0x5b, 0b1110, 0x55, 0b0100),
        (0x5b, 0b1111, 0xf5, 0b0101),
        (0x5c, 0b0000, 0x62, 0b0000),
        (0x5c, 0b0001, 0xc2, 0b0001),
        (0x5c, 0b0010, 0x62, 0b0000),
        (0x5c, 0b0011, 0xc2, 0b0001),
        (0x5c, 0b0100, 0x5c, 0b0100),
        (0x5c, 0b0101, 0xfc, 0b0101),
        (0x5c, 0b0110, 0x56, 0b0100),
        (0x5c, 0b0111, 0xf6, 0b0101),
        (0x5c, 0b1000, 0x62, 0b0000),
        (0x5c, 0b1001, 0xc2, 0b0001),
        (0x5c, 0b1010, 0x62, 0b0000),
        (0x5c, 0b1011, 0xc2, 0b0001),
        (0x5c, 0b1100, 0x5c, 0b0100),
        (0x5c, 0b1101, 0xfc, 0b0101),
        (0x5c, 0b1110, 0x56, 0b0100),
        (0x5c, 0b1111, 0xf6, 0b0101),
        (0x5d, 0b0000, 0x63, 0b0000),
        (0x5d, 0b0001, 0xc3, 0b0001),
        (0x5d, 0b0010, 0x63, 0b0000),
        (0x5d, 0b0011, 0xc3, 0b0001),
        (0x5d, 0b0100, 0x5d, 0b0100),
        (0x5d, 0b0101, 0xfd, 0b0101),
        (0x5d, 0b0110, 0x57, 0b0100),
        (0x5d, 0b0111, 0xf7, 0b0101),
        (0x5d, 0b1000, 0x63, 0b0000),
        (0x5d, 0b1001, 0xc3, 0b0001),
        (0x5d, 0b1010, 0x63, 0b0000),
        (0x5d, 0b1011, 0xc3, 0b0001),
        (0x5d, 0b1100, 0x5d, 0b0100),
        (0x5d, 0b1101, 0xfd, 0b0101),
        (0x5d, 0b1110, 0x57, 0b0100),
        (0x5d, 0b1111, 0xf7, 0b0101),
        (0x5e, 0b0000, 0x64, 0b0000),
        (0x5e, 0b0001, 0xc4, 0b0001),
        (0x5e, 0b0010, 0x64, 0b0000),
        (0x5e, 0b0011, 0xc4, 0b0001),
        (0x5e, 0b0100, 0x5e, 0b0100),
        (0x5e, 0b0101, 0xfe, 0b0101),
        (0x5e, 0b0110, 0x58, 0b0100),
        (0x5e, 0b0111, 0xf8, 0b0101),
        (0x5e, 0b1000, 0x64, 0b0000),
        (0x5e, 0b1001, 0xc4, 0b0001),
        (0x5e, 0b1010, 0x64, 0b0000),
        (0x5e, 0b1011, 0xc4, 0b0001),
        (0x5e, 0b1100, 0x5e, 0b0100),
        (0x5e, 0b1101, 0xfe, 0b0101),
        (0x5e, 0b1110, 0x58, 0b0100),
        (0x5e, 0b1111, 0xf8, 0b0101),
        (0x5f, 0b0000, 0x65, 0b0000),
        (0x5f, 0b0001, 0xc5, 0b0001),
        (0x5f, 0b0010, 0x65, 0b0000),
        (0x5f, 0b0011, 0xc5, 0b0001),
        (0x5f, 0b0100, 0x5f, 0b0100),
        (0x5f, 0b0101, 0xff, 0b0101),
        (0x5f, 0b0110, 0x59, 0b0100),
        (0x5f, 0b0111, 0xf9, 0b0101),
        (0x5f, 0b1000, 0x65, 0b0000),
        (0x5f, 0b1001, 0xc5, 0b0001),
        (0x5f, 0b1010, 0x65, 0b0000),
        (0x5f, 0b1011, 0xc5, 0b0001),
        (0x5f, 0b1100, 0x5f, 0b0100),
        (0x5f, 0b1101, 0xff, 0b0101),
        (0x5f, 0b1110, 0x59, 0b0100),
        (0x5f, 0b1111, 0xf9, 0b0101),
        (0x60, 0b0000, 0x60, 0b0000),
        (0x60, 0b0001, 0xc0, 0b0001),
        (0x60, 0b0010, 0x66, 0b0000),
        (0x60, 0b0011, 0xc6, 0b0001),
        (0x60, 0b0100, 0x60, 0b0100),
        (0x60, 0b0101, 0x00, 0b1101),
        (0x60, 0b0110, 0x5a, 0b0100),
        (0x60, 0b0111, 0xfa, 0b0101),
        (0x60, 0b1000, 0x60, 0b0000),
        (0x60, 0b1001, 0xc0, 0b0001),
        (0x60, 0b1010, 0x66, 0b0000),
        (0x60, 0b1011, 0xc6, 0b0001),
        (0x60, 0b1100, 0x60, 0b0100),
        (0x60, 0b1101, 0x00, 0b1101),
        (0x60, 0b1110, 0x5a, 0b0100),
        (0x60, 0b1111, 0xfa, 0b0101),
        (0x61, 0b0000, 0x61, 0b0000),
        (0x61, 0b0001, 0xc1, 0b0001),
        (0x61, 0b0010, 0x67, 0b0000),
        (0x61, 0b0011, 0xc7, 0b0001),
        (0x61, 0b0100, 0x61, 0b0100),
        (0x61, 0b0101, 0x01, 0b0101),
        (0x61, 0b0110, 0x5b, 0b0100),
        (0x61, 0b0111, 0xfb, 0b0101),
        (0x61, 0b1000, 0x61, 0b0000),
        (0x61, 0b1001, 0xc1, 0b0001),
        (0x61, 0b1010, 0x67, 0b0000),
        (0x61, 0b1011, 0xc7, 0b0001),
        (0x61, 0b1100, 0x61, 0b0100),
        (0x61, 0b1101, 0x01, 0b0101),
        (0x61, 0b1110, 0x5b, 0b0100),
        (0x61, 0b1111, 0xfb, 0b0101),
        (0x62, 0b0000, 0x62, 0b0000),
        (0x62, 0b0001, 0xc2, 0b0001),
        (0x62, 0b0010, 0x68, 0b0000),
        (0x62, 0b0011, 0xc8, 0b0001),
        (0x62, 0b0100, 0x62, 0b0100),
        (0x62, 0b0101, 0x02, 0b0101),
        (0x62, 0b0110, 0x5c, 0b0100),
        (0x62, 0b0111, 0xfc, 0b0101),
        (0x62, 0b1000, 0x62, 0b0000),
        (0x62, 0b1001, 0xc2, 0b0001),
        (0x62, 0b1010, 0x68, 0b0000),
        (0x62, 0b1011, 0xc8, 0b0001),
        (0x62, 0b1100, 0x62, 0b0100),
        (0x62, 0b1101, 0x02, 0b0101),
        (0x62, 0b1110, 0x5c, 0b0100),
        (0x62, 0b1111, 0xfc, 0b0101),
        (0x63, 0b0000, 0x63, 0b0000),
        (0x63, 0b0001, 0xc3, 0b0001),
        (0x63, 0b0010, 0x69, 0b0000),
        (0x63, 0b0011, 0xc9, 0b0001),
        (0x63, 0b0100, 0x63, 0b0100),
        (0x63, 0b0101, 0x03, 0b0101),
        (0x63, 0b0110, 0x5d, 0b0100),
        (0x63, 0b0111, 0xfd, 0b0101),
        (0x63, 0b1000, 0x63, 0b0000),
        (0x63, 0b1001, 0xc3, 0b0001),
        (0x63, 0b1010, 0x69, 0b0000),
        (0x63, 0b1011, 0xc9, 0b0001),
        (0x63, 0b1100, 0x63, 0b0100),
        (0x63, 0b1101, 0x03, 0b0101),
        (0x63, 0b1110, 0x5d, 0b0100),
        (0x63, 0b1111, 0xfd, 0b0101),
        (0x64, 0b0000, 0x64, 0b0000),
        (0x64, 0b0001, 0xc4, 0b0001),
        (0x64, 0b0010, 0x6a, 0b0000),
        (0x64, 0b0011, 0xca, 0b0001),
        (0x64, 0b0100, 0x64, 0b0100),
        (0x64, 0b0101, 0x04, 0b0101),
        (0x64, 0b0110, 0x5e, 0b0100),
        (0x64, 0b0111, 0xfe, 0b0101),
        (0x64, 0b1000, 0x64, 0b0000),
        (0x64, 0b1001, 0xc4, 0b0001),
        (0x64, 0b1010, 0x6a, 0b0000),
        (0x64, 0b1011, 0xca, 0b0001),
        (0x64, 0b1100, 0x64, 0b0100),
        (0x64, 0b1101, 0x04, 0b0101),
        (0x64, 0b1110, 0x5e, 0b0100),
        (0x64, 0b1111, 0xfe, 0b0101),
        (0x65, 0b0000, 0x65, 0b0000),
        (0x65, 0b0001, 0xc5, 0b0001),
        (0x65, 0b0010, 0x6b, 0b0000),
        (0x65, 0b0011, 0xcb, 0b0001),
        (0x65, 0b0100, 0x65, 0b0100),
        (0x65, 0b0101, 0x05, 0b0101),
        (0x65, 0b0110, 0x5f, 0b0100),
        (0x65, 0b0111, 0xff, 0b0101),
        (0x65, 0b1000, 0x65, 0b0000),
        (0x65, 0b1001, 0xc5, 0b0001),
        (0x65, 0b1010, 0x6b, 0b0000),
        (0x65, 0b1011, 0xcb, 0b0001),
        (0x65, 0b1100, 0x65, 0b0100),
        (0x65, 0b1101, 0x05, 0b0101),
        (0x65, 0b1110, 0x5f, 0b0100),
        (0x65, 0b1111, 0xff, 0b0101),
        (0x66, 0b0000, 0x66, 0b0000),
        (0x66, 0b0001, 0xc6, 0b0001),
        (0x66, 0b0010, 0x6c, 0b0000),
        (0x66, 0b0011, 0xcc, 0b0001),
        (0x66, 0b0100, 0x66, 0b0100),
        (0x66, 0b0101, 0x06, 0b0101),
        (0x66, 0b0110, 0x60, 0b0100),
        (0x66, 0b0111, 0x00, 0b1101),
        (0x66, 0b1000, 0x66, 0b0000),
        (0x66, 0b1001, 0xc6, 0b0001),
        (0x66, 0b1010, 0x6c, 0b0000),
        (0x66, 0b1011, 0xcc, 0b0001),
        (0x66, 0b1100, 0x66, 0b0100),
        (0x66, 0b1101, 0x06, 0b0101),
        (0x66, 0b1110, 0x60, 0b0100),
        (0x66, 0b1111, 0x00, 0b1101),
        (0x67, 0b0000, 0x67, 0b0000),
        (0x67, 0b0001, 0xc7, 0b0001),
        (0x67, 0b0010, 0x6d, 0b0000),
        (0x67, 0b0011, 0xcd, 0b0001),
        (0x67, 0b0100, 0x67, 0b0100),
        (0x67, 0b0101, 0x07, 0b0101),
        (0x67, 0b0110, 0x61, 0b0100),
        (0x67, 0b0111, 0x01, 0b0101),
        (0x67, 0b1000, 0x67, 0b0000),
        (0x67, 0b1001, 0xc7, 0b0001),
        (0x67, 0b1010, 0x6d, 0b0000),
        (0x67, 0b1011, 0xcd, 0b0001),
        (0x67, 0b1100, 0x67, 0b0100),
        (0x67, 0b1101, 0x07, 0b0101),
        (0x67, 0b1110, 0x61, 0b0100),
        (0x67, 0b1111, 0x01, 0b0101),
        (0x68, 0b0000, 0x68, 0b0000),
        (0x68, 0b0001, 0xc8, 0b0001),
        (0x68, 0b0010, 0x6e, 0b0000),
        (0x68, 0b0011, 0xce, 0b0001),
        (0x68, 0b0100, 0x68, 0b0100),
        (0x68, 0b0101, 0x08, 0b0101),
        (0x68, 0b0110, 0x62, 0b0100),
        (0x68, 0b0111, 0x02, 0b0101),
        (0x68, 0b1000, 0x68, 0b0000),
        (0x68, 0b1001, 0xc8, 0b0001),
        (0x68, 0b1010, 0x6e, 0b0000),
        (0x68, 0b1011, 0xce, 0b0001),
        (0x68, 0b1100, 0x68, 0b0100),
        (0x68, 0b1101, 0x08, 0b0101),
        (0x68, 0b1110, 0x62, 0b0100),
        (0x68, 0b1111, 0x02, 0b0101),
        (0x69, 0b0000, 0x69, 0b0000),
        (0x69, 0b0001, 0xc9, 0b0001),
        (0x69, 0b0010, 0x6f, 0b0000),
        (0x69, 0b0011, 0xcf, 0b0001),
        (0x69, 0b0100, 0x69, 0b0100),
        (0x69, 0b0101, 0x09, 0b0101),
        (0x69, 0b0110, 0x63, 0b0100),
        (0x69, 0b0111, 0x03, 0b0101),
        (0x69, 0b1000, 0x69, 0b0000),
        (0x69, 0b1001, 0xc9, 0b0001),
        (0x69, 0b1010, 0x6f, 0b0000),
        (0x69, 0b1011, 0xcf, 0b0001),
        (0x69, 0b1100, 0x69, 0b0100),
        (0x69, 0b1101, 0x09, 0b0101),
        (0x69, 0b1110, 0x63, 0b0100),
        (0x69, 0b1111, 0x03, 0b0101),
        (0x6a, 0b0000, 0x70, 0b0000),
        (0x6a, 0b0001, 0xd0, 0b0001),
        (0x6a, 0b0010, 0x70, 0b0000),
        (0x6a, 0b0011, 0xd0, 0b0001),
        (0x6a, 0b0100, 0x6a, 0b0100),
        (0x6a, 0b0101, 0x0a, 0b0101),
        (0x6a, 0b0110, 0x64, 0b0100),
        (0x6a, 0b0111, 0x04, 0b0101),
        (0x6a, 0b1000, 0x70, 0b0000),
        (0x6a, 0b1001, 0xd0, 0b0001),
        (0x6a, 0b1010, 0x70, 0b0000),
        (0x6a, 0b1011, 0xd0, 0b0001),
        (0x6a, 0b1100, 0x6a, 0b0100),
        (0x6a, 0b1101, 0x0a, 0b0101),
        (0x6a, 0b1110, 0x64, 0b0100),
        (0x6a, 0b1111, 0x04, 0b0101),
        (0x6b, 0b0000, 0x71, 0b0000),
        (0x6b, 0b0001, 0xd1, 0b0001),
        (0x6b, 0b0010, 0x71, 0b0000),
        (0x6b, 0b0011, 0xd1, 0b0001),
        (0x6b, 0b0100, 0x6b, 0b0100),
        (0x6b, 0b0101, 0x0b, 0b0101),
        (0x6b, 0b0110, 0x65, 0b0100),
        (0x6b, 0b0111, 0x05, 0b0101),
        (0x6b, 0b1000, 0x71, 0b0000),
        (0x6b, 0b1001, 0xd1, 0b0001),
        (0x6b, 0b1010, 0x71, 0b0000),
        (0x6b, 0b1011, 0xd1, 0b0001),
        (0x6b, 0b1100, 0x6b, 0b0100),
        (0x6b, 0b1101, 0x0b, 0b0101),
        (0x6b, 0b1110, 0x65, 0b0100),
        (0x6b, 0b1111, 0x05, 0b0101),
        (0x6c, 0b0000, 0x72, 0b0000),
        (0x6c, 0b0001, 0xd2, 0b0001),
        (0x6c, 0b0010, 0x72, 0b0000),
        (0x6c, 0b0011, 0xd2, 0b0001),
        (0x6c, 0b0100, 0x6c, 0b0100),
        (0x6c, 0b0101, 0x0c, 0b0101),
        (0x6c, 0b0110, 0x66, 0b0100),
        (0x6c, 0b0111, 0x06, 0b0101),
        (0x6c, 0b1000, 0x72, 0b0000),
        (0x6c, 0b1001, 0xd2, 0b0001),
        (0x6c, 0b1010, 0x72, 0b0000),
        (0x6c, 0b1011, 0xd2, 0b0001),
        (0x6c, 0b1100, 0x6c, 0b0100),
        (0x6c, 0b1101, 0x0c, 0b0101),
        (0x6c, 0b1110, 0x66, 0b0100),
        (0x6c, 0b1111, 0x06, 0b0101),
        (0x6d, 0b0000, 0x73, 0b0000),
        (0x6d, 0b0001, 0xd3, 0b0001),
        (0x6d, 0b0010, 0x73, 0b0000),
        (0x6d, 0b0011, 0xd3, 0b0001),
        (0x6d, 0b0100, 0x6d, 0b0100),
        (0x6d, 0b0101, 0x0d, 0b0101),
        (0x6d, 0b0110, 0x67, 0b0100),
        (0x6d, 0b0111, 0x07, 0b0101),
        (0x6d, 0b1000, 0x73, 0b0000),
        (0x6d, 0b1001, 0xd3, 0b0001),
        (0x6d, 0b1010, 0x73, 0b0000),
        (0x6d, 0b1011, 0xd3, 0b0001),
        (0x6d, 0b1100, 0x6d, 0b0100),
        (0x6d, 0b1101, 0x0d, 0b0101),
        (0x6d, 0b1110, 0x67, 0b0100),
        (0x6d, 0b1111, 0x07, 0b0101),
        (0x6e, 0b0000, 0x74, 0b0000),
        (0x6e, 0b0001, 0xd4, 0b0001),
        (0x6e, 0b0010, 0x74, 0b0000),
        (0x6e, 0b0011, 0xd4, 0b0001),
        (0x6e, 0b0100, 0x6e, 0b0100),
        (0x6e, 0b0101, 0x0e, 0b0101),
        (0x6e, 0b0110, 0x68, 0b0100),
        (0x6e, 0b0111, 0x08, 0b0101),
        (0x6e, 0b1000, 0x74, 0b0000),
        (0x6e, 0b1001, 0xd4, 0b0001),
        (0x6e, 0b1010, 0x74, 0b0000),
        (0x6e, 0b1011, 0xd4, 0b0001),
        (0x6e, 0b1100, 0x6e, 0b0100),
        (0x6e, 0b1101, 0x0e, 0b0101),
        (0x6e, 0b1110, 0x68, 0b0100),
        (0x6e, 0b1111, 0x08, 0b0101),
        (0x6f, 0b0000, 0x75, 0b0000),
        (0x6f, 0b0001, 0xd5, 0b0001),
        (0x6f, 0b0010, 0x75, 0b0000),
        (0x6f, 0b0011, 0xd5, 0b0001),
        (0x6f, 0b0100, 0x6f, 0b0100),
        (0x6f, 0b0101, 0x0f, 0b0101),
        (0x6f, 0b0110, 0x69, 0b0100),
        (0x6f, 0b0111, 0x09, 0b0101),
        (0x6f, 0b1000, 0x75, 0b0000),
        (0x6f, 0b1001, 0xd5, 0b0001),
        (0x6f, 0b1010, 0x75, 0b0000),
        (0x6f, 0b1011, 0xd5, 0b0001),
        (0x6f, 0b1100, 0x6f, 0b0100),
        (0x6f, 0b1101, 0x0f, 0b0101),
        (0x6f, 0b1110, 0x69, 0b0100),
        (0x6f, 0b1111, 0x09, 0b0101),
        (0x70, 0b0000, 0x70, 0b0000),
        (0x70, 0b0001, 0xd0, 0b0001),
        (0x70, 0b0010, 0x76, 0b0000),
        (0x70, 0b0011, 0xd6, 0b0001),
        (0x70, 0b0100, 0x70, 0b0100),
        (0x70, 0b0101, 0x10, 0b0101),
        (0x70, 0b0110, 0x6a, 0b0100),
        (0x70, 0b0111, 0x0a, 0b0101),
        (0x70, 0b1000, 0x70, 0b0000),
        (0x70, 0b1001, 0xd0, 0b0001),
        (0x70, 0b1010, 0x76, 0b0000),
        (0x70, 0b1011, 0xd6, 0b0001),
        (0x70, 0b1100, 0x70, 0b0100),
        (0x70, 0b1101, 0x10, 0b0101),
        (0x70, 0b1110, 0x6a, 0b0100),
        (0x70, 0b1111, 0x0a, 0b0101),
        (0x71, 0b0000, 0x71, 0b0000),
        (0x71, 0b0001, 0xd1, 0b0001),
        (0x71, 0b0010, 0x77, 0b0000),
        (0x71, 0b0011, 0xd7, 0b0001),
        (0x71, 0b0100, 0x71, 0b0100),
        (0x71, 0b0101, 0x11, 0b0101),
        (0x71, 0b0110, 0x6b, 0b0100),
        (0x71, 0b0111, 0x0b, 0b0101),
        (0x71, 0b1000, 0x71, 0b0000),
        (0x71, 0b1001, 0xd1, 0b0001),
        (0x71, 0b1010, 0x77, 0b0000),
        (0x71, 0b1011, 0xd7, 0b0001),
        (0x71, 0b1100, 0x71, 0b0100),
        (0x71, 0b1101, 0x11, 0b0101),
        (0x71, 0b1110, 0x6b, 0b0100),
        (0x71, 0b1111, 0x0b, 0b0101),
        (0x72, 0b0000, 0x72, 0b0000),
        (0x72, 0b0001, 0xd2, 0b0001),
        (0x72, 0b0010, 0x78, 0b0000),
        (0x72, 0b0011, 0xd8, 0b0001),
        (0x72, 0b0100, 0x72, 0b0100),
        (0x72, 0b0101, 0x12, 0b0101),
        (0x72, 0b0110, 0x6c, 0b0100),
        (0x72, 0b0111, 0x0c, 0b0101),
        (0x72, 0b1000, 0x72, 0b0000),
        (0x72, 0b1001, 0xd2, 0b0001),
        (0x72, 0b1010, 0x78, 0b0000),
        (0x72, 0b1011, 0xd8, 0b0001),
        (0x72, 0b1100, 0x72, 0b0100),
        (0x72, 0b1101, 0x12, 0b0101),
        (0x72, 0b1110, 0x6c, 0b0100),
        (0x72, 0b1111, 0x0c, 0b0101),
        (0x73, 0b0000, 0x73, 0b0000),
        (0x73, 0b0001, 0xd3, 0b0001),
        (0x73, 0b0010, 0x79, 0b0000),
        (0x73, 0b0011, 0xd9, 0b0001),
        (0x73, 0b0100, 0x73, 0b0100),
        (0x73, 0b0101, 0x13, 0b0101),
        (0x73, 0b0110, 0x6d, 0b0100),
        (0x73, 0b0111, 0x0d, 0b0101),
        (0x73, 0b1000, 0x73, 0b0000),
        (0x73, 0b1001, 0xd3, 0b0001),
        (0x73, 0b1010, 0x79, 0b0000),
        (0x73, 0b1011, 0xd9, 0b0001),
        (0x73, 0b1100, 0x73, 0b0100),
        (0x73, 0b1101, 0x13, 0b0101),
        (0x73, 0b1110, 0x6d, 0b0100),
        (0x73, 0b1111, 0x0d, 0b0101),
        (0x74, 0b0000, 0x74, 0b0000),
        (0x74, 0b0001, 0xd4, 0b0001),
        (0x74, 0b0010, 0x7a, 0b0000),
        (0x74, 0b0011, 0xda, 0b0001),
        (0x74, 0b0100, 0x74, 0b0100),
        (0x74, 0b0101, 0x14, 0b0101),
        (0x74, 0b0110, 0x6e, 0b0100),
        (0x74, 0b0111, 0x0e, 0b0101),
        (0x74, 0b1000, 0x74, 0b0000),
        (0x74, 0b1001, 0xd4, 0b0001),
        (0x74, 0b1010, 0x7a, 0b0000),
        (0x74, 0b1011, 0xda, 0b0001),
        (0x74, 0b1100, 0x74, 0b0100),
        (0x74, 0b1101, 0x14, 0b0101),
        (0x74, 0b1110, 0x6e, 0b0100),
        (0x74, 0b1111, 0x0e, 0b0101),
        (0x75, 0b0000, 0x75, 0b0000),
        (0x75, 0b0001, 0xd5, 0b0001),
        (0x75, 0b0010, 0x7b, 0b0000),
        (0x75, 0b0011, 0xdb, 0b0001),
        (0x75, 0b0100, 0x75, 0b0100),
        (0x75, 0b0101, 0x15, 0b0101),
        (0x75, 0b0110, 0x6f, 0b0100),
        (0x75, 0b0111, 0x0f, 0b0101),
        (0x75, 0b1000, 0x75, 0b0000),
        (0x75, 0b1001, 0xd5, 0b0001),
        (0x75, 0b1010, 0x7b, 0b0000),
        (0x75, 0b1011, 0xdb, 0b0001),
        (0x75, 0b1100, 0x75, 0b0100),
        (0x75, 0b1101, 0x15, 0b0101),
        (0x75, 0b1110, 0x6f, 0b0100),
        (0x75, 0b1111, 0x0f, 0b0101),
        (0x76, 0b0000, 0x76, 0b0000),
        (0x76, 0b0001, 0xd6, 0b0001),
        (0x76, 0b0010, 0x7c, 0b0000),
        (0x76, 0b0011, 0xdc, 0b0001),
        (0x76, 0b0100, 0x76, 0b0100),
        (0x76, 0b0101, 0x16, 0b0101),
        (0x76, 0b0110, 0x70, 0b0100),
        (0x76, 0b0111, 0x10, 0b0101),
        (0x76, 0b1000, 0x76, 0b0000),
        (0x76, 0b1001, 0xd6, 0b0001),
        (0x76, 0b1010, 0x7c, 0b0000),
        (0x76, 0b1011, 0xdc, 0b0001),
        (0x76, 0b1100, 0x76, 0b0100),
        (0x76, 0b1101, 0x16, 0b0101),
        (0x76, 0b1110, 0x70, 0b0100),
        (0x76, 0b1111, 0x10, 0b0101),
        (0x77, 0b0000, 0x77, 0b0000),
        (0x77, 0b0001, 0xd7, 0b0001),
        (0x77, 0b0010, 0x7d, 0b0000),
        (0x77, 0b0011, 0xdd, 0b0001),
        (0x77, 0b0100, 0x77, 0b0100),
        (0x77, 0b0101, 0x17, 0b0101),
        (0x77, 0b0110, 0x71, 0b0100),
        (0x77, 0b0111, 0x11, 0b0101),
        (0x77, 0b1000, 0x77, 0b0000),
        (0x77, 0b1001, 0xd7, 0b0001),
        (0x77, 0b1010, 0x7d, 0b0000),
        (0x77, 0b1011, 0xdd, 0b0001),
        (0x77, 0b1100, 0x77, 0b0100),
        (0x77, 0b1101, 0x17, 0b0101),
        (0x77, 0b1110, 0x71, 0b0100),
        (0x77, 0b1111, 0x11, 0b0101),
        (0x78, 0b0000, 0x78, 0b0000),
        (0x78, 0b0001, 0xd8, 0b0001),
        (0x78, 0b0010, 0x7e, 0b0000),
        (0x78, 0b0011, 0xde, 0b0001),
        (0x78, 0b0100, 0x78, 0b0100),
        (0x78, 0b0101, 0x18, 0b0101),
        (0x78, 0b0110, 0x72, 0b0100),
        (0x78, 0b0111, 0x12, 0b0101),
        (0x78, 0b1000, 0x78, 0b0000),
        (0x78, 0b1001, 0xd8, 0b0001),
        (0x78, 0b1010, 0x7e, 0b0000),
        (0x78, 0b1011, 0xde, 0b0001),
        (0x78, 0b1100, 0x78, 0b0100),
        (0x78, 0b1101, 0x18, 0b0101),
        (0x78, 0b1110, 0x72, 0b0100),
        (0x78, 0b1111, 0x12, 0b0101),
        (0x79, 0b0000, 0x79, 0b0000),
        (0x79, 0b0001, 0xd9, 0b0001),
        (0x79, 0b0010, 0x7f, 0b0000),
        (0x79, 0b0011, 0xdf, 0b0001),
        (0x79, 0b0100, 0x79, 0b0100),
        (0x79, 0b0101, 0x19, 0b0101),
        (0x79, 0b0110, 0x73, 0b0100),
        (0x79, 0b0111, 0x13, 0b0101),
        (0x79, 0b1000, 0x79, 0b0000),
        (0x79, 0b1001, 0xd9, 0b0001),
        (0x79, 0b1010, 0x7f, 0b0000),
        (0x79, 0b1011, 0xdf, 0b0001),
        (0x79, 0b1100, 0x79, 0b0100),
        (0x79, 0b1101, 0x19, 0b0101),
        (0x79, 0b1110, 0x73, 0b0100),
        (0x79, 0b1111, 0x13, 0b0101),
        (0x7a, 0b0000, 0x80, 0b0000),
        (0x7a, 0b0001, 0xe0, 0b0001),
        (0x7a, 0b0010, 0x80, 0b0000),
        (0x7a, 0b0011, 0xe0, 0b0001),
        (0x7a, 0b0100, 0x7a, 0b0100),
        (0x7a, 0b0101, 0x1a, 0b0101),
        (0x7a, 0b0110, 0x74, 0b0100),
        (0x7a, 0b0111, 0x14, 0b0101),
        (0x7a, 0b1000, 0x80, 0b0000),
        (0x7a, 0b1001, 0xe0, 0b0001),
        (0x7a, 0b1010, 0x80, 0b0000),
        (0x7a, 0b1011, 0xe0, 0b0001),
        (0x7a, 0b1100, 0x7a, 0b0100),
        (0x7a, 0b1101, 0x1a, 0b0101),
        (0x7a, 0b1110, 0x74, 0b0100),
        (0x7a, 0b1111, 0x14, 0b0101),
        (0x7b, 0b0000, 0x81, 0b0000),
        (0x7b, 0b0001, 0xe1, 0b0001),
        (0x7b, 0b0010, 0x81, 0b0000),
        (0x7b, 0b0011, 0xe1, 0b0001),
        (0x7b, 0b0100, 0x7b, 0b0100),
        (0x7b, 0b0101, 0x1b, 0b0101),
        (0x7b, 0b0110, 0x75, 0b0100),
        (0x7b, 0b0111, 0x15, 0b0101),
        (0x7b, 0b1000, 0x81, 0b0000),
        (0x7b, 0b1001, 0xe1, 0b0001),
        (0x7b, 0b1010, 0x81, 0b0000),
        (0x7b, 0b1011, 0xe1, 0b0001),
        (0x7b, 0b1100, 0x7b, 0b0100),
        (0x7b, 0b1101, 0x1b, 0b0101),
        (0x7b, 0b1110, 0x75, 0b0100),
        (0x7b, 0b1111, 0x15, 0b0101),
        (0x7c, 0b0000, 0x82, 0b0000),
        (0x7c, 0b0001, 0xe2, 0b0001),
        (0x7c, 0b0010, 0x82, 0b0000),
        (0x7c, 0b0011, 0xe2, 0b0001),
        (0x7c, 0b0100, 0x7c, 0b0100),
        (0x7c, 0b0101, 0x1c, 0b0101),
        (0x7c, 0b0110, 0x76, 0b0100),
        (0x7c, 0b0111, 0x16, 0b0101),
        (0x7c, 0b1000, 0x82, 0b0000),
        (0x7c, 0b1001, 0xe2, 0b0001),
        (0x7c, 0b1010, 0x82, 0b0000),
        (0x7c, 0b1011, 0xe2, 0b0001),
        (0x7c, 0b1100, 0x7c, 0b0100),
        (0x7c, 0b1101, 0x1c, 0b0101),
        (0x7c, 0b1110, 0x76, 0b0100),
        (0x7c, 0b1111, 0x16, 0b0101),
        (0x7d, 0b0000, 0x83, 0b0000),
        (0x7d, 0b0001, 0xe3, 0b0001),
        (0x7d, 0b0010, 0x83, 0b0000),
        (0x7d, 0b0011, 0xe3, 0b0001),
        (0x7d, 0b0100, 0x7d, 0b0100),
        (0x7d, 0b0101, 0x1d, 0b0101),
        (0x7d, 0b0110, 0x77, 0b0100),
        (0x7d, 0b0111, 0x17, 0b0101),
        (0x7d, 0b1000, 0x83, 0b0000),
        (0x7d, 0b1001, 0xe3, 0b0001),
        (0x7d, 0b1010, 0x83, 0b0000),
        (0x7d, 0b1011, 0xe3, 0b0001),
        (0x7d, 0b1100, 0x7d, 0b0100),
        (0x7d, 0b1101, 0x1d, 0b0101),
        (0x7d, 0b1110, 0x77, 0b0100),
        (0x7d, 0b1111, 0x17, 0b0101),
        (0x7e, 0b0000, 0x84, 0b0000),
        (0x7e, 0b0001, 0xe4, 0b0001),
        (0x7e, 0b0010, 0x84, 0b0000),
        (0x7e, 0b0011, 0xe4, 0b0001),
        (0x7e, 0b0100, 0x7e, 0b0100),
        (0x7e, 0b0101, 0x1e, 0b0101),
        (0x7e, 0b0110, 0x78, 0b0100),
        (0x7e, 0b0111, 0x18, 0b0101),
        (0x7e, 0b1000, 0x84, 0b0000),
        (0x7e, 0b1001, 0xe4, 0b0001),
        (0x7e, 0b1010, 0x84, 0b0000),
        (0x7e, 0b1011, 0xe4, 0b0001),
        (0x7e, 0b1100, 0x7e, 0b0100),
        (0x7e, 0b1101, 0x1e, 0b0101),
        (0x7e, 0b1110, 0x78, 0b0100),
        (0x7e, 0b1111, 0x18, 0b0101),
        (0x7f, 0b0000, 0x85, 0b0000),
        (0x7f, 0b0001, 0xe5, 0b0001),
        (0x7f, 0b0010, 0x85, 0b0000),
        (0x7f, 0b0011, 0xe5, 0b0001),
        (0x7f, 0b0100, 0x7f, 0b0100),
        (0x7f, 0b0101, 0x1f, 0b0101),
        (0x7f, 0b0110, 0x79, 0b0100),
        (0x7f, 0b0111, 0x19, 0b0101),
        (0x7f, 0b1000, 0x85, 0b0000),
        (0x7f, 0b1001, 0xe5, 0b0001),
        (0x7f, 0b1010, 0x85, 0b0000),
        (0x7f, 0b1011, 0xe5, 0b0001),
        (0x7f, 0b1100, 0x7f, 0b0100),
        (0x7f, 0b1101, 0x1f, 0b0101),
        (0x7f, 0b1110, 0x79, 0b0100),
        (0x7f, 0b1111, 0x19, 0b0101),
    ];
}