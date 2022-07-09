use std::env;

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

    // let cartridge = Cartridge::new(&rom_fname());
    // let mmu: MMU = MMU::new(&rom_fname());
    let mut cpu: CPU = CPU::new(&rom_fname());

    println!("{}", cpu.mmu.cartridge.title_to_string());
    println!("{}", cpu.mmu.cartridge.rom_to_string());
    println!("{}", cpu.mmu.cartridge.ram_to_string());
    println!("{:?}", cpu.mmu.cartridge.destination_code);
    println!("{:?}", cpu.mmu.cartridge.cartridge_type);
    println!("{:?}", cpu.mmu.cartridge.cartridge_type.as_str());
    println!("{:?}", cpu.mmu.cartridge.new_licensee_code);
    println!("{:?}", cpu.mmu.cartridge.old_licensee_code);
    println!("{:?}", cpu.mmu.cartridge.mask_rom_version_number);
    println!("{:?}", cpu.mmu.cartridge.header_checksum);
    println!("{:?}", cpu.mmu.cartridge.rom_banks_amount);

    cpu.debug();
}
