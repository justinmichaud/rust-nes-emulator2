# Rust NES emulator v2

Games that don't use any fancy ppu trickery work, including Donkey Kong and Super Mario Bros. Only mapper 0 is supported. Sound is not supported.

![Super Mario Bros](/smb.gif?raw=true "Super Mario Bros")

# Building for web
Download the emscripten portable sdk, and source the emsdk_env.sh script. Run `make` and `make serve`, and you should be good to go! This is a huge PITA, so it may take some hacking around.

# Building for desktop
Install SDL2-devel, then build. Then, `cargo run --release --bin emulator`. Put rom file in assets/smb.nes (sha1sum: ea343f4e445a9050d4b4fbac2c77d0693b1d0922)