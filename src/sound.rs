use sdl2::audio::*;
use sdl2;
use std::sync::mpsc::*;
use memory::Mapper;
use memory::Mem;
use cpu::Cpu;

struct SoundData {
    data: [Vec<u8>; 5],
    volume: f32,
    pos: [usize; 5],

    recv: Receiver<(usize, Vec<u8>)>,
}

impl AudioCallback for SoundData {
    type Channel = u8;

    fn callback(&mut self, out: &mut [u8]) {

        if let Ok((i, d)) = self.recv.try_recv() {
            self.data[i] = d;
        }

        for dst in out.iter_mut() {
            let mut val = 0u32;

            for i in 0..self.data.len() {
                if self.pos[i] >= self.data[i].len() {
                    self.pos[i] = 0;
                }
                val += (*self.data[i].get(self.pos[i]).unwrap_or(&0) as f32 * self.volume) as u32;
                self.pos[i] += 1;
            }

            *dst = (val/5) as u8;
        }
    }
}

fn make_tone(hz: u32) -> Vec<u8> {
    let mut data = vec![];
    if hz > 0 {
        for _ in 0..1 {
            let max = (44100/hz/2) as u8;
            for i in 0..max {
                data.push(i * 2);
            }
            for i in 0..max {
                data.push((max - i - 1) * 2);
            }
        }
    }
    data
}

pub fn init_audio() -> NesSound {
    let sdl = sdl2::init().unwrap();

    let audio = sdl.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1), // mono
        samples: Some(128)
    };

    let (send, recv) = channel();

    let device = audio.open_playback(None, &desired_spec, |_spec| {
        SoundData {
            data: [vec![],vec![],vec![],vec![],vec![]],
            volume: 1.0,
            pos: [0,0,0,0,0],
            recv
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

        frame_counter_inhibit: false,
        frame_counter_mode: 0,
    }
}

pub struct NesSound {
    _audio: sdl2::AudioSubsystem,
    _device:  AudioDevice<SoundData>,
    send: Sender<(usize, Vec<u8>)>,

    timer: [u16; 3],
    length_counter: [u16; 3],
    length_counter_halt: [bool; 3],
    constant_volume: [bool; 2],
    volume: [u8; 2],

    frame_counter_inhibit: bool,
    frame_counter_mode: u8,

    dirty: bool,

}

impl NesSound {
    pub fn tick(&mut self, cpu: &mut Cpu, _mapper: &mut Box<Mapper>) {
        if cpu.count >= 14915*4 {
            //TODO not quite accurate
            if !self.length_counter_halt[0] {
                self.length_counter[0] = if self.length_counter[0] >= 2 { self.length_counter[0] - 2 } else { 0 };
            }
            //TODO untested
            if self.frame_counter_mode == 0 && !self.frame_counter_inhibit {
                println!("IRQ");
                cpu.irq();
            }
        }

        if !self.dirty { return; }
        self.dirty = false;

        println!("{}, {}, {}, {}, {}, frame counter: {}, {}", self.timer[0], self.length_counter[0], self.length_counter_halt[0], self.constant_volume[0], self.volume[0], self.frame_counter_mode, self.frame_counter_inhibit);
        self.send.send((0,make_tone((1789773.0/(16.0*(self.timer[0] as f32 + 1.0))) as u32))).unwrap();
    }
}

const length_lookup: [u16; 88] = [
    // Shamelessly stolen from https://github.com/geky/mbed-apu/blob/master/source/channel.cpp
    0x7f1, 0x77f, 0x713, 0x6ad, 0x64d, 0x5f3, 0x59d, 0x54c,
    0x500, 0x4b8, 0x474, 0x434, 0x3f8, 0x3bf, 0x389, 0x356,
    0x326, 0x2f9, 0x2ce, 0x2a6, 0x280, 0x25c, 0x23a, 0x21a,
    0x1fb, 0x1df, 0x1c4, 0x1ab, 0x193, 0x17c, 0x167, 0x152,
    0x13f, 0x12d, 0x11c, 0x10c, 0xfd,  0xef,  0xe1,  0xd5,
    0xc9,  0xbd,  0xb3,  0xa9,  0x9f,  0x96,  0x8e,  0x86,
    0x7e,  0x77,  0x70,  0x6a,  0x64,  0x5e,  0x59,  0x54,
    0x4f,  0x4b,  0x46,  0x42,  0x3f,  0x3b,  0x38,  0x34,
    0x31,  0x2f,  0x2c,  0x29,  0x27,  0x25,  0x23,  0x21,
    0x1f,  0x1d,  0x1b,  0x1a,  0x18,  0x17,  0x15,  0x14,
    0x13,  0x12,  0x11,  0x10,  0xf,   0xe,   0xd,   0x0];

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
            0x4000 => {
                self.length_counter_halt[0] = (val&0b00100000) != 0;
                self.constant_volume[0] = (val&0b00010000) != 0;
                self.volume[0] = val&0b00001111;
            }
            0x4002 => self.timer[0] = (self.timer[0] & 0xFF00) + (val as u16),
            0x4003 => {
                self.timer[0] = self.timer[0] & 0x00FF + (val as u16 & 0b00000111) << 4;
                self.length_counter[0] = *length_lookup.get((val as usize & 0b11111000) >> 3).unwrap_or(&0);
                self.dirty = true;
            },
            0x4017 => {
                self.frame_counter_mode = (val&0b10000000)>>7;
                self.frame_counter_inhibit = (val&0b01000000)!=0;
            }
            _ => {}
        }
    }
}