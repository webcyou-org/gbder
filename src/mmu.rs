
use crate::cartridge::Cartridge;
use crate::bus::Bus;
use crate::ppu::PPU;
use crate::joypad::Joypad;
use crate::timer::Timer;

// Memory Management Unit
pub struct MMU {
    pub cartridge: Cartridge,
    ram: [u8; 0x2000],
    hram: [u8; 0x7f],
    pub joypad: Joypad,
    timer: Timer,
    pub ppu: PPU,
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
            joypad: Joypad::new(),
            ppu: PPU::new(),
            timer: Timer::new(),
            int_flag: 0,
            int_enable: 0,
        }
    }

    fn do_dma(&mut self, val: u8) {
        if val < 0x80 || 0xdf < val {
            panic!("Invalid DMA source address")
        }

        let src_base = (val as u16) << 8;
        let dst_base = 0xfe00;

        for i in 0..0xa0 {
            let tmp = self.read(src_base | i);
            self.write(dst_base | i, tmp);
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            // ROM
            0x0000..=0x7fff => self.cartridge.write(addr, val),
            // VRAM
            0x8000..=0x9fff => self.ppu.write(addr, val),
            // External RAM
            0xa000..=0xbfff => self.cartridge.write(addr, val),
            // RAM
            0xc000..=0xdfff => self.ram[(addr & 0x1fff) as usize] = val,
            // Echo RAM
            0xe000..=0xfdff => self.ram[((addr - 0x2000) & 0x1fff) as usize] = val,
            // OAM
            0xfe00..=0xfe9f => self.ppu.write(addr, val),
            // Joypad
            0xff00 => self.joypad.write(addr, val),
            // Timer
            0xff04..=0xff07 => self.timer.write(addr, val),
            // Interrupt flag
            0xff0f => self.int_flag = val,
            // PPU
            0xff40..=0xff45 | 0xff47..=0xff4b => self.ppu.write(addr, val),
            // OAM DMA
            0xff46 => self.do_dma(val),
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
            0x8000..=0x9fff => self.ppu.read(addr),
            // External RAM
            0xa000..=0xbfff => self.cartridge.read(addr),
            // RAM
            0xc000..=0xdfff => self.ram[(addr & 0x1fff) as usize],
            // Echo RAM
            0xe000..=0xfdff => self.ram[((addr - 0x2000) & 0x1fff) as usize],
            // OAM
            0xfe00..=0xfe9f => self.ppu.read(addr),
            // Joypad
            0xff00 => self.joypad.read(addr),
            // Timer
            0xff04..=0xff07 => self.timer.read(addr),
            // Interrupt flag
            0xff0f => self.int_flag,
            // PPU
            0xff40..=0xff45 | 0xff47..=0xff4b => self.ppu.read(addr),
            // HRAM
            0xff80..=0xfffe => self.hram[(addr & 0x7f) as usize],
            // Interrupt enable
            0xffff => self.int_enable,
            _ => 0xff,
        }
    }

    pub fn update(&mut self, cycle: u8) {
        self.cartridge.update(cycle);
        self.ppu.update(cycle);
        self.timer.update(cycle);
        self.joypad.update(cycle);

        if self.ppu.irq_vblank {
            self.int_flag |= 0x1;
            self.ppu.irq_vblank = false;
        }

        if self.ppu.irq_lcdc {
            self.int_flag |= 0x2;
            self.ppu.irq_lcdc = false;
        }
        
        if self.timer.irq {
            self.int_flag |= 0x4;
            self.timer.irq = false;
        }

        if self.joypad.irq {
            self.int_flag |= 0x10;
            self.joypad.irq = false;
        }
    }
}