
pub fn set_low_frequency_param(fparam: u32, low: u32) -> u32 {
    (fparam & 0x700) | (low & 0x0FF)
}

pub fn set_high_frequency_param(fparam: u32, high: u32) -> u32 {
    ((high & 0x7) << 8) | fparam & 0xFF
}

pub fn calculate_frequency(f: u32) -> u32 {
    131_072 / (2048 - f)
}

pub fn calculate_phase_duty(d: u8) -> f32 {
    match d {
        0 => 0.125,
        1 => 0.25,
        2 => 0.5,
        3 => 0.75,
        _ => 0.5
    }
}

pub fn calculate_volume(v: u8) -> i8 {
    let v: f32 = v as f32;
    let coef: f32 = 1.0 / 15.0;
    let maxv: f32 = 127.0;
    (v * coef * maxv) as i8
}

pub fn calculate_sample(s: u8) -> i8 {
    /*
    F |               **
    E |              *  *
    D |             *    *
    C |            *      *
    B |           *        *
    A |          *          *
    9 |         *            *
    8 |        *              *
    7 |       *                *
    6 |      *                  *
    5 |     *                    *
    4 |    *                      *
    3 |   *                        *
    2 |  *                          *
    1 | *                            *
    0 |*                              *
    *  --------------------------------
    */
    match s {
        0 => -127,
        1 => -109,
        2 => -90,
        3 => -72,
        4 => -54,
        5 => -36,
        6 => -18,
        7 => 0,
        8 => 0,
        9 => 18,
        10 => 36,
        11 => 54,
        12 => 72,
        13 => 90,
        14 => 109,
        15 => 127,
        _ => panic!(),
    }
}
