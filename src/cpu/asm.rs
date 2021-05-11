#[allow(dead_code)]
pub fn disassemble(opcode: u8, immediate8: u8, immediate16: u16) -> String {
    let opcode = opcode as usize;
    match opcode {
        0xCB => INST_ASM_CB[immediate8 as usize].to_string(),
        _ => match INST_SIZE[opcode as usize] {
            1 => INST_ASM[opcode].to_string(),
            2 => INST_ASM[opcode].replace("$00", &format!("${:02X}", immediate8)),
            3 => INST_ASM[opcode].replace("$0000", &format!("${:04X}", immediate16)),
            _ => panic!("Invalid Instruction {:02x}", opcode),
        }
    }
}

pub fn instruction_size(opcode: u8) -> u16 {
    INST_SIZE[opcode as usize] as u16
}

pub fn instruction_ticks(opcode: u8) -> u64 {
    INST_TICKS[opcode as usize] as u64
}

#[allow(dead_code)]
/// Instruction Size
pub const INST_SIZE: [u8; 256] = [
//  0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
    1, 3, 1, 1, 1, 1, 2, 1, 3, 1, 1, 1, 1, 1, 2, 1,    // 0x00 ~ 0x0F
    2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1,    // 0x10 ~ 0x1F
    2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1,    // 0x20 ~ 0x2F
    2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1,    // 0x30 ~ 0x3F

    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0x40 ~ 0x4F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0x50 ~ 0x5F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0x60 ~ 0x6F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0x70 ~ 0x7F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0x80 ~ 0x8F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0x90 ~ 0x9F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0xA0 ~ 0xAF
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,    // 0xB0 ~ 0xBF

    1, 1, 3, 3, 3, 1, 2, 1, 1, 1, 3, 2, 3, 3, 2, 1,    // 0xC0 ~ 0xCF
    1, 1, 3, 1, 3, 1, 2, 1, 1, 1, 3, 1, 3, 1, 2, 1,    // 0xD0 ~ 0xDF
    2, 1, 1, 1, 1, 1, 2, 1, 2, 1, 3, 1, 1, 1, 2, 1,    // 0xE0 ~ 0xEF
    2, 1, 2, 1, 1, 1, 2, 1, 2, 1, 3, 1, 1, 1, 2, 1,    // 0xF0 ~ 0xFF
];

#[allow(dead_code)]
/// Instruction Ticks
pub const INST_TICKS: [u8; 256] = [
//  x0  x1  x2  x3  x4  x5  x6  x7  x8  x9  xA  xB  xC  xD  xE  xF
    4,  12, 8,  8,  4,  4,  8,  4,  20, 8,  8,  8,  4,  4,  8,  4,    // 0x00 ~ 0x0F
    4,  12, 8,  8,  4,  4,  8,  4,  12, 8,  8,  8,  4,  4,  8,  4,    // 0x10 ~ 0x1F
    8,  12, 8,  8,  4,  4,  8,  4,  8,  8,  8,  8,  4,  4,  8,  4,    // 0x20 ~ 0x2F
    8,  12, 8,  8, 12, 12, 12,  4,  8,  8,  8,  8,  4,  4,  8,  4,    // 0x30 ~ 0x3F

    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,    // 0x40 ~ 0x4F
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,    // 0x50 ~ 0x5F
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,    // 0x60 ~ 0x6F
    8,  8,  8,  8,  8,  8,  4,  8,  4,  4,  4,  4,  4,  4,  8,  4,    // 0x70 ~ 0x7F

    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,    // 0x80 ~ 0x8F
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,    // 0x90 ~ 0x9F
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,    // 0xA0 ~ 0xAF
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,    // 0xB0 ~ 0xBF

    8,  12, 12, 16, 12, 16, 8,  16, 8,  16, 12, 8,  12, 24, 8,  16,    // 0xC0 ~ 0xCF
    8,  12, 12, 4,  12, 16, 8,  16, 8,  16, 12, 4,  12, 4,  8,  16,    // 0xD0 ~ 0xDF
    12, 12, 8,  4,  4,  16, 8,  16, 16, 4,  16, 4,  4,  4,  8,  16,    // 0xE0 ~ 0xEF
    12, 12, 8,  4,  4,  16, 8,  16, 12, 8,  16, 4,  4,  4,  8,  16,    // 0xF0 ~ 0xFF
];

