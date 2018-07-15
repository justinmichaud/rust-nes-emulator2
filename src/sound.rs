use sdl2::audio::*;
use sdl2;
use memory::Mapper;
use memory::Mem;
use cpu::Cpu;
use std::sync::Arc;
use std::sync::Mutex;
use sdl2::Sdl;

struct SoundData {
    state_mut: Arc<Mutex<NesApuState>>,
}

impl AudioCallback for SoundData {
    type Channel = u8;

    fn callback(&mut self, out: &mut [u8]) {
        let mut state = self.state_mut.lock().unwrap();

        for dst in out.iter_mut() {
            *dst = state.tick() as u8;
        }
    }
}

pub fn init_audio(sdl: Sdl) -> NesSound {
    let audio = sdl.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(SAMPLES_PER_SECOND as i32),
        channels: Some(1), // mono
        samples: Some(128)
    };

    let apu_state = NesApuState {
        envelope_timer_samples: 0,
        wave_timer_samples: 0,
        length_counter_samples: 0,
        length_counter: 0,
        length_counter_halt: false,
        volume: 0,
        constant_volume: false,
        timer: 0,
    };

    let arc = Arc::new(Mutex::new(apu_state));

    let device = audio.open_playback(None, &desired_spec, |_spec| {
        SoundData {
            state_mut: arc.clone(),
        }
    }).unwrap();

    // Start playback
    device.resume();

    NesSound {
        _audio: audio,
        _device: device,
        state_mut: arc,

        frame_counter_inhibit: false,
        frame_counter_mode: 0,
    }
}

struct NesApuState {
    envelope_timer_samples: u32,
    wave_timer_samples: u32,
    length_counter_samples: u32,

    length_counter: u8,
    length_counter_halt: bool,
    volume: u8,
    constant_volume: bool,
    timer: u16,
}

impl NesApuState {
    fn tick(&mut self) -> u32 {
        let wave_samples_period = 8.0*(self.timer as f64 + 1.0) / APU_CYCLES_PER_SAMPLE;
        let envelope_samples_period = (self.volume as f64 + 1.0) * APU_CYCLES_PER_ENVELOPE_CLOCK / APU_CYCLES_PER_SAMPLE;
        let note_duration_samples = self.length_counter as f64 * 2.0 * APU_CYCLES_PER_ENVELOPE_CLOCK / APU_CYCLES_PER_SAMPLE;

        let wave_pos = if (self.wave_timer_samples as f64 % wave_samples_period) <= wave_samples_period/2.0 { 0 } else { 1 };

        let volume = if self.constant_volume { 8 * self.volume as u32 } else {
            let envelope_pos = if self.length_counter_halt {
                (self.envelope_timer_samples as f64 % envelope_samples_period) / envelope_samples_period
            } else {
                self.envelope_timer_samples as f64 / envelope_samples_period
            };
            if envelope_pos > 1.0 { 0 } else {
                ((1.0 - envelope_pos) * 15.0 * 8.0) as u32
            }
        };

        let length_counter_now = if self.length_counter_halt { self.length_counter } else {
            let pos = self.length_counter_samples as f64 / note_duration_samples;
            if pos > 1.0 { 0 } else { self.length_counter }
        };

        let sample = if length_counter_now > 0 && self.timer > 8 {
            wave_pos * volume
        } else { 0 };

        self.length_counter = length_counter_now;
        self.envelope_timer_samples += 1;
        self.wave_timer_samples += 1;
        self.length_counter_samples += 1;

        sample
    }
}

pub struct NesSound {
    _audio: sdl2::AudioSubsystem,
    _device:  AudioDevice<SoundData>,
    state_mut: Arc<Mutex<NesApuState>>,

    frame_counter_inhibit: bool,
    frame_counter_mode: u8,
}

impl NesSound {
    pub fn tick(&mut self, _cpu: &mut Cpu, _mapper: &mut Box<Mapper>) {
    }
}

// Table stolen from https://github.com/andrew-hoffman/halfnes/blob/master/src/main/java/com/grapeshot/halfnes/APU.java
const LENGTH_LOOKUP: [u8; 32] = [10, 254, 20, 2, 40, 4, 80, 6,
    160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30];
// https://nesdoug.com/2015/12/02/14-intro-to-sound/
// https://wiki.nesdev.com/w/index.php/APU
const APU: f64 = 1789773.0/2.0;
const APU_CYCLES_PER_ENVELOPE_CLOCK: f64 = 3728.5;
const APU_CYCLES_PER_SAMPLE: f64 = APU/SAMPLES_PER_SECOND as f64;
const SAMPLES_PER_SECOND: u32 = 44100;

impl Mem for NesSound {
    fn read(&mut self, _mapper: &mut Box<Mapper>, addr: u16) -> u8 {
        match addr as usize {
            0x4015 => {
                let mut state = self.state_mut.lock().unwrap();

                let old_counter_inhibit = if self.frame_counter_inhibit {1} else {0};
                self.frame_counter_inhibit = false;
                (old_counter_inhibit << 6) | (if state.length_counter > 0 { 0b00000001 } else {0})
            }
            _ => 0
        }
    }

    fn write(&mut self, _mapper: &mut Box<Mapper>, addr: u16, val: u8) {
        let mut state = self.state_mut.lock().unwrap();

        match addr as usize {
            0x4004 => {
                state.length_counter_halt = (val&0b00100000) != 0;
                state.constant_volume = (val&0b00010000) != 0;
                state.volume = val&0b00001111;
                state.length_counter_samples = 0;
                state.envelope_timer_samples = 0;
            }
            0x4006 => {
                state.timer = (state.timer & 0b11111111_00000000) | ((val as u16) & 0b00000000_11111111);
                state.wave_timer_samples = 0;
            }
            0x4007 => {
                state.timer = (state.timer & 0b00000000_11111111) | ((val as u16 & 0b00000111) << 8);
                state.length_counter = *LENGTH_LOOKUP.get((val as usize & 0b11111000) >> 3).unwrap_or(&0);
                state.length_counter_samples = 0;
                state.envelope_timer_samples = 0;
                state.wave_timer_samples = 0;
            }
            0x4017 => {
                self.frame_counter_mode = (val&0b10000000)>>7;
                self.frame_counter_inhibit = (val&0b01000000)!=0;
            }
            _ => {}
        }
    }
}