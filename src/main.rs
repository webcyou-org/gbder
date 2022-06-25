use std::env;

// use gbder::cartridge::Cartridge;

mod cartridge;
mod bus;
mod mmu;

use mmu::MMU;

fn rom_fname() -> String {
    env::args().nth(1).unwrap()
}

fn main() {
    // let mut path_buf = PathBuf::from(env::args().nth(1).unwrap());
    // let mut fname = path_buf.to_str().unwrap().to_string();
    // println!("{}", path_buf.to_str().unwrap().to_string());

    // let cartridge = Cartridge::new(&rom_fname());
    let mmu: MMU = MMU::new(&rom_fname());

    println!("{}", mmu.cartridge.title_to_string());
    println!("{}", mmu.cartridge.rom_to_string());
    println!("{}", mmu.cartridge.ram_to_string());
    println!("{:?}", mmu.cartridge.destination_code);
    println!("{:?}", mmu.cartridge.cartridge_type);
    println!("{:?}", mmu.cartridge.cartridge_type.as_str());
    println!("{:?}", mmu.cartridge.new_licensee_code);
    println!("{:?}", mmu.cartridge.old_licensee_code);
    println!("{:?}", mmu.cartridge.mask_rom_version_number);
    println!("{:?}", mmu.cartridge.header_checksum);
    println!("{:?}", mmu.cartridge.rom_banks_amount);

}
