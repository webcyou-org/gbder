use std::env;

extern crate sdl2;

use std::thread;
use std::time;

use sdl2::pixels::PixelFormatEnum;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

// use gbder::cartridge::Cartridge;

mod cartridge;
mod bus;
mod mmu;
mod cpu;

// use mmu::MMU;
use cpu::CPU;

fn rom_fname() -> String {
    env::args().nth(1).unwrap()
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("GBdeR", 320, 288)
        .position_centered()
        .build()
        .unwrap();
    
    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 160, 144)
        .unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    texture
        .with_lock(None, |buf: &mut [u8], pitch: usize| {
        })
        .unwrap();

    canvas.clear();
    canvas.copy(&texture, None, None).unwrap();
    canvas.present();
    

    // let mut path_buf = PathBuf::from(env::args().nth(1).unwrap());
    // let mut fname = path_buf.to_str().unwrap().to_string();
    // println!("{}", path_buf.to_str().unwrap().to_string());
    let mut cpu: CPU = CPU::new(&rom_fname());

    cpu.mmu.cartridge.debug();

    // println!("{:?}", cpu.step() as u32);

    let mut i = 0;
    'running: loop {
         i = (i + 1) % 255;
         canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
         canvas.clear();
         for event in event_pump.poll_iter() {
             match event {
                 Event::Quit {..} |
                 Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                     break 'running
                 },
                 _ => {}
             }
         }
         // The rest of the game loop goes here...
         canvas.present();
         ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
     }

    // 'running: loop {
    //     let now = time::Instant::now();
    //     let mut elapsed_tick: u32 = 0;

    //     // Emulate one frame
    //     while elapsed_tick < 456 * (144 + 10) {
    //         elapsed_tick += cpu.step() as u32;
    //     }

    //     let wait = time::Duration::from_micros(1000000 / 60);
    //     let elapsed = now.elapsed();

    //     if wait > elapsed {
    //         thread::sleep(wait - elapsed);
    //     }
    // }
    // cpu.debug();

    // standby(&mut cpu);
}

fn standby(cpu: &mut CPU) {
    println!("step to n key press");

    let mut word = String::new();
    std::io::stdin().read_line(&mut word).ok();
    let answer = word.trim().to_string();

    match &*answer {
        "n" => {
            println!("------------cpu step debug----------------");
            cpu.debug();
            standby(cpu);
        },
        _ => println!("------------cpu step debug end----------------"),
    }
}
