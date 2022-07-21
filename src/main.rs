use std::env;

use std::thread;
use std::time;

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
    // let mut path_buf = PathBuf::from(env::args().nth(1).unwrap());
    // let mut fname = path_buf.to_str().unwrap().to_string();
    // println!("{}", path_buf.to_str().unwrap().to_string());
    let mut cpu: CPU = CPU::new(&rom_fname());

    cpu.mmu.cartridge.debug();

    println!("{:?}", cpu.step() as u32);

    'running: loop {
        let now = time::Instant::now();
        let mut elapsed_tick: u32 = 0;

        // Emulate one frame
        while elapsed_tick < 456 * (144 + 10) {
            elapsed_tick += cpu.step() as u32;
        }

        let wait = time::Duration::from_micros(1000000 / 60);
        let elapsed = now.elapsed();

        if wait > elapsed {
            thread::sleep(wait - elapsed);
        }
    }
    // cpu.debug();
    // cpu.debug();
    // cpu.debug();
    // cpu.debug();
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
