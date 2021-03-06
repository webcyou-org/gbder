use std::fs::File;
use std::io::{Read};
// use std::path::PathBuf;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use std::str;

use crate::bus::Bus;

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
    Mbc3TimerBattery = 0x0f,
    Mbc3TimerRamBattery = 0x10,
    Mbc5 = 0x19,
    Mbc5Ram = 0x1a,
    Mbc5RamBattery = 0x1b,
    Mbc5Rumble = 0x1c,
    Mbc5RumbleRam = 0x1d,
    Mbc5RumbleRamBattery = 0x1e,
    Mbc6 = 0x20,
    Mbc7SensorRumbleRamBattery = 0x22,
    PocketCamera = 0xfc,
    BandaiTama5 = 0xfd,
    HuC3 = 0xfe,
    HuC1RamBattery = 0xff,
}

impl CartridgeType {
    pub fn as_str(&self) -> &'static str {
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
            CartridgeType::Mbc3TimerBattery => "MBC3+TIMER+BATTERY",
            CartridgeType::Mbc3TimerRamBattery => "MBC3+TIMER+RAM+BATTERY",
            CartridgeType::Mbc5 => "MBC5",
            CartridgeType::Mbc5Ram => "MBC5+RAM",
            CartridgeType::Mbc5RamBattery => "MBC5+RAM+BATTERY",
            CartridgeType::Mbc5Rumble => "MBC5+RUMBLE",
            CartridgeType::Mbc5RumbleRam => "MBC5+RUMBLE+RAM",
            CartridgeType::Mbc5RumbleRamBattery => "MBC5+RUMBLE+RAM+BATTERY",
            CartridgeType::Mbc6 => "MBC6",
            CartridgeType::Mbc7SensorRumbleRamBattery => "MBC7+SENSOR+RUMBLE+RAM+BATTERY",
            CartridgeType::PocketCamera => "POCKET CAMERA",
            CartridgeType::BandaiTama5 => "BANDAI TAMA5",
            CartridgeType::HuC3 => "HuC3",
            CartridgeType::HuC1RamBattery => "HuC1+RAM+BATTERY",
            //     _ => "Unknown",
        }
    }
}

pub struct Cartridge {
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,
    pub entry_point: Vec<u8>,
    pub logo: Vec<u8>,
    pub title: Vec<u8>,
    pub new_licensee_code: Vec<u8>,
    pub sgb_flag: bool,
    pub cartridge_type: CartridgeType,
    pub rom_size: usize,
    pub ram_size: usize,
    pub destination_code: DestinationCode,
    pub old_licensee_code: u8,
    pub mask_rom_version_number: u8,
    pub header_checksum: u8,
    pub rom_banks_amount: u8,
    // pub global_checksum: [u8; 2],
    pub ram_enable: bool,
    pub bank_no_upper: u8,
    pub bank_no_lower: u8,
    pub mode: bool,
}

