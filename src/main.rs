use std::fs::File;
use std::io::{Read};
use std::env;
// use std::path::PathBuf;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use std::str;

#[derive(FromPrimitive, Debug)]
pub enum DestinationCode {
    Japanese = 0x00,
    NonJapanese = 0x01,
    Unknown = 0xFF,
}

#[derive(FromPrimitive, Debug)]
pub enum CartridgeType {
    RomOnly = 0x00,
    Mbc1 = 0x01,
    Mbc1Ram = 0x02,
    Mbc1RamBattery = 0x03,
    Mbc2 = 0x05,
    Mbc2Battery = 0x06,
    RomRam = 0x08,
    RomRamBattery = 0x09,
    Mmm01 = 0x0b,
    Mmm01Ram = 0x0c,
    Mmm01RamBattery = 0x0d,
    Mbc3 = 0x11,
    Mbc3Ram = 0x12,
    Mbc3RamBattery = 0x13,
}

impl CartridgeType {
    fn as_str(&self) -> &'static str {
        match self {
            CartridgeType::RomOnly => "ROM ONLY",
            CartridgeType::Mbc1 => "MBC1",
            CartridgeType::Mbc1Ram => "MBC1+RAM",
            CartridgeType::Mbc1RamBattery => "MBC1+RAM+BATTERY",
            CartridgeType::Mbc2 => "MBC2",
            CartridgeType::Mbc2Battery => "MBC2+BATTERY",
            CartridgeType::RomRam => "ROM+RAM",
            CartridgeType::RomRamBattery => "ROM+RAM+BATTERY",
            CartridgeType::Mmm01 => "MMM01",
            CartridgeType::Mmm01Ram => "MMM01+RAM",
            CartridgeType::Mmm01RamBattery => "MMM01+RAM+BATTERY",
            CartridgeType::Mbc3 => "MBC3",
            CartridgeType::Mbc3Ram => "MBC3+RAM",
            CartridgeType::Mbc3RamBattery => "MBC3+RAM+BATTERY",
            //     0x0f => "MBC3+TIMER+BATTERY",
            //     0x10 => "MBC3+TIMER+RAM+BATTERY",
            //     0x19 => "MBC5",
            //     0x1a => "MBC5+RAM",
            //     0x1b => "MBC5+RAM+BATTERY",
            //     0x1c => "MBC5+RUMBLE",
            //     0x1d => "MBC5+RUMBLE+RAM",
            //     0x1e => "MBC5+RUMBLE+RAM+BATTERY",
            //     0x20 => "MBC6",
            //     0x22 => "MBC7+SENSOR+RUMBLE+RAM+BATTERY",
            //     0xfc => "POCKET CAMERA",
            //     0xfd => "BANDAI TAMA5",
            //     0xfe => "HuC3",
            //     0xff => "HuC1+RAM+BATTERY",
            //     _ => "Unknown",
        }
    }
}

pub struct Cartridge {
    pub entry_point: Vec<u8>,
    pub logo: Vec<u8>,
    pub title: Vec<u8>,
    // pub new_licensee_code: [u8; 2],
    pub sgb_flag: bool,
    pub cartridge_type: CartridgeType,
    pub rom_size: usize,
    pub ram_size: usize,
    pub destination_code: DestinationCode,
    // pub old_licensee_code: u8,
    // pub mask_rom_version_number: u8,
    // pub header_checksum: u8,
    // pub global_checksum: [u8; 2],
    // pub data: Vec<u8>,
}

impl Cartridge {
    pub fn new(fname: &str) -> Self {    
        let mut rom = Vec::new();
        let mut file = File::open(fname).unwrap();
        file.read_to_end(&mut rom).unwrap();

        // 0147 - Cartridge Type
        // let rrom = FromPrimitive::from_u8(rom[0x0147]);
        
        Cartridge {
            entry_point: Cartridge::entry_point(&rom),
            logo: Cartridge::logo(&rom),
            sgb_flag: Cartridge::sgb_flag(&rom),
            cartridge_type: Cartridge::cartridge_type(&rom),
            title: Cartridge::title(&rom),
            rom_size: Cartridge::rom_size(&rom),
            ram_size: Cartridge::ram_size(&rom),
            destination_code: Cartridge::destination_code(&rom),
        }
    }

    // 0100-0103 - Entry Point
    fn entry_point(rom: &Vec<u8>) -> Vec<u8> {
        let mut entry_point = Vec::new();
        for i in 0x0100..0x0103 {
            entry_point.push(rom[i]);
        };
        entry_point
    }

    // 0104-0133 - Nintendo Logo
    fn logo(rom: &Vec<u8>) -> Vec<u8> {
        let mut logo = Vec::new();
        for i in 0x0104..0x0133 {
            logo.push(rom[i]);
        };
        logo
    }

    // 013F-0142 - Manufacturer Code
    // 0143 - CGB Flag
    // 0144-0145 - New Licensee Code
    // 0146 - SGB Flag
    fn sgb_flag(rom: &Vec<u8>) -> bool {
        0x03 == rom[0x0146]
    }

    // 0147 - Cartridge Type
    fn cartridge_type(rom: &Vec<u8>) -> CartridgeType {
        if let Some(cartridge_type) = FromPrimitive::from_u8(rom[0x0147]) {
            cartridge_type
        } else {
            CartridgeType::RomOnly
        }
    }

    // 014A - Destination Code
    fn destination_code(rom: &Vec<u8>) -> DestinationCode {
        if let Some(destination_code) = FromPrimitive::from_u8(rom[0x014A]) {
            destination_code
        } else {
            DestinationCode::Unknown
        }
    }

    // 014B - Old Licensee Code
    // 014C - Mask ROM Version number
    // 014D - Header Checksum
    // 014E-014F - Global Checksum

    // 0134-0143 - Title
    fn title(rom: &Vec<u8>) -> Vec<u8> {
        let mut title = Vec::new();
        for i in 0x0134..0x0143 {
            title.push(rom[i]);
        };
        title
    }
    
    pub fn title_to_string(&self) -> String {
        String::from_utf8(self.title.to_vec()).unwrap()
    }
    
    // 0148 - ROM Size
    fn rom_size(rom: &Vec<u8>) -> usize {
        match rom[0x0148] {
            0 => 32 * 1024,
            n => 32 * 1024 << (n as usize),
        }
    }

    pub fn rom_to_string(&self) -> String {
        format!("ROM size {}KB", self.rom_size / 1024)
    }

    // 0149 - RAM Size
    fn ram_size(rom: &Vec<u8>) -> usize {
        match rom[0x0149] {
            0 => 0,
            1 => 2 * 1024,
            2 => 8 * 1024,
            3 => 32 * 1024,
            4 => 128 * 1024,
            5 => 64 * 1024,
            _ => panic!("RAM size invalid"),
        }
    }

    pub fn ram_to_string(&self) -> String {
        format!("RAM size {}KB", self.ram_size / 1024)
    }
}

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
}
