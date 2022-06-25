
use crate::cartridge::Cartridge;

// Memory Management Unit
pub struct MMU {
    pub cartridge: Cartridge,
    ram: [u8; 0x2000],
    hram: [u8; 0x7f],
}

impl MMU {
    /// Creates a new `MMU`.
    pub fn new(rom_name: &str) -> Self {
        MMU {
            cartridge: Cartridge::new(rom_name),
            ram: [0; 0x2000],
            hram: [0; 0x7f],
        }
    }
}