use sdl2::audio::*;
use sdl2;
use std::sync::mpsc::*;
use memory::Mapper;
use memory::Mem;
use cpu::Cpu;

type SoundEnvelopeData = (usize, Vec<u8>, bool);

struct SoundData {
    data: [Vec<u8>; 5],
    volume: f32,
    pos: [usize; 5],
    repeat: [bool; 5],

    recv: Receiver<SoundEnvelopeData>,
}

impl AudioCallback for SoundData {
    type Channel = u8;

    fn callback(&mut self, out: &mut [u8]) {

        while let Ok((i, d, r)) = self.recv.try_recv() {
            self.data[i] = d;
            self.pos[i] = 0;
            self.repeat[i] = r;
        }

        for dst in out.iter_mut() {
            let mut val = 0u32;

            for i in 0..self.data.len() {
                if self.repeat[i] && self.pos[i] >= self.data[i].len() {
                    self.pos[i] = 0;
                }
                val += (*self.data[i].get(self.pos[i]).unwrap_or(&0) as f32 * self.volume) as u32;
                self.pos[i] += 1;
            }

            *dst = (val/5) as u8;
        }
    }
}

pub fn init_audio() -> NesSound {
    let sdl = sdl2::init().unwrap();

    let audio = sdl.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(SAMPLES_PER_SECOND as i32),
        channels: Some(1), // mono
        samples: Some(128)
    };

    let (send, recv) = channel();

    let device = audio.open_playback(None, &desired_spec, |_spec| {
        SoundData {
            data: [vec![],vec![],vec![],vec![],vec![]],
            volume: 1.0,
            pos: [0,0,0,0,0],
            repeat: [false, false, false, false, false],
            recv,
        }
    }).unwrap();

    // Start playback
    device.resume();

    NesSound {
        _audio: audio,
        _device: device,
        send,
        timer: [0; 3],
        length_counter: [0; 3],
        length_counter_halt: [false; 3],
        constant_volume: [false; 2],
        volume: [0; 2],

        dirty: false,
        did_clock: 0,

        frame_counter_inhibit: false,
        frame_counter_mode: 0,
    }
}

pub struct NesSound {
    _audio: sdl2::AudioSubsystem,
    _device:  AudioDevice<SoundData>,
    send: Sender<SoundEnvelopeData>,

    timer: [u16; 3],
    length_counter: [u8; 3],
    length_counter_halt: [bool; 3],
    constant_volume: [bool; 2],
    volume: [u8; 2],

    frame_counter_inhibit: bool,
    frame_counter_mode: u8,

    dirty: bool,
    did_clock: u8,
}

impl NesSound {
    pub fn tick(&mut self, cpu: &mut Cpu, _mapper: &mut Box<Mapper>) {
        if cpu.count < 14915 {
            self.did_clock = 0;
        } else if cpu.count >= 14915 && self.did_clock == 0 || cpu.count >= 14915*2 && self.did_clock == 1 {
            self.did_clock = self.did_clock + 1;
            self.length_counter[0] = if self.length_counter[0] >= 1 { self.length_counter[0] - 1 } else { 0 };
        }
        else if self.did_clock == 3 {
            //TODO untested
            self.did_clock = self.did_clock + 1;
            if self.frame_counter_mode == 0 && !self.frame_counter_inhibit {
                println!("IRQ");
                cpu.irq();
            }
        }

        if !self.dirty { return; }
        self.dirty = false;

//        println!("Timer: {} LengthCounter: {} Loop: {} Const: {} Volume: {}, frame counter: {}, {}", self.timer[0], self.length_counter[0], self.length_counter_halt[0], self.constant_volume[0], self.volume[0], self.frame_counter_mode, self.frame_counter_inhibit);
        self.send.send(self.make_data()).unwrap();
    }

    fn make_data(&self) -> SoundEnvelopeData {
        let mut data = vec![];
        let repeat = self.length_counter_halt[0];
        let wave_apu_cycles_period = 8.0*(self.timer[0] as f64 + 1.0);
        let wave_samples_period = wave_apu_cycles_period / APU_CYCLES_PER_SAMPLE;
        let envelope_period = (self.volume[0] as f64 + 1.0) * APU_CYCLES_PER_ENVELOPE_CLOCK;

        let note_duration_apu_cycles = self.length_counter[0] as f64 * 2.0 * APU_CYCLES_PER_ENVELOPE_CLOCK;

//        println!("Repeat: {} WaveSamplesPeriod: {}, EnvelopePeriod: {}", repeat, wave_samples_period, envelope_period);

        while data.len() as f64 * APU_CYCLES_PER_SAMPLE <= note_duration_apu_cycles {
            let wave_pos = if (data.len() as f64 % wave_samples_period) <= wave_samples_period/2.0 { 0 } else { 1 };

            let volume = if self.constant_volume[0] { 8 * self.volume[0] as u32 } else {
                let envelope_pos = data.len() as f64 / envelope_period as f64;
                if envelope_pos > 1.0 { 0 } else { ((1.0 - envelope_pos) * 15.0 * 9.0) as u32 }
            };
            let sample = wave_pos * volume;
            data.push(sample as u8);
        }
        (0, data, repeat)
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
                let old_counter_inhibit = if self.frame_counter_inhibit {1} else {0};
                self.frame_counter_inhibit = false;
                (old_counter_inhibit << 6) | (if self.length_counter[0] > 0 { 0b00000001 } else {0})
            }
            _ => 0
        }
    }

    fn write(&mut self, _mapper: &mut Box<Mapper>, addr: u16, val: u8) {
//        println!("Write {:X} to {:X}", val, addr);
        match addr as usize {
            0x4004 => {
                self.length_counter_halt[0] = (val&0b00100000) != 0;
                self.constant_volume[0] = (val&0b00010000) != 0;
                self.volume[0] = val&0b00001111;
                self.dirty = true;
            }
            0x4006 => {
                self.dirty = true;
                self.timer[0] = (self.timer[0] & 0b11111111_00000000) | ((val as u16) & 0b00000000_11111111);
            }
            0x4007 => {
                self.timer[0] = (self.timer[0] & 0b00000000_11111111) | ((val as u16 & 0b00000111) << 8);
                self.length_counter[0] = *LENGTH_LOOKUP.get((val as usize & 0b11111000) >> 3).unwrap_or(&0);
                self.dirty = true;
            }
            0x4017 => {
                self.frame_counter_mode = (val&0b10000000)>>7;
                self.frame_counter_inhibit = (val&0b01000000)!=0;
            }
            _ => {}
        }
    }
}