/// Instruction Assembly
#[allow(dead_code)]
pub const INST_ASM: [&str; 256] = [
    // 0x00 ~ 0x0F
    "NOP",
    "LD BC, $0000",
    "LD (BC), A",
    "INC BC",
    "INC B",
    "DEC B",
    "LD B, $00",
    "RLCA",
    "LD ($0000),SP",
    "ADD HL, BC",
    "LD A, (BC)",
    "DEC BC",
    "INC C",
    "DEC C",
    "LD C, $00",
    "RRCA",

    // 0x10 ~ 0x1F
    "STOP 0",
    "LD DE, $0000",
    "LD (DE), A",
    "INC DE",
    "INC D",
    "DEC D",
    "LD D, $00",
    "RLA",
    "JR $00",
    "ADD HL, DE",
    "LD A, (DE)",
    "DEC DE",
    "INC E",
    "DEC E",
    "LD E, $00",
    "RRA",

    // 0x20 ~ 0x2F
    "JR NZ $00",
    "LD HL, $0000",
    "LDI (HL), A",
    "INC HL",
    "INC H",
    "DEC H",
    "LD H, $00",
    "DAA",
    "JR Z $00",
    "ADD HL, HL",
    "LDI A, (HL)",
    "DEC HL",
    "INC L",
    "DEC L",
    "LD L, $00",
    "CPL",

    // 0x30 ~ 0x3F
    "JR NC $00",
    "LD SP, $0000",
    "LDD (HL), A",
    "INC SP",
    "INC (HL)",
    "DEC (HL)",
    "LD (HL), $00",
    "SCF",
    "JR C $00",
    "ADD HL, SP",
    "LDD A, (HL)",
    "DEC SP",
    "INC A",
    "DEC A",
    "LD A, $00",
    "CCF",

    // 0x40 ~ 0x4F
    "LD B, B",
    "LD B, C",
    "LD B, D",
    "LD B, E",
    "LD B, H",
    "LD B, L",
    "LD B, (HL)",
    "LD B, A",
    "LD C, B",
    "LD C, C",
    "LD C, D",
    "LD C, E",
    "LD C, H",
    "LD C, L",
    "LD C, (HL)",
    "LD C, A",

    // 0x50 ~ 0x5F
    "LD D, B",
    "LD D, C",
    "LD D, D",
    "LD D, E",
    "LD D, H",
    "LD D, L",
    "LD D, (HL)",
    "LD D, A",
    "LD E, B",
    "LD E, C",
    "LD E, D",
    "LD E, E",
    "LD E, H",
    "LD E, L",
    "LD E, (HL)",
    "LD E, A",

    // 0x60 ~ 0x6F
    "LD H, B",
    "LD H, C",
    "LD H, D",
    "LD H, E",
    "LD H, H",
    "LD H, L",
    "LD H, (HL)",
    "LD H, A",
    "LD L, B",
    "LD L, C",
    "LD L, D",
    "LD L, E",
    "LD L, H",
    "LD L, L",
    "LD L, (HL)",
    "LD L, A",

    // 0X70 ~ 0X7F
    "LD (HL), B",
    "LD (HL), C",
    "LD (HL), D",
    "LD (HL), E",
    "LD (HL), H",
    "LD (HL), L",
    "LD (HL), (HL)",
    "LD (HL), A",
    "LD A, B",
    "LD A, C",
    "LD A, D",
    "LD A, E",
    "LD A, H",
    "LD A, L",
    "LD A, (HL)",
    "LD A, A",

    // 0X80 ~ 0X8F
    "ADD A, B",
    "ADD A, C",
    "ADD A, D",
    "ADD A, E",
    "ADD A, H",
    "ADD A, L",
    "ADD A, (HL)",
    "ADD A, A",
    "ADC A, B",
    "ADC A, C",
    "ADC A, D",
    "ADC A, E",
    "ADC A, H",
    "ADC A, L",
    "ADC A, (HL)",
    "ADC A, A",

    // 0X90 ~ 0X9F
    "SUB A, B",
    "SUB A, C",
    "SUB A, D",
    "SUB A, E",
    "SUB A, H",
    "SUB A, L",
    "SUB A, (HL)",
    "SUB A, A",
    "SBC A, B",
    "SBC A, C",
    "SBC A, D",
    "SBC A, E",
    "SBC A, H",
    "SBC A, L",
    "SBC A, (HL)",
    "SBC A, A",

    // 0XA0 ~ 0XAF
    "AND A, B",
    "AND A, C",
    "AND A, D",
    "AND A, E",
    "AND A, H",
    "AND A, L",
    "AND A, (HL)",
    "AND A, A",
    "XOR A, B",
    "XOR A, C",
    "XOR A, D",
    "XOR A, E",
    "XOR A, H",
    "XOR A, L",
    "XOR A, (HL)",
    "XOR A, A",

    // 0XB0 ~ 0XBF
    "OR A, B",
    "OR A, C",
    "OR A, D",
    "OR A, E",
    "OR A, H",
    "OR A, L",
    "OR A, (HL)",
    "OR A, A",
    "CP A, B",
    "CP A, C",
    "CP A, D",
    "CP A, E",
    "CP A, H",
    "CP A, L",
    "CP A, (HL)",
    "CP A, A",

    // 0XC0 ~ 0XCF
    "RET NZ",
    "POP BC",
    "JP NZ $0000",
    "JP $0000",
    "CALL NZ $0000",
    "PUSH BC",
    "ADD A, $00",
    "RST $00",
    "RET Z",
    "RET",
    "JP Z $0000",
    "PREFIX CB",
    "CALL Z $0000",
    "CALL $0000",
    "ADC A, $00",
    "RST $08",

    // 0XD0 ~ 0XDF
    "RET NC",
    "POP DE",
    "JP NC $0000",
    "[D3] - INVALID;",
    "CALL NC $0000",
    "PUSH DE",
    "SUB $00",
    "RST $10",
    "RET C",
    "RETI",
    "JP C $0000",
    "[DB] - INVALID;",
    "CALL C $0000",
    "[DD] - INVALID;",
    "SBC A, $00",
    "RST $18",

    // 0XE0 ~ 0XEF
    "LDH ($FF$00),A;",
    "POP HL",
    "LD (C), A",
    "[E3] - INVALID;",
    "[E4] - INVALID;",
    "PUSH HL",
    "AND $00",
    "RST $20",
    "ADD SP,$00",
    "JP HL",
    "LD ($0000), A",
    "[EB] - INVALID;",
    "[EC] - INVALID;",
    "[ED] - INVALID;",
    "XOR $00",
    "RST $28",

    // 0XF0 ~ 0XFF
    "LDH A,($FF$00);",
    "POP AF",
    "LD A, (C)",
    "DI",
    "[F4] - INVALID;",
    "PUSH AF",
    "OR $00",
    "RST $30",
    "LD HL, SP+$00;",
    "LD SP, HL",
    "LD A, ($0000)",
    "EI",
    "[FC] - INVALID;",
    "[FD] - INVALID;",
    "CP $00",
    "RST $38",
];

