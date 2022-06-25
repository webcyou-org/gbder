use std::env;

use gbder::cartridge::Cartridge;

fn rom_fname() -> String {
    env::args().nth(1).unwrap()
}

fn main() {
    // let mut path_buf = PathBuf::from(env::args().nth(1).unwrap());
    // let mut fname = path_buf.to_str().unwrap().to_string();
    // println!("{}", path_buf.to_str().unwrap().to_string());

    let cartridge = Cartridge::new(&rom_fname());

    println!("{}", cartridge.title_to_string());
    println!("{}", cartridge.rom_to_string());
    println!("{}", cartridge.ram_to_string());
    println!("{:?}", cartridge.destination_code);
    println!("{:?}", cartridge.cartridge_type);
    println!("{:?}", cartridge.cartridge_type.as_str());
    println!("{:?}", cartridge.new_licensee_code);
    println!("{:?}", cartridge.old_licensee_code);
    println!("{:?}", cartridge.mask_rom_version_number);
    println!("{:?}", cartridge.header_checksum);
    println!("{:?}", cartridge.rom_banks_amount);

}
