# Rust NES emulator v2

Games that don't use any fancy ppu trickery work, including Donkey Kong and Super Mario Bros. Only mapper 0 is supported. Sound is not supported.

![Super Mario Bros](/smb.gif?raw=true "Super Mario Bros")

# Building for web
This used to work, but I need to fix it

# Building for desktop
Install SDL2-devel, then `cargo run --release`. Put rom file in assets/smb.nes (sha1sum: ea343f4e445a9050d4b4fbac2c77d0693b1d0922)

# Bugs:
- Piston HiDPI support seems to be broken? Also, on Fedora the window piston makes is very buggy