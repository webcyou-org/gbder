use crate::mmu::MMU;

pub struct CPU {
    pub mmu: MMU,
    pc: u16,
    sp: u16,
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
}

impl CPU {
    pub fn new(rom_name: &str) -> Self {
        CPU {
            mmu: MMU::new(rom_name),
            pc: 0x100,
            sp: 0,
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
        }
    }

    // AF register
    fn af(&self) -> u16 {
        (self.a as u16) << 8 | self.f as u16
    }

    fn set_af(&mut self, val: u16) {
        self.a = (val >> 8 & 0xff) as u8;
        self.f = (val & 0xff) as u8;
    }

    // BC register
    fn bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    fn set_bc(&mut self, val: u16) {
        self.b = (val >> 8 & 0xff) as u8;
        self.c = (val & 0xff) as u8;
    }

    // DE register
    fn de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }

    fn set_de(&mut self, val: u16) {
        self.d = (val >> 8 & 0xff) as u8;
        self.e = (val & 0xff) as u8;
    }

    // HL register
    fn hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    fn set_hl(&mut self, val: u16) {
        self.h = (val >> 8 & 0xff) as u8;
        self.l = (val & 0xff) as u8;
    }

    // Z flag
    fn set_f_z(&mut self, z: bool) {
        self.f = (self.f & !(1 << 7)) | (u8::from(z) << 7);
    }

    fn f_z(&self) -> bool {
        (self.f >> 7) & 1 == 1
    }

    // N flag
    fn set_f_n(&mut self, n: bool) {
        self.f = (self.f & !(1 << 6)) | (u8::from(n) << 6);
    }

    fn f_n(&self) -> bool {
        (self.f >> 6) & 1 == 1
    }

    // H flag
    fn set_f_h(&mut self, h: bool) {
        self.f = (self.f & !(1 << 5)) | (u8::from(h) << 5);
    }

    fn f_h(&self) -> bool {
        (self.f >> 5) & 1 == 1
    }

    // C flag
    fn set_f_c(&mut self, c: bool) {
        self.f = (self.f & !(1 << 4)) | (u8::from(c) << 4);
    }

    fn f_c(&self) -> bool {
        (self.f >> 4) & 1 == 1
    }

    pub fn step(&mut self) -> u8 {
        self.mmu.update(0);
        0
    }

    pub fn debug(&mut self) {
        // self.set_af(0x10ff);
        // self.set_f_z(true);
        // self.set_f_n(true);
        self.set_f_c(true); 

        println!("af: {:?}", self.af());
        println!("a: {:?}", self.a);
        println!("f: {:?}", self.f);
        println!("b: {:?}", self.b);
        println!("c: {:?}", self.c);
        println!("d: {:?}", self.d);
        println!("e: {:?}", self.e);
        println!("h: {:?}", self.h);
        println!("l: {:?}", self.l);
        println!("pc: {:?}", self.pc);
        println!("sp: {:?}", self.sp);

        println!("flag z: {:?}", self.f_z());
        println!("flag n: {:?}", self.f_n());
        println!("flag c: {:?}", self.f_c());
        println!("flag h: {:?}", self.f_h());

        let tick = self.step();
        println!("tick: {}", tick);
    }
}