# 🥝 KiwiGB

A low-level gameboy emulator written in rust and SDL2.

![Tetris Main Screen](./pics/tetris.png)

![Dr. Mario Demo Screen](./pics/dr-mario.png)

Forked and restructured from https://github.com/Miliox/kiwi using unsafe.

## Dependencies

```
# macOS
brew install sdl2 sdl2_ttf

# Ubuntu
apt install libsdl2-dev libsdl2-ttf-dev
```

## Test Room

- [x] BIOS
- [x] Tetris
- [ ] Dr. Mario (Kind Playable, Sprite issues)
- [ ] Alleway   (Sprite Mess)

### Blargg GB (Pass)

- [x] 01-special
- [x] 02-interrupt
- [x] 03-op sp,hl
- [x] 04-op r,imm
- [x] 05-op rp
- [x] 06-ld r,r
- [x] 07-jr,jp,call,ret,rst
- [x] 08-misc instrs
- [x] 09-op r,r
- [x] 10-bit ops
- [x] 11-op a,{hl}
- [ ] cpu_instrs (MBC1)