impl Cartridge {
    pub fn new(fname: &str) -> Self {    
        let mut rom = Vec::new();
        let mut file = File::open(fname).unwrap();
        file.read_to_end(&mut rom).unwrap();
        
        Cartridge {            
            entry_point: Cartridge::entry_point(&rom),
            logo: Cartridge::logo(&rom),
            title: Cartridge::title(&rom),
            new_licensee_code: Cartridge::new_licensee_code(&rom),
            sgb_flag: Cartridge::sgb_flag(&rom),
            cartridge_type: Cartridge::cartridge_type(&rom),
            rom_size: Cartridge::rom_size(&rom),
            ram_size: Cartridge::ram_size(&rom),
            destination_code: Cartridge::destination_code(&rom),
            old_licensee_code: Cartridge::old_licensee_code(&rom),
            mask_rom_version_number: Cartridge::mask_rom_version_number(&rom),
            header_checksum: Cartridge::header_checksum(&rom),
            rom_banks_amount: Cartridge::rom_banks_amount(&rom),
            ram: vec![0; Cartridge::ram_size(&rom)],
            rom: rom,
            ram_enable: false,
            bank_no_upper: 0,
            bank_no_lower: 0,
            mode: false,
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
    fn new_licensee_code(rom: &Vec<u8>) -> Vec<u8> {
        let mut new_licensee_code = Vec::new();
        for i in 0x0144..0x0145 {
            new_licensee_code.push(rom[i]);
        };
        new_licensee_code
    }    

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
    fn old_licensee_code(rom: &Vec<u8>) -> u8 {
        rom[0x014B]
    }
    
    // 014C - Mask ROM Version number
    fn mask_rom_version_number(rom: &Vec<u8>) -> u8 {
        rom[0x014C]
    }

    // 014D - Header Checksum
    fn header_checksum(rom: &Vec<u8>) -> u8 {
        let mut checksum: u8 = 0;
        for i in 0x0134..0x014d {
            checksum = checksum.wrapping_sub(rom[i]).wrapping_sub(1);
        }
        if checksum != rom[0x014d] {
            panic!("ROM header checksum is incorrect");
        }
        checksum
    }

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

    fn rom_banks_amount(rom: &Vec<u8>) -> u8 {
        2 << rom[0x0148]
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

    fn rom_bank_no(&self) -> u8 {
        let bank_no = if self.mode {
            self.bank_no_lower
        } else {
            self.bank_no_upper << 5 | self.bank_no_lower
        };

        let bank_no = match bank_no {
            0 | 0x20 | 0x40 | 0x60 => bank_no + 1,
            _ => bank_no,
        };

        bank_no & (self.rom_banks_amount - 1)
    }

    fn ram_bank_no(&self) -> u8 {
        if self.mode {
            self.bank_no_upper
        } else {
            0
        }
    }

    pub fn read_save_file(&mut self, fname: &str) {
        if let Ok(mut file) = File::open(fname) {
            self.ram = Vec::new();
            file.read_to_end(&mut self.ram).unwrap();
        }
    }    

    pub fn debug(&mut self) {
        println!("{}", self.title_to_string());
        println!("{}", self.rom_to_string());
        println!("{}", self.ram_to_string());
        println!("{:?}", self.destination_code);
        println!("{:?}", self.cartridge_type);
        println!("{:?}", self.cartridge_type.as_str());
        println!("{:?}", self.new_licensee_code);
        println!("{:?}", self.old_licensee_code);
        println!("{:?}", self.mask_rom_version_number);
        println!("{:?}", self.header_checksum);
        println!("{:?}", self.rom_banks_amount);
    }
}

impl Bus for Cartridge {
    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            // RAM enable
            0x0000..=0x1fff => self.ram_enable = val & 0x0f == 0x0a,
            // ROM bank number (lower 5 bits)
            0x2000..=0x3fff => self.bank_no_lower = val & 0x1f,
            // RAM bank number or ROM bank number (upper 2 bits)
            0x4000..=0x5fff => self.bank_no_upper = val & 0x03,
            // ROM/RAM mode select
            0x6000..=0x7fff => self.mode = val & 0x01 > 0,
            // RAM bank 00-03
            0xa000..=0xbfff => {
                if !self.ram_enable {
                    return;
                }
                let offset = (8 * 1024) * self.ram_bank_no() as usize;
                self.ram[(addr & 0x1fff) as usize + offset] = val
            }
            _ => unreachable!("Unexpected address: 0x{:04x}", addr),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            // ROM bank 00
            0x0000..=0x3fff => self.rom[addr as usize],
            // ROM bank 01-7f
            0x4000..=0x7fff => {
                let offset = (16 * 1024) * self.rom_bank_no() as usize;
                self.rom[(addr & 0x3fff) as usize + offset]
            }
            // RAM bank 00-03
            0xa000..=0xbfff => {
                if !self.ram_enable {
                    return 0xff;
                }
                let offset = (8 * 1024) * self.ram_bank_no() as usize;
                self.ram[(addr & 0x1fff) as usize + offset]
            }
            _ => unreachable!("Unexpected address: 0x{:04x}", addr),
        }
    }

    fn update(&mut self, _tick: u8) {}
}