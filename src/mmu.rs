
use crate::cartridge::Cartridge;
use crate::bus::Bus;
// Memory Management Unit
pub struct MMU {
    pub cartridge: Cartridge,
    ram: [u8; 0x2000],
    hram: [u8; 0x7f],
    /// Interrupt flag
    pub int_flag: u8,
    /// Interrupt enable
    pub int_enable: u8,
}

impl MMU {
    pub fn new(rom_name: &str) -> Self {
        MMU {
            cartridge: Cartridge::new(rom_name),
            ram: [0; 0x2000],
            hram: [0; 0x7f],
            int_flag: 0,
            int_enable: 0,
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            // ROM
            0x0000..=0x7fff => self.cartridge.write(addr, val),
            // VRAM
            // External RAM
            0xa000..=0xbfff => self.cartridge.write(addr, val),
            // RAM
            0xc000..=0xdfff => self.ram[(addr & 0x1fff) as usize] = val,
            // Echo RAM
            0xe000..=0xfdff => self.ram[((addr - 0x2000) & 0x1fff) as usize] = val,
            // OAM
            // Joypad
            // Timer
            // Interrupt flag
            0xff0f => self.int_flag = val,
            // PPU
            // OAM DMA
            // HRAM
            0xff80..=0xfffe => self.hram[(addr & 0x7f) as usize] = val,
            // Interrupt enable
            0xffff => self.int_enable = val,
            _ => (),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            // ROM
            0x0000..=0x7fff => self.cartridge.read(addr),
            // VRAM
            // External RAM
            0xa000..=0xbfff => self.cartridge.read(addr),
            // RAM
            0xc000..=0xdfff => self.ram[(addr & 0x1fff) as usize],
            // Echo RAM
            0xe000..=0xfdff => self.ram[((addr - 0x2000) & 0x1fff) as usize],
            // OAM
            // Joypad
            // Timer
            // Interrupt flag
            0xff0f => self.int_flag,
            // PPU
            // HRAM
            0xff80..=0xfffe => self.hram[(addr & 0x7f) as usize],
            // Interrupt enable
            0xffff => self.int_enable,
            _ => 0xff,
        }
    }

    pub fn update(&mut self, cycle: u8) {
        self.cartridge.update(cycle);
    }
}