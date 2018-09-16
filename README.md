# Rust NES emulator v2

A simple NES emulator, with partial support for Mapper 4 (Super Mario Bros. 2 and 3 use this) and audio. It is still very buggy, and was built entirely for the learning experience.

This is the second iteration of [v1](https://github.com/justinmichaud/rust-nes-emulator). This version adds a few features, fixes some build issues, and removes the Super Mario Bros hacks / level editing capabilities of the first version.

![Super Mario Bros](/smb.gif?raw=true "Super Mario Bros")

![Super Mario Bros 2](/smb2.1.png?raw=true "Super Mario Bros 2")
![Super Mario Bros 2](/smb2.2.png?raw=true "Super Mario Bros 2")
![Super Mario Bros 2](/smb2.3.png?raw=true "Super Mario Bros 2")

![Super Mario Bros 3](/smb3.1.png?raw=true "Super Mario Bros 3")
![Super Mario Bros 3](/smb3.2.png?raw=true "Super Mario Bros 3")
![Super Mario Bros 3](/smb3.3.png?raw=true "Super Mario Bros 3")

There are still a few bugs left to work out in SMB3 relating to graphical glitches. Also, performance could be improved and the code could be cleaned up significantly. The scanline emulation in particular is slow and inaccurate.

For audio, the two pulse channels are supported, but sweep is buggy. The triangle, noise and DMC channels are not supported. This is enough to hear the melody of the Super Mario Bros games, but special effects are wonky and there is no bass or percussion.

# Building for web
This used to work, but I need to fix it

# Building for desktop
Install SDL2-devel, then `cargo run --release`. Put rom file in assets/smb.nes (sha1sum: ea343f4e445a9050d4b4fbac2c77d0693b1d0922)