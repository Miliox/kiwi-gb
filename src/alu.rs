use super::flags::Flags;

/// Add arg to acc
///
/// Flags Affected:
/// - Z: Set if result is zero
/// - N: Reset
/// - H: Set if carry from bit 3
/// - C: Set if carry from bit 7
pub fn add(mut flags: Flags, acc: u8, arg: u8) -> (Flags, u8) {
    let (_, half) = acc.wrapping_shl(4).overflowing_add(arg.wrapping_shl(4));
    let (acc, carry) = acc.overflowing_add(arg);

    flags.set_zero_if(acc == 0);
    flags.reset_sub();
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
pub fn sub(mut flags: Flags, acc: u8, arg: u8) -> (Flags, u8) {
    let (_, half) = acc.wrapping_shr(4).overflowing_sub(arg.wrapping_shr(4));
    let (acc, carry) = acc.overflowing_sub(arg);

    flags.set_zero_if(acc == 0);
    flags.set_sub();
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
        return add(flags, acc, arg)
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
        return sub(flags, acc, arg);
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
pub fn and(mut flags: Flags, mut acc: u8, arg: u8) -> (Flags, u8) {
    acc = acc & arg;

    flags.set_zero_if(acc == 0);
    flags.reset_sub();
    flags.set_half();
    flags.reset_carry();

    (flags, acc)
}

/// Logical OR of acc with arg
///
/// Flags Affected:
/// - Z: Set if result is zero
/// - N: Reset
/// - H: Reset
/// - C: Reset
pub fn or(mut flags: Flags, mut acc: u8, arg: u8) -> (Flags, u8) {
    acc = acc | arg;

    flags.set_zero_if(acc == 0);
    flags.reset_sub();
    flags.reset_half();
    flags.reset_carry();

    (flags, acc)
}

/// Logical eXclusive OR (XOR) of acc with arg
///
/// Flags Affected:
/// - Z: Set if result is zero
/// - N: Reset
/// - H: Reset
/// - C: Reset
pub fn xor(mut flags: Flags, mut acc: u8, arg: u8) -> (Flags, u8) {
    acc = acc ^ arg;

    flags.set_zero_if(acc == 0);
    flags.reset_sub();
    flags.reset_half();
    flags.reset_carry();

    (flags, acc)
}

/// Rotates acc to the left with bit 7 being moved to bit 0 and also stored into the carry.
///
/// Flags Affected:
/// - Z - Set if result is zero
/// - N - Reset
/// - H - Reset
/// - C - Contains old bit 7
pub fn rlc(mut flags: Flags, mut acc: u8) -> (Flags, u8) {
    acc = acc.rotate_left(1);

    flags.set_zero_if(acc == 0);
    flags.reset_sub();
    flags.reset_half();
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
pub fn rrc(mut flags: Flags, mut acc: u8) -> (Flags, u8) {
    acc = acc.rotate_right(1);

    flags.set_zero_if(acc == 0);
    flags.reset_sub();
    flags.reset_half();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_exaustive_test() {
        for flags in 0u8..15u8 {
            for acc in 0u8..255u8 {
                for arg in 0u8..255u8 {
                    let half = ((acc & 0xf) + (arg & 0xf)) > 0xf;
                    let (expected, carry) = acc.overflowing_add(arg);

                    let (flags, acc) = add(Flags::from(flags << 4), acc, arg);
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
        for flags in 0u8..15u8 {
            for acc in 0u8..255u8 {
                for arg in 0u8..255u8 {
                    let half = (acc & 0xf0) < (arg & 0xf0);
                    let (expected, carry) = acc.overflowing_sub(arg);

                    let (flags, acc) = sub(Flags::from(flags << 4), acc, arg);

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
        for flags in 0u8..15u8 {
            for acc in 0u8..255u8 {
                for arg in 0u8..255u8 {
                    let flags = Flags::from(flags << 4);
                    let expected = acc & arg;

                    let (flags, acc) = and(flags, acc, arg);

                    assert_eq!(expected, acc);
                    assert_eq!(acc == 0, flags.zero());
                    assert_eq!(false, flags.sub());
                    assert_eq!(true, flags.half());
                    assert_eq!(false, flags.carry());
                }
            }
        }
    }

    #[test]
    fn or_exaustive_test() {
        for flags in 0u8..15u8 {
            for acc in 0u8..255u8 {
                for arg in 0u8..255u8 {
                    let flags = Flags::from(flags << 4);
                    let expected = acc | arg;

                    let (flags, acc) = or(flags, acc, arg);

                    assert_eq!(expected, acc);
                    assert_eq!(acc == 0, flags.zero());
                    assert_eq!(false, flags.sub());
                    assert_eq!(false, flags.half());
                    assert_eq!(false, flags.carry());
                }
            }
        }
    }

    #[test]
    fn xor_exaustive_test() {
        for flags in 0u8..15u8 {
            for acc in 0u8..255u8 {
                for arg in 0u8..255u8 {
                    let flags = Flags::from(flags << 4);
                    let expected = acc ^ arg;

                    let (flags, acc) = xor(flags, acc, arg);

                    assert_eq!(expected, acc);
                    assert_eq!(acc == 0, flags.zero());
                    assert_eq!(false, flags.sub());
                    assert_eq!(false, flags.half());
                    assert_eq!(false, flags.carry());
                }
            }
        }
    }

    #[test]
    fn rlc_exaustive_test() {
        for flags in 0u8..15u8 {
            for acc in 0u8..255u8 {
                let flags = Flags::from(flags << 4);
                let carry = (acc & 0x80) != 0;
                let expected = acc.rotate_left(1);

                let (flags, acc) = rlc(flags, acc);

                assert_eq!(expected, acc);
                assert_eq!(acc == 0, flags.zero());
                assert_eq!(false, flags.sub());
                assert_eq!(false, flags.half());
                assert_eq!(carry, flags.carry());
            }
        }
    }

    #[test]
    fn rrc_exaustive_test() {
        for flags in 0u8..15u8 {
            for acc in 0u8..255u8 {
                let flags = Flags::from(flags << 4);
                let carry = (acc & 0x01) != 0;
                let expected = acc.rotate_right(1);

                let (flags, acc) = rrc(flags, acc);

                assert_eq!(expected, acc);
                assert_eq!(acc == 0, flags.zero());
                assert_eq!(false, flags.sub());
                assert_eq!(false, flags.half());
                assert_eq!(carry, flags.carry());
            }
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
}