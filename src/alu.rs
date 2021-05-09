use super::flags::Flags;

/// Add arg to acc
///
/// Flags Affected:
/// - Z: Set if result is zero
/// - N: Reset
/// - H: Set if carry from bit 3
/// - C: Set if carry from bit 7
pub fn add(acc: u8, flags: u8, arg: u8) -> (u8, u8) {
    let (_, half) = acc.wrapping_shl(4).overflowing_add(arg.wrapping_shl(4));
    let (acc, carry) = acc.overflowing_add(arg);

    let mut flags = Flags::from(flags);
    flags.set_zero_if(acc == 0);
    flags.reset_sub();
    flags.set_half_if(half);
    flags.set_carry_if(carry);

    (acc, flags.into())
}