#[allow(dead_code)]
/// Instruction Assembly (CB Extension)
pub const INST_ASM_CB: [&str; 256] = [
    // 0x00 ~ 0x07
    "RLC B",
    "RLC C",
    "RLC D",
    "RLC E",
    "RLC H",
    "RLC L",
    "RLC (HL)",
    "RLC A",

    // 0x08 ~ 0x0F
    "RRC B",
    "RRC C",
    "RRC D",
    "RRC E",
    "RRC H",
    "RRC L",
    "RRC (HL)",
    "RRC A",

    // 0x10 ~ 0x17
    "RL B",
    "RL C",
    "RL D",
    "RL E",
    "RL H",
    "RL L",
    "RL (HL)",
    "RL A",
 
    // 0x18 ~ 0x1F
    "RR B",
    "RR C",
    "RR D",
    "RR E",
    "RR H",
    "RR L",
    "RR (HL)",
    "RR A",

    // 0x20 ~ 0x27
    "SLA B",
    "SLA C",
    "SLA D",
    "SLA E",
    "SLA H",
    "SLA L",
    "SLA (HL)",
    "SLA A",

    // 0x28 ~ 0x2F
    "SRA B",
    "SRA C",
    "SRA D",
    "SRA E",
    "SRA H",
    "SRA L",
    "SRA (HL)",
    "SRA A",

    // 0x30 ~ 0x37
    "SWAP B",
    "SWAP C",
    "SWAP D",
    "SWAP E",
    "SWAP H",
    "SWAP L",
    "SWAP (HL)",
    "SWAP A",

    // 0x38 ~ 0x3F
    "SRL B",
    "SRL C",
    "SRL D",
    "SRL E",
    "SRL H",
    "SRL L",
    "SRL (HL)",
    "SRL A",

    // 0x40 ~ 0x47
    "BIT 0, B",
    "BIT 0, C",
    "BIT 0, D",
    "BIT 0, E",
    "BIT 0, H",
    "BIT 0, L",
    "BIT 0, (HL)",
    "BIT 0, A",

    // 0x48 ~ 0x4F
    "BIT 1, B",
    "BIT 1, C",
    "BIT 1, D",
    "BIT 1, E",
    "BIT 1, H",
    "BIT 1, L",
    "BIT 1, (HL)",
    "BIT 1, A",

    // 0x50 ~ 0x57
    "BIT 2, B",
    "BIT 2, C",
    "BIT 2, D",
    "BIT 2, E",
    "BIT 2, H",
    "BIT 2, L",
    "BIT 2, (HL)",
    "BIT 2, A",

    // 0x58 ~ 0x5F
    "BIT 3, B",
    "BIT 3, C",
    "BIT 3, D",
    "BIT 3, E",
    "BIT 3, H",
    "BIT 3, L",
    "BIT 3, (HL)",
    "BIT 3, A",

    // 0x60 ~ 0x67
    "BIT 4, B",
    "BIT 4, C",
    "BIT 4, D",
    "BIT 4, E",
    "BIT 4, H",
    "BIT 4, L",
    "BIT 4, (HL)",
    "BIT 4, A",

    // 0x68 ~ 0x6F
    "BIT 5, B",
    "BIT 5, C",
    "BIT 5, D",
    "BIT 5, E",
    "BIT 5, H",
    "BIT 5, L",
    "BIT 5, (HL)",
    "BIT 5, A",

    // 0x70 ~ 0x77
    "BIT 6, B",
    "BIT 6, C",
    "BIT 6, D",
    "BIT 6, E",
    "BIT 6, H",
    "BIT 6, L",
    "BIT 6, (HL)",
    "BIT 6, A",

    // 0x78 ~ 0x8F
    "BIT 7, B",
    "BIT 7, C",
    "BIT 7, D",
    "BIT 7, E",
    "BIT 7, H",
    "BIT 7, L",
    "BIT 7, (HL)",
    "BIT 7, A",

    // 0x80 ~ 0x87
    "RES 0, B",
    "RES 0, C",
    "RES 0, D",
    "RES 0, E",
    "RES 0, H",
    "RES 0, L",
    "RES 0, (HL)",
    "RES 0, A",

    // 0x88 ~ 0x8F
    "RES 1, B",
    "RES 1, C",
    "RES 1, D",
    "RES 1, E",
    "RES 1, H",
    "RES 1, L",
    "RES 1, (HL)",
    "RES 1, A",

    // 0x90 ~ 0x97
    "RES 2, B",
    "RES 2, C",
    "RES 2, D",
    "RES 2, E",
    "RES 2, H",
    "RES 2, L",
    "RES 2, (HL)",
    "RES 2, A",

    // 0x98 ~ 0x9F
    "RES 3, B",
    "RES 3, C",
    "RES 3, D",
    "RES 3, E",
    "RES 3, H",
    "RES 3, L",
    "RES 3, (HL)",
    "RES 3, A",

    // 0xA0 ~ 0xA7
    "RES 4, B",
    "RES 4, C",
    "RES 4, D",
    "RES 4, E",
    "RES 4, H",
    "RES 4, L",
    "RES 4, (HL)",
    "RES 4, A",

    // 0xA8 ~ 0xAF
    "RES 5, B",
    "RES 5, C",
    "RES 5, D",
    "RES 5, E",
    "RES 5, H",
    "RES 5, L",
    "RES 5, (HL)",
    "RES 5, A",

    // 0xB0 ~ 0xB7
    "RES 6, B",
    "RES 6, C",
    "RES 6, D",
    "RES 6, E",
    "RES 6, H",
    "RES 6, L",
    "RES 6, (HL)",
    "RES 6, A",

    // 0xB8 ~ 0xBF
    "RES 7, B",
    "RES 7, C",
    "RES 7, D",
    "RES 7, E",
    "RES 7, H",
    "RES 7, L",
    "RES 7, (HL)",
    "RES 7, A",

    // 0xC0 ~ 0xC7
    "SET 0, B",
    "SET 0, C",
    "SET 0, D",
    "SET 0, E",
    "SET 0, H",
    "SET 0, L",
    "SET 0, (HL)",
    "SET 0, A",

    // 0xC8 ~ 0xCF
    "SET 1, B",
    "SET 1, C",
    "SET 1, D",
    "SET 1, E",
    "SET 1, H",
    "SET 1, L",
    "SET 1, (HL)",
    "SET 1, A",

    // 0xD0 ~ 0xD7
    "SET 2, B",
    "SET 2, C",
    "SET 2, D",
    "SET 2, E",
    "SET 2, H",
    "SET 2, L",
    "SET 2, (HL)",
    "SET 2, A",

    // 0xD8 ~ 0xDF
    "SET 3, B",
    "SET 3, C",
    "SET 3, D",
    "SET 3, E",
    "SET 3, H",
    "SET 3, L",
    "SET 3, (HL)",
    "SET 3, A",

    // 0xE0 ~ 0xE7
    "SET 4, B",
    "SET 4, C",
    "SET 4, D",
    "SET 4, E",
    "SET 4, H",
    "SET 4, L",
    "SET 4, (HL)",
    "SET 4, A",

    // 0xE8 ~ 0xEF
    "SET 5, B",
    "SET 5, C",
    "SET 5, D",
    "SET 5, E",
    "SET 5, H",
    "SET 5, L",
    "SET 5, (HL)",
    "SET 5, A",

    // 0xF0 ~ 0xF7
    "SET 6, B",
    "SET 6, C",
    "SET 6, D",
    "SET 6, E",
    "SET 6, H",
    "SET 6, L",
    "SET 6, (HL)",
    "SET 6, A",

    // 0xF8 ~ 0xFF
    "SET 7, B",
    "SET 7, C",
    "SET 7, D",
    "SET 7, E",
    "SET 7, H",
    "SET 7, L",
    "SET 7, (HL)",
    "SET 7, A",
];
