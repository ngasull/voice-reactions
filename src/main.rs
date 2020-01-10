extern crate sdl2;

use sdl2::audio::{AudioCallback, AudioSpecDesired};
use std::sync::mpsc;

const N_SAMPLES_BUF: i32 = 44100 / 4;
const ACTIVE_THRESHOLD: i16 = i16::max_value() / 4;

struct Recording {
    record_buffer: Vec<i16>,
    pos: usize,
    tx: mpsc::Sender<Vec<i16>>,
}

impl AudioCallback for Recording {
    type Channel = i16;

    fn callback(&mut self, input: &mut [i16]) {
        for x in input {
            self.record_buffer[self.pos] = *x;
            self.pos = (self.pos + 1) % self.record_buffer.len();

            if self.pos == 0 {
                self.tx.send(self.record_buffer.clone())
                    .expect("could not send record buffer");
            }
        }
    }
}

fn main() -> Result<(), String> {
    let (tx, rx) = mpsc::channel();
    let sdl_context = sdl2::init()?;
    let audio_subsystem = sdl_context.audio()?;
    let desired_spec = AudioSpecDesired {
        freq: None,
        channels: None,
        samples: None
    };

    let capture_device = audio_subsystem.open_capture(None, &desired_spec, |spec| {
        println!("Capture Spec = {:?}", spec);
        Recording {
            record_buffer: vec![0; N_SAMPLES_BUF as usize],
            pos: 0,
            tx,
        }
    })?;

    println!("AudioDriver: {:?}", capture_device.subsystem().current_audio_driver());
    capture_device.resume();

    loop {
        let recorded_vec = rx.recv().map_err(|e| e.to_string())?;
        let average: i16 = (recorded_vec.iter().fold(0, |x: i32, &y| x + y.abs() as i32) / N_SAMPLES_BUF) as i16;

        if average > ACTIVE_THRESHOLD {
            print!("{}\n", average);
        }
    }
}
