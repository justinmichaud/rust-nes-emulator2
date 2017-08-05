# Rust NES emulator

See [justinmichaud.com](http://justinmichaud.com/smb_challenge/index.html) for a playable demo.

Games that don't use any fancy ppu trickery work, including Donkey Kong and Super Mario Bros. Only mapper 0 is supported. Sound is not supported.

![Super Mario Bros](/smb.gif?raw=true "Super Mario Bros")

# Super Mario Bros level generation

When given a Super Mario Bros. rom, the emulator can spit out a text file representation of the overworld levels in the game. The following level was generated using the output of emulator (overworld levels from SMB + the lost levels, repeated 20x each in random order), and torch-rnn with the default settings after 20 epochs. It was modified to add the begining level header and the final flag.

A playable demo of the generated level is available above, or (modified to fit the game, ID AC01-0000-034E-BC93) on Super Mario Maker.

![](/0.png?raw=true)

# Super Mario Bros Hacks

If USE_HACKS in settings.rs is set to true, the title screen and prelevel screens will be automatically skipped, and you will have infinite lives.
If USE_HACKS and SPECIAL are set, the game is tweaked for a one-button challenge. You can jump, and you cannot stop. The game screen is warped for an extra challenge, but deaths are instant and you have infinite lives:

![Super Mario Bros - SPECIAL and USE_HACKS](/smb-special-usehacks.png?raw=true "Super Mario Bros SPECIAL and USE HACKS")

# Building for web
Download the emscripten portable sdk, and source the emsdk_env.sh script. Run `make` and `make serve`, and you should be good to go!
