#[allow(dead_code)]
pub const DMG_BIOS: [u8; 0x100] = [
    0x31, 0xfe, 0xff, // LD SP, $fffe      ; $0000 Setup Stack
    0xaf,             // XOR A             ; $0003 Zero the memory from $8000-$9fff (VRAM)
    0x21, 0xff, 0x9f, // LD HL, $9ffff     ; $0004

    // Addr_07:
    0x32,             // LD (HL-), A       ; $0007
    0xcb, 0x7c,       // BIT 7, H          ; $0008
    0x20, 0xfb,       // JR NZ, Addr_07    ; $000a

    0x21, 0x26, 0xff, // LD HL, $ff26      ; $000c Setup Audio
    0x0e, 0x11,       // LD C, $11         ; $000f
    0x3e, 0x80,       // LD A, $80         ; $0011
    0x32,             // LD (HL-), A       ; $0013
    0xe2,             // LD ($FF00+C), A   ; $0014
    0x0c,             // INC C             ; $0015
    0x3e, 0xf3,       // LD A, $f3         ; $0016
    0xe2,             // LD ($FF00+C), A   ; $0018
    0x32,             // LD (HL-), A       ; $0019
    0x3e, 0x77,       // LD A, $77         ; $001a
    0x77,             // LD (HL), A        ; $001c

    0x3e, 0xfc,       // LD A, $fc         ; $001d Setup BG Palette
    0xe0, 0x47,       // LD ($FF00+$47), A ; $001f

    0x11, 0x04, 0x01, // LD DE, $0104      ; $0021 Convert and load logo from cartridge into Video RAM
    0x21, 0x10, 0x80, // LD HL, $8010      ; $0024

    // Addr_27:
    0x1a,             // LD A, (DE)        ; $0027
    0xcd, 0x95, 0x00, // CALL $0095        ; $0028
    0xcd, 0x96, 0x00, // CALL $0096        ; $002b
    0x13,             // INC DE            ; $002e
    0x7b,             // LD A, E           ; $002f
    0xfe, 0x34,       // CP $34            ; $0030
    0x20, 0xf3,       // JR NZ, Addr_27    ; $0032

    0x11, 0xd8, 0x00, // LD DE, $00d8      ; $0034 Load additional bytes into Video RAM
    0x06, 0x08,       // LD B, $08         ; $0037

    // Addr_39:
    0x1a,             // LD A, (DE)        ; $0039
    0x13,             // INC DE            ; $003a
    0x22,             // LD (HL+), A       ; $003b
    0x23,             // INC HL            ; $003c
    0x05,             // DEC B             ; $003d
    0x20, 0xf9,       // JR NZ, Addr_39    ; $003e

    0x3e, 0x19,       // LD A, $19         ; $0040 Setup Background Tile Map
    0xea, 0x10, 0x99, // LD ($9910), A     ; $0042
    0x21, 0x2f, 0x99, // LD HL, $992f      ; $0045

    // Addr_48:
    0x0e, 0x0c,       // LD C, $0c         ; $0048

    // Addr_4A:
    0x3d,             // DEC A             ; $004a
    0x28, 0x08,       // JR Z, Addr_55     ; $004b
    0x32,             // LD (HL-), A       ; $004d
    0x0d,             // DEC C             ; $004e
    0x20, 0xf9,       // JR NZ, Addr_4A    ; $004f
    0x2e, 0x0f,       // LD L, $0f         ; $0051
    0x18, 0xf3,       // JR Addr_48        ; $0053

    // Addr_55: Scroll logo on screen, and play logo sound
    0x67,             // LD H, A           ; $0055 Initialize scroll count, H=0
    0x3e, 0x64,       // LD A, $64         ; $0056
    0x57,             // LD D, A           ; $0058 Set loop count, D=$64
    0xe0, 0x42,       // LD ($FF00+42), A  ; $0059 Set vertical scroll register
    0x3e, 0x91,       // LD A, $91         ; $005b
    0xe0, 0x40,       // LD ($FF00+40), A  ; $005d Turn on LCD, showing background
    0x04,             // INC B             ; $005f Set B=1

    // Addr_60:
    0x1e, 0x02,       // LD E, $02         ; $0060

    // Addr_62:
    0x0e, 0x0c,       // LD C, $0c         ; $0062

    // Addr_64:
    0xf0, 0x44,       // LD A, ($FF00+44)  ; $0064 Wait for screen frame
    0xfe, 0x90,       // CP $90            ; $0066
    0x20, 0xfa,       // JR NZ, Addr_64    ; $0068
    0x0d,             // DEC C             ; $006a
    0x20, 0xf7,       // JR NZ, Addr_64    ; $006b
    0x1d,             // DEC E             ; $006d
    0x20, 0xf2,       // JR NZ, Addr_62    ; $006e

    0x0e, 0x13,       // LD C, $13         ; $0070
    0x24,             // INC H             ; $0072 Increment scroll count
    0x7c,             // LD A, H           ; $0073
    0x1e, 0x83,       // LD E, $83         ; $0074
    0xfe, 0x62,       // CP $62            ; $0076 $62 counts in, play sound #1
    0x28, 0x06,       // JR Z, Addr_80     ; $0078
    0x1e, 0xc1,       // LD E, $c1         ; $007a
    0xfe, 0x64,       // CP $64            ; $007c
    0x20, 0x06,       // JR NZ, Addr_86    ; $007e $64 counts in, play sound #2

    // Addr_80:
    0x7b,             // LD A, E           ; $0080 Play sound
    0xe2,             // LD ($FF00+C), A   ; $0081
    0x0c,             // INC C             ; $0082
    0x3e, 0x87,       // LD A, $87         ; $0083
    0xe2,             // LD ($FF00+C), A   ; $0085

    // Addr_86
    0xf0, 0x42,       // LD A, ($FF00+42)  ; $0086
    0x90,             // SUB B             ; $0088
    0xe0, 0x42,       // LD ($FF00+42), A  ; $0089 Scroll logo up if B=1
    0x15,             // DEC D             ; $008b
    0x20, 0xd2,       // JR NZ, Addr_60    ; $008c

    0x05,             // DEC B             ; $008e Set B=0 for the 1st time
    0x20, 0x4f,       // JR NZ, Addr_E0    ; $008f ... next time, jump to "Nintendo logo check"

    0x16, 0x20,       // LD D, $20         ; $0091 Use scrolling loop to pause
    0x18, 0xcb,       // JR Addr_60        ; $0093

    // Graphics Routines

    // Addr_95:
    0x4f,             // LD C, A           ; $0095 "Double Up" all the bits of the graphics data

    // Addr_96:
    0x06, 0x04,       // LD B, $04         ; $0096 ... and store in Video RAM

    // Addr_98:
    0xc5,             // PUSH BC           ; $0098
    0xcb, 0x11,       // RL C              ; $0099
    0x17,             // RLA               ; $009b
    0xc1,             // POP BC            ; $009c
    0xcb, 0x11,       // RL C              ; $009d
    0x17,             // RLA               ; $009f
    0x05,             // DEC B             ; $00a0
    0x20, 0xf5,       // JR NZ, Addr_98    ; $00a1
    0x22,             // LD (HL+), A       ; $00a3
    0x23,             // INC HL            ; $00a4
    0x22,             // LD (HL+), A       ; $00a5
    0x23,             // INC HL            ; $00a6
    0xc9,             // RET               ; $00a7

    // Addr_A8: Nintendo Logo Data
    0xce, 0xed, 0x66, 0x66, 0xcc, 0x0d, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0d,
    0x00, 0x08, 0x11, 0x1f, 0x88, 0x89, 0x00, 0x0e, 0xdc, 0xcc, 0x6e, 0xe6, 0xdd, 0xdd, 0xd9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6e, 0x0e, 0xec, 0xcc, 0xdd, 0xdc, 0x99, 0x9f, 0xbb, 0xb9, 0x33, 0x3e,

    // Addr_D8: More video data
    0x3c, 0x42, 0xb9, 0xa5, 0xb9, 0xa5, 0x42, 0x3c,

    // Addr_E0: Nintendo logo comparison routine
    0x21, 0x04, 0x01,  // LD HL, $0104     ; $00e0 Point HL to Nintendo Logo in Cartridge
    0x11, 0xa8, 0x00,  // LD DE, $00a8     ; $00e3 Point DE to Nintendo Logo in DMG ROM

    // Addr_E6:
    0x1a,              // LD A, (DE)       ; $00e6
    0x13,              // INC DE           ; $00e7
    0xbe,              // CP (HL)          ; $00e8 Compare logo data in cartridge to DMG ROM
    0x20, 0xfe,        // JR NZ, $FE       ; $00e9 If not match, lock up here
    0x23,              // INC HL           ; $00eb
    0x7d,              // LD A, L          ; $00ec
    0xfe, 0x34,        // CP $34           ; $00ed Do this for $30 bytes
    0x20, 0xf5,        // JR NZ, Addr_F4   ; $00ef

    0x06, 0x19,        // LD B, $19        ; $00f1
    0x78,              // LD A, B          ; $00f3

    // Addr_F4:
    0x86,              // ADD (HL)         ; $00f4
    0x23,              // INC HL           ; $00f5
    0x05,              // DEC B            ; $00f6
    0x20, 0xfb,        // JR NZ, Addr_F4   ; $00f7
    0x86,              // ADD (HL)         ; $00f9
    0x20, 0xfe,        // JR NZ, $FE       ; $00fa if $19 + bytes from $0134-$014d don't sum to $00 lock up here
    0x3e, 0x01,        // LD A, $01        ; $00fc
    0xe0, 0x50,        // LD ($FF00+50), A ; $00fe Turn Off DMG ROM
];
