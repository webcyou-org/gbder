use std::env;
// use std::path::PathBuf;

extern crate sdl2;

use std::thread;
use std::time;

use sdl2::Sdl;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Canvas;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
// use sdl2::sys::Window;
// use std::time::Duration;

mod cartridge;
mod bus;
mod mmu;
mod ppu;
mod cpu;

// use mmu::MMU;
use cpu::CPU;

fn rom_fname() -> String {
    env::args().nth(1).unwrap()
}

// Returns save filename for current ROM.
// fn save_fname() -> String {
//     let mut path_buf = PathBuf::from(rom_fname());
//     path_buf.set_extension("sav");
//     path_buf.to_str().unwrap().to_string()
// }

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

// fn standby(cpu: &mut CPU) {
//     println!("step to n key press");

//     let mut word = String::new();
//     std::io::stdin().read_line(&mut word).ok();
//     let answer = word.trim().to_string();

//     match &*answer {
//         "n" => {
//             println!("------------cpu step debug----------------");
//             cpu.debug();
//             standby(cpu);
//         },
//         _ => println!("------------cpu step debug end----------------"),
//     }
// }
