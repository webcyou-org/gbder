use std::env;

extern crate sdl2;

use std::thread;
use std::time;

use sdl2::Sdl;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Canvas;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod cartridge;
mod bus;
mod joypad;
mod mmu;
mod ppu;
mod cpu;
mod timer;

use cpu::CPU;

fn translate_keycode(key: Keycode) -> Option<joypad::Key> {
    match key {
        Keycode::Down => Some(joypad::Key::Down),
        Keycode::Up => Some(joypad::Key::Up),
        Keycode::Left => Some(joypad::Key::Left),
        Keycode::Right => Some(joypad::Key::Right),
        Keycode::Return => Some(joypad::Key::Start),
        Keycode::RShift => Some(joypad::Key::Select),
        Keycode::X => Some(joypad::Key::A),
        Keycode::Z => Some(joypad::Key::B),
        _ => None,
    }
}

// Handles key down event.
fn handle_keydown(cpu: &mut cpu::CPU, key: Keycode) {
    translate_keycode(key).map(|k| cpu.mmu.joypad.keydown(k));
}

// Handles key up event.
fn handle_keyup(cpu: &mut cpu::CPU, key: Keycode) {
    translate_keycode(key).map(|k| cpu.mmu.joypad.keyup(k));
}

fn rom_fname() -> String {
    env::args().nth(1).unwrap()
}

fn sdl_init(sdl_context: &Sdl) -> Canvas<sdl2::video::Window>  {
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("GBdeR", 320, 288)
        .position_centered()
        .build()
        .unwrap();
    window.into_canvas().build().unwrap()
}

fn main() {
    let sdl_context = sdl2::init().unwrap();    
    let mut canvas = sdl_init(&sdl_context);
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 160, 144)
        .unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    canvas.clear();
    canvas.copy(&texture, None, None).unwrap();
    canvas.present(); 
   
    let mut cpu: CPU = CPU::new(&rom_fname());

    'running: loop {
        let now = time::Instant::now();
        let mut elapsed_tick: u32 = 0;

        // Emulate one frame
        while elapsed_tick < 456 * (144 + 10) {
            elapsed_tick += cpu.step() as u32;
        }        

        texture
            .with_lock(None, |buf: &mut [u8], pitch: usize| {
                let fb = cpu.mmu.ppu.frame_buffer();

                for y in 0..144 {
                    for x in 0..160 {
                        let offset = y * pitch + x * 3;
                        let color = fb[y * 160 + x];

                        buf[offset] = color;
                        buf[offset + 1] = color;
                        buf[offset + 2] = color;
                    }
                }
            })
            .unwrap();

        canvas.clear();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => handle_keydown(&mut cpu, keycode),
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => handle_keyup(&mut cpu, keycode),
                _ => {}
            }
        }

        let wait = time::Duration::from_micros(1000000 / 60);
        let elapsed = now.elapsed();
        if wait > elapsed {
            thread::sleep(wait - elapsed);
        }
     }
}
