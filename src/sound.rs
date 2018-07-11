use sdl2::audio::*;
use sdl2;
use std::sync::mpsc::*;

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
            self.pos[i] = 0;
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
    for _ in 0..5 {
        let max = (44100/hz/2) as u8;
        for i in 0..max {
            data.push(i * 2);
        }
        for i in 0..max {
            data.push((max - i - 1) * 2);
        }
    }
    data
}

pub fn init_audio() -> NesSound {
    let sdl = sdl2::init().unwrap();

    let audio = sdl.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(1), // mono
        samples: None      // default
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
        send
    }
}

pub struct NesSound {
    _audio: sdl2::AudioSubsystem,
    _device:  AudioDevice<SoundData>,
    send: Sender<(usize, Vec<u8>)>
}

impl NesSound {
    pub fn play_tone(&self, hz: u32) {
        self.send.send((0,make_tone(hz))).unwrap();
    }
}