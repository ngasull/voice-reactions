extern crate ffmpeg;
extern crate sdl2;

use anyhow::{anyhow, Result};
use ffmpeg::util::format::pixel::Pixel;
use ffmpeg::util::frame::video::Video;
use ffmpeg::software::converter;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::sync::mpsc;

const N_SAMPLES_BUF: i32 = 44100 / 10;
const ACTIVE_THRESHOLD: i16 = i16::max_value() / 4;
const INACTIVE_N_THRESHOLD: i16 = 5;

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

fn record(audio_subsystem: &sdl2::AudioSubsystem, activity_tx: mpsc::Sender::<bool>) -> anyhow::Result<sdl2::audio::AudioDevice<Recording>> {
    let (tx, rx) = mpsc::channel();
    let mut n_active = 0;
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
    }).map_err(|e| anyhow!(e))?;

    capture_device.resume();

    std::thread::spawn(move || -> anyhow::Result<()> {
        loop {
            let recorded_vec = rx.recv()?;
            let average: i16 = (recorded_vec.iter().fold(0, |x: i32, &y| x + (y as i32).abs()) / N_SAMPLES_BUF) as i16;

            if average > ACTIVE_THRESHOLD {
                n_active = INACTIVE_N_THRESHOLD;
            } else {
                if n_active == 1 {
                    activity_tx.send(false)?;
                }
                n_active -= 1;
            }

            if n_active > 0 {
                activity_tx.send(true)?;
            }
        }
    });

    Ok(capture_device)
}

fn get_frame(
    packets_iter: &mut ffmpeg::format::context::input::PacketIter,
    video_stream_id: &usize,
    video_decoder: &mut ffmpeg::codec::decoder::video::Video
    ) -> Result<ffmpeg::util::frame::video::Video> {
    return loop {
        match packets_iter.next() {
            Some((stream, packet)) if stream.index() == *video_stream_id => {
                let mut raw = Video::empty();
                let res = video_decoder.decode(&packet, &mut raw).and_then(|_| {
                    let mut output = Video::empty();
                    converter((raw.width(), raw.height()), raw.format(), Pixel::RGB24)?
                        .run(&raw, &mut output)
                        .map(|_| output)
                });
                break res.map_err(|e| anyhow!(e));
            },
            None => break Err(anyhow!("No more packet")),
            _ => {
                packets_iter.next();
            },
        }
    }
}

fn main() -> Result<()> {
    let sdl_context = sdl2::init().map_err(|e| anyhow!(e))?;
    let audio_subsystem = sdl_context.audio().map_err(|e| anyhow!(e))?;
    let mut event_pump = sdl_context.event_pump().map_err(|e| anyhow!(e))?;
    let video_subsystem = sdl_context.video().map_err(|e| anyhow!(e))?;
    let (voice_tx, voice_rx) = mpsc::channel();
    let _capture_device = record(&audio_subsystem, voice_tx)?;

    // Streaming with ffmpeg
    ffmpeg::init()?;
    let input_path = std::env::current_dir().unwrap().join("video.mp4");
    let input_str = input_path.to_string_lossy().into_owned();
    //let mut opts = ffmpeg::Dictionary::new();
    let mut input = ffmpeg::format::input(&input_str)?;
    let (video_stream_id, mut video_decoder) = {
        let stream = input
            .streams()
            .best(ffmpeg::util::media::Type::Video)
            .ok_or(anyhow!("Failed to get video stream"))?;
        (stream.index(), stream.codec().decoder().video()?)
    };
    let packets_iter = &mut input.packets();

    let window = video_subsystem.window("rust-sdl2 demo: Video", 800, 600)
        .position_centered()
        .fullscreen_desktop()
        .opengl()
        .build()?;
    let (win_w, win_h) = window.size();
    let target_rect = Some(sdl2::rect::Rect::new(0, 0, win_w, win_h));

    let mut canvas = window.into_canvas().build()?;
    let texture_creator = canvas.texture_creator();

    let first_frame = get_frame(packets_iter, &video_stream_id, &mut video_decoder)?;
    let mut texture = texture_creator.create_texture_streaming(sdl2::pixels::PixelFormatEnum::RGB24, first_frame.width(), first_frame.height())?;
    texture.with_lock(None, |buffer: &mut [u8], _pitch: usize| {
        buffer.copy_from_slice(first_frame.data(0));
    }).unwrap();
    canvas.copy(&texture, None, target_rect).unwrap();
    canvas.present();

    let mut voice = false;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..}
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }

        if let Ok(v) = voice_rx.try_recv() {
            voice = v;
        }

        if voice {
            // Play some frames
            let frame = match get_frame(packets_iter, &video_stream_id, &mut video_decoder) {
                Ok(p) => p,
                Err(msg) => {
                    eprintln!("Error when getting next frame: {}", msg);
                    break;
                },
            };
            texture.with_lock(None, |buffer: &mut [u8], _pitch: usize| {
                buffer.copy_from_slice(frame.data(0));
            }).unwrap();
            canvas.clear();
            canvas.copy(&texture, None, target_rect).unwrap();
            canvas.present();
        }

        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    Ok(())
}
