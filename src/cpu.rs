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
    ime: bool, // IME - 割り込み有効フラグ (Interrupt Master Enable Flag)
    cycle: u8,
    halted: bool,
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
            ime: false,
            cycle: 0,
            halted: false,
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
        let mut total_cycle = 0;
        self.cycle = 0;

        if self.halted {
            self.cycle += 4;
        } else {
            self.fetch_and_exec();
        }

        total_cycle += self.cycle;

        self.mmu.update(self.cycle);

        if self.ime {
            self.cycle = 0;
            // self.check_irqs();
            self.mmu.update(self.cycle);
            total_cycle += self.cycle;
        }        
        total_cycle
    }

    // 8-bit operand
    fn write_r8(&mut self, idx: u8, val: u8) {
        match idx {
            0 => self.b = val,
            1 => self.c = val,
            2 => self.d = val,
            3 => self.e = val,
            4 => self.h = val,
            5 => self.l = val,
            6 => {
                let hl = self.hl();
                self.write_mem8(hl, val);
            }
            7 => self.a = val,
            _ => panic!("Invalid operand index: {}", idx),
        }
    }

    fn read_r8(&mut self, idx: u8) -> u8 {
        match idx {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => {
                let hl = self.hl();
                self.read_mem8(hl)
            }
            7 => self.a,
            _ => panic!("Invalid operand index: {}", idx),
        }
    }

    // 16-bit operand
    fn write_r16(&mut self, idx: u8, val: u16) {
        match idx {
            0 => self.set_bc(val),
            1 => self.set_de(val),
            2 => self.set_hl(val),
            3 => self.sp = val,
            _ => panic!("Invalid operand index: {}", idx),
        }
    }

    fn read_r16(&mut self, idx: u8) -> u16 {
        match idx {
            0 => self.bc(),
            1 => self.de(),
            2 => self.hl(),
            3 => self.sp,
            _ => panic!("Invalid operand index: {}", idx),
        }
    }

    // 8-bit immediate memory
    fn read_d8(&mut self) -> u8 {
        let pc = self.pc;
        let imm = self.read_mem8(pc);
        self.pc = self.pc.wrapping_add(1);

        imm
    }

    // 16-bit immediate memory
    fn read_d16(&mut self) -> u16 {
        let pc = self.pc;
        let imm = self.read_mem16(pc);
        self.pc = self.pc.wrapping_add(2);

        imm
    }

    // 8-bit value memory
    fn write_mem8(&mut self, addr: u16, val: u8) {
        self.mmu.write(addr, val);
        self.cycle += 4;
    }

    fn read_mem8(&mut self, addr: u16) -> u8 {
        let ret = self.mmu.read(addr);
        self.cycle += 4;
        ret
    }

    // 16-bit value memory
    fn write_mem16(&mut self, addr: u16, val: u16) {
        self.write_mem8(addr, (val & 0xff) as u8);
        self.write_mem8(addr.wrapping_add(1), (val >> 8) as u8);
    }

    fn read_mem16(&mut self, addr: u16) -> u16 {
        let lo = self.read_mem8(addr);
        let hi = self.read_mem8(addr.wrapping_add(1));

        (hi as u16) << 8 | lo as u16
    }

    // NOP
    fn nop(&mut self) {}

    // LD r16, d16
    fn ld_r16_d16(&mut self, reg: u8) {
        let val = self.read_d16();    
        self.write_r16(reg, val);
        println!("ld_r16_d16");
    }

    // LD (d16), SP
    fn ld_ind_d16_sp(&mut self) {
        let addr = self.read_d16();
        self.write_mem16(addr, self.sp);
    }

    // LD SP, HL
    fn ld_sp_hl(&mut self) {
        self.cycle += 4;
        self.sp = self.hl();
    }

    // ADD HL, r16
    fn add_hl_r16(&mut self, reg: u8) {
        let hl = self.hl();
        let val = self.read_r16(reg);

        let half_carry = (hl & 0xfff) + (val & 0xfff) > 0xfff;
        let (res, carry) = hl.overflowing_add(val);
        self.set_hl(res);

        self.cycle += 4;

        self.set_f_n(false);
        self.set_f_h(half_carry);
        self.set_f_c(carry);
    }

    fn _add_sp(&mut self, offset: i8) -> u16 {
        let val = offset as u16;

        let half_carry = (self.sp & 0x0f) + (val & 0x0f) > 0x0f;
        let carry = (self.sp & 0xff) + (val & 0xff) > 0xff;

        self.set_f_z(false);
        self.set_f_n(false);
        self.set_f_h(half_carry);
        self.set_f_c(carry);

        self.sp.wrapping_add(val)
    }

    // ADD SP, d8
    fn add_sp_d8(&mut self) {
        let val = self.read_d8() as i8;
        self.sp = self._add_sp(val);
        self.cycle += 8;
    }

    // LD HL, SP+d8
    fn ld_hl_sp_d8(&mut self) {
        let offset = self.read_d8() as i8;
        self.cycle += 4;
        let res = self._add_sp(offset);
        self.set_hl(res);
    }

    // AND r8
    fn and_r8(&mut self, reg: u8) {
        let res = self.a & self.read_r8(reg);

        self.a = res;

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(true);
        self.set_f_c(false);
    }

    // OR r8
    fn or_r8(&mut self, reg: u8) {
        let res = self.a | self.read_r8(reg);

        self.a = res;

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(false);
    }

    // XOR r8
    fn xor_r8(&mut self, reg: u8) {
        let res = self.a ^ self.read_r8(reg);

        self.a = res;

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(false);
    }    

    // CP r8
    fn cp_r8(&mut self, reg: u8) {
        let a = self.a;
        let val = self.read_r8(reg);

        self.set_f_z(a == val);
        self.set_f_n(true);
        self.set_f_h(a & 0x0f < val & 0x0f);
        self.set_f_c(a < val);
    }

    fn fetch_and_exec(&mut self) {
        let opcode = self.read_d8();
        let reg = opcode & 7;
        let reg2 = opcode >> 3 & 7;

        println!("opcode: {:?}", opcode);
        println!("reg: {:?}", reg);
        println!("reg2: {:?}", reg2);

        match opcode {
            // NOP
            0x00 => self.nop(),
            // LD r16, d16
            0x01 | 0x11 | 0x21 | 0x31 => self.ld_r16_d16(opcode >> 4),
            _ => println!("Unimplemented opcode"),
            // _ => panic!("Unimplemented opcode 0x{:x}", opcode),
        }
    }

    fn halt(&mut self) {
        self.halted = true;
    }

    pub fn debug(&mut self) {
        // self.set_af(0x10ff);
        // self.set_f_z(true);
        // self.set_f_n(true);
        // self.set_f_c(true); 

        self.step();

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

        // let tick = self.step();
        // println!("tick: {}", tick);
    }
}