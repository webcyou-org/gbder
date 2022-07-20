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

    // Checks branch condition
    fn cc(&self, idx: u8) -> bool {
        match idx {
            0 => !self.f_z(),
            1 => self.f_z(),
            2 => !self.f_c(),
            3 => self.f_c(),
            _ => panic!("Invalid branch condition index: {}", idx),
        }
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

    // NOP: No operation.
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

    // DAA: Decimal adjust register A
    fn daa(&mut self) {
        let mut a = self.a;

        if !self.f_n() {
            if self.f_c() || a > 0x99 {
                a = a.wrapping_add(0x60);
                self.set_f_c(true);
            }
            if self.f_h() || a & 0x0f > 0x09 {
                a = a.wrapping_add(0x06);
            }
        } else {
            if self.f_c() {
                a = a.wrapping_sub(0x60);
            }
            if self.f_h() {
                a = a.wrapping_sub(0x06);
            }
        }

        self.a = a;

        self.set_f_z(a == 0);
        self.set_f_h(false);
    }

    // CPL: Complement A
    fn cpl(&mut self) {
        self.a = !self.a;
        self.set_f_n(true);
        self.set_f_h(true);
    }

    // CCF: Complement carry flag
    fn ccf(&mut self) {
        self.set_f_n(false);
        self.set_f_h(false);

        let c = self.f_c();
        self.set_f_c(!c);
    }

    // SCF: Set carry flag
    fn scf(&mut self) {
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(true);
    }    
    
    fn _add(&mut self, val: u8) {
        let half_carry = (self.a & 0xf) + (val & 0xf) > 0xf;
        let (res, carry) = self.a.overflowing_add(val);

        self.a = res;

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(half_carry);
        self.set_f_c(carry);
    }

    // ADD r8
    fn add_r8(&mut self, reg: u8) {
        let val = self.read_r8(reg);
        self._add(val);
    }

    // ADC r8
    fn adc_r8(&mut self, reg: u8) {
        let val = self.read_r8(reg);
        self._adc(val);
    }

    // SUB r8
    fn sub_r8(&mut self, reg: u8) {
        let val = self.read_r8(reg);
        self._sub(val);
    }

    // SBC r8
    fn sbc_r8(&mut self, reg: u8) {
        let val = self.read_r8(reg);
        self._sbc(val);
    }

    // ADD d8
    fn add_d8(&mut self) {
        let val = self.read_d8();
        self._add(val);
    }

    fn _sub(&mut self, val: u8) {
        let half_carry = (self.a & 0xf) < (val & 0xf);
        let (res, carry) = self.a.overflowing_sub(val);

        self.a = res;

        self.set_f_z(res == 0);
        self.set_f_n(true);
        self.set_f_h(half_carry);
        self.set_f_c(carry);
    }

    // SUB d8
    fn sub_d8(&mut self) {
        let val = self.read_d8();
        self._sub(val);
    }

    fn _adc(&mut self, val: u8) {
        let c = if self.f_c() { 1 } else { 0 };

        let res = self.a.wrapping_add(val).wrapping_add(c);
        let half_carry = (self.a & 0xf) + (val & 0xf) + c > 0xf;
        let carry = (self.a as u16) + (val as u16) + (c as u16) > 0xff;

        self.a = res;

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(half_carry);
        self.set_f_c(carry);
    }

    // ADC d8
    fn adc_d8(&mut self) {
        let val = self.read_d8();
        self._adc(val);
    }

    fn _sbc(&mut self, val: u8) {
        let c = if self.f_c() { 1 } else { 0 };

        let res = self.a.wrapping_sub(val).wrapping_sub(c);
        let half_carry = (self.a & 0xf) < (val & 0xf) + c;
        let carry = (self.a as u16) < (val as u16) + (c as u16);

        self.a = res;

        self.set_f_z(res == 0);
        self.set_f_n(true);
        self.set_f_h(half_carry);
        self.set_f_c(carry);
    }

    // SBC d8
    fn sbc_d8(&mut self) {
        let val = self.read_d8();
        self._sbc(val);
    }

    // AND d8
    fn and_d8(&mut self) {
        let val = self.read_d8();
        let res = self.a & val;

        self.a = res;

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(true);
        self.set_f_c(false);
    }

    // OR d8
    fn or_d8(&mut self) {
        let val = self.read_d8();
        let res = self.a | val;

        self.a = res;

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(false);
    }    

    // XOR d8
    fn xor_d8(&mut self) {
        let val = self.read_d8();
        let res = self.a ^ val;

        self.a = res;

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(false);
    }

    // CP d8
    fn cp_d8(&mut self) {
        let imm = self.read_d8();
        let a = self.a;

        self.set_f_z(a == imm);
        self.set_f_n(true);
        self.set_f_h(a & 0x0f < imm & 0x0f);
        self.set_f_c(a < imm);
    }

    // LD (HL+), A
    fn ldi_hl_a(&mut self) {
        let addr = self.hl();
        let a = self.a;
        self.write_mem8(addr, a);
        let hl = self.hl();
        self.set_hl(hl.wrapping_add(1));
    }

    // LD (HL-), A
    fn ldd_hl_a(&mut self) {
        let addr = self.hl();
        let a = self.a;
        self.write_mem8(addr, a);
        let hl = self.hl();
        self.set_hl(hl.wrapping_sub(1));
    }

    // LD A, (HL+)
    fn ldi_a_hl(&mut self) {
        let addr = self.hl();
        self.a = self.read_mem8(addr);
        let hl = self.hl();
        self.set_hl(hl.wrapping_add(1));
    }

    // LD A, (HL-)
    fn ldd_a_hl(&mut self) {
        let addr = self.hl();
        self.a = self.read_mem8(addr);
        let hl = self.hl();
        self.set_hl(hl.wrapping_sub(1));
    }

    // LD (BC), A
    fn ld_ind_bc_a(&mut self) {
        let addr = self.bc();
        let a = self.a;
        self.write_mem8(addr, a);
    }

    // LD (DE), A
    fn ld_ind_de_a(&mut self) {
        let addr = self.de();
        let a = self.a;
        self.write_mem8(addr, a);
    }

    // LD A, (BC)
    fn ld_a_ind_bc(&mut self) {
        let bc = self.bc();
        self.a = self.read_mem8(bc);
    }

    // LD A, (DE)
    fn ld_a_ind_de(&mut self) {
        let de = self.de();
        self.a = self.read_mem8(de);
    }

    // Test bit
    fn bit(&mut self, pos: u8, reg: u8) {
        let z = (self.read_r8(reg) >> pos & 1) == 0;
        self.set_f_z(z);
        self.set_f_n(false);
        self.set_f_h(true);
    }

    // Set bit
    fn set(&mut self, pos: u8, reg: u8) {
        let val = self.read_r8(reg);
        self.write_r8(reg, val | (1 << pos));
    }

    // Reset bit
    fn res(&mut self, pos: u8, reg: u8) {
        let val = self.read_r8(reg);
        self.write_r8(reg, val & !(1 << pos));
    }

    fn _rl(&mut self, reg: u8) {
        let orig = self.read_r8(reg);
        let res = (orig << 1) | (if self.f_c() { 1 } else { 0 });
        self.write_r8(reg, res);

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(orig >> 7 & 1 == 1);
    }

    // Rotate left through carry
    fn rl(&mut self, reg: u8) {
        self._rl(reg);
    }

    fn _rlc(&mut self, reg: u8) {
        let orig = self.read_r8(reg);
        let res = orig.rotate_left(1);
        self.write_r8(reg, res);

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(orig >> 7 & 1 == 1);
    }

    // Rotate left
    fn rlc(&mut self, reg: u8) {
        self._rlc(reg);
    }

    fn _rr(&mut self, reg: u8) {
        let orig = self.read_r8(reg);
        let res = (orig >> 1) | (if self.f_c() { 1 } else { 0 } << 7);
        self.write_r8(reg, res);

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(orig & 1 == 1);
    }

    // Rotate right through carry
    fn rr(&mut self, reg: u8) {
        self._rr(reg);
    }

    fn _rrc(&mut self, reg: u8) {
        let orig = self.read_r8(reg);
        let res = orig.rotate_right(1);
        self.write_r8(reg, res);

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(orig & 1 == 1);
    }

    // Rotate right
    fn rrc(&mut self, reg: u8) {
        self._rrc(reg);
    }

    // Shift left into carry
    fn sla(&mut self, reg: u8) {
        let orig = self.read_r8(reg);
        let res = orig << 1;
        self.write_r8(reg, res);

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(orig & 0x80 > 0);
    }

    // Shift right into carry
    fn sra(&mut self, reg: u8) {
        let orig = self.read_r8(reg);
        let res = (orig >> 1) | (orig & 0x80);
        self.write_r8(reg, res);

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(orig & 1 > 0);
    }

    // Swap low/hi-nibble
    fn swap(&mut self, reg: u8) {
        let orig = self.read_r8(reg);
        let res = ((orig & 0x0f) << 4) | ((orig & 0xf0) >> 4);
        self.write_r8(reg, res);

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(false);
    }

    // Shift right through carry
    fn srl(&mut self, reg: u8) {
        let orig = self.read_r8(reg);
        let res = orig >> 1;
        self.write_r8(reg, res);

        self.set_f_z(res == 0);
        self.set_f_n(false);
        self.set_f_h(false);
        self.set_f_c(orig & 1 == 1);
    }

    fn _jp(&mut self, addr: u16) {
        self.pc = addr;
        self.cycle += 4;
    }

    fn jp_cc_d8(&mut self, cci: u8) {
        let addr = self.read_d16();
        if self.cc(cci) {
            self._jp(addr);
        }
    }

    // Unconditional jump to d16
    fn jp_d16(&mut self) {
        let address = self.read_d16();
        self._jp(address);
    }

    // Unconditional jump to HL
    fn jp_hl(&mut self) {
        self.pc = self.hl();
    }

    // Jump to pc+d8 if CC
    fn jr_cc_d8(&mut self, cci: u8) {
        let offset = self.read_d8() as i8;
        if self.cc(cci) {
            self._jr(offset);
        }
    }

    fn _jr(&mut self, offset: i8) {
        self.pc = self.pc.wrapping_add(offset as u16);
        self.cycle += 4;
    }

    /// Jump to pc+d8
    fn jr_d8(&mut self) {
        let offset = self.read_d8() as i8;
        self._jr(offset);
    }    

    fn ld_io_d8_a(&mut self) {
        let offset = self.read_d8() as u16;
        let addr = 0xff00 | offset;
        let a = self.a;
    
        self.write_mem8(addr, a);
    }

    fn ld_a_io_d8(&mut self) {
        let offset = self.read_d8() as u16;
        let addr = 0xff00 | offset;

        self.a = self.read_mem8(addr);
    }

    fn ld_io_c_a(&mut self) {
        let addr = 0xff00 | self.c as u16;
        let a = self.a;

        self.write_mem8(addr, a);
    }

    fn ld_a_io_c(&mut self) {
        let addr = 0xff00 | self.c as u16;

        self.a = self.read_mem8(addr);
    }

    // LD r8, d8
    fn ld_r8_d8(&mut self, reg: u8) {
        let imm = self.read_d8();

        self.write_r8(reg, imm);
    }

    // INC r8
    fn inc_r8(&mut self, reg: u8) {
        let orig = self.read_r8(reg);
        let res = orig.wrapping_add(1);
        self.write_r8(reg, res);

        self.set_f_z(res == 0);
        self.set_f_h(orig & 0x0f == 0x0f);
        self.set_f_n(false);
    }

    // DEC r8
    fn dec_r8(&mut self, reg: u8) {
        let orig = self.read_r8(reg);
        let res = orig.wrapping_sub(1);
        self.write_r8(reg, res);

        self.set_f_z(res == 0);
        self.set_f_h(orig & 0x0f == 0x00);
        self.set_f_n(true);
    }

    // LD r8, r8
    fn ld_r8_r8(&mut self, reg1: u8, reg2: u8) {  
        let val = self.read_r8(reg2);
        self.write_r8(reg1, val);
    }

    fn _call(&mut self, addr: u16) {
        self.sp = self.sp.wrapping_sub(2);
        let sp = self.sp;
        let pc = self.pc;

        self.cycle += 4;

        self.write_mem16(sp, pc);
        self.pc = addr;
    }

    // CALL d16
    fn call_d16(&mut self) {
        let addr = self.read_d16();
        self._call(addr);
    }

    // CALL CC, d16
    fn call_cc_d16(&mut self, cci: u8) {
        let addr = self.read_d16();

        if self.cc(cci) {
            self._call(addr);
        }
    }

    fn rst(&mut self, addr: u8) {    
        self._call(addr as u16);
    }

    fn _ret(&mut self) {
        let sp = self.sp;
        self.pc = self.read_mem16(sp);
        self.sp = self.sp.wrapping_add(2);

        self.cycle += 4;
    }

    // RET
    fn ret(&mut self) {    
        self._ret();
    }

    // RET CC
    fn ret_cc(&mut self, cci: u8) {     
        self.cycle += 4;

        if self.cc(cci) {
            self._ret();
        }
    }

    // PUSH BC
    fn push_bc(&mut self) {
        self.sp = self.sp.wrapping_sub(2);
        let val = self.bc();
        let sp = self.sp;

        self.cycle += 4;

        self.write_mem16(sp, val);
    }

    // PUSH DE
    fn push_de(&mut self) {     
        self.sp = self.sp.wrapping_sub(2);
        let val = self.de();
        let sp = self.sp;

        self.cycle += 4;

        self.write_mem16(sp, val);
    }

    // PUSH HL
    fn push_hl(&mut self) {
        self.sp = self.sp.wrapping_sub(2);
        let val = self.hl();
        let sp = self.sp;

        self.cycle += 4;
        self.write_mem16(sp, val);
    }

    // PUSH AF
    fn push_af(&mut self) {      
        self.sp = self.sp.wrapping_sub(2);
        let val = self.af();
        let sp = self.sp;

        self.cycle += 4;

        self.write_mem16(sp, val);
    }

    // POP BC
    fn pop_bc(&mut self) {      
        let sp = self.sp;
        let val = self.read_mem16(sp);
        self.set_bc(val);
        self.sp = self.sp.wrapping_add(2);
    }

    // POP DE
    fn pop_de(&mut self) {
        let sp = self.sp;
        let val = self.read_mem16(sp);
        self.set_de(val);
        self.sp = self.sp.wrapping_add(2);
    }

    // POP HL
    fn pop_hl(&mut self) {    
        let sp = self.sp;
        let val = self.read_mem16(sp);
        self.set_hl(val);
        self.sp = self.sp.wrapping_add(2);
    }

    /// POP AF
    fn pop_af(&mut self) {
        let sp = self.sp;
        // lower nibble of F is always zero
        let val = self.read_mem16(sp) & 0xfff0;
        self.set_af(val);
        self.sp = self.sp.wrapping_add(2);
    }

    fn rlca(&mut self) {
        self._rlc(7);
        self.set_f_z(false);
    }

    fn rla(&mut self) {
        self._rl(7);
        self.set_f_z(false);
    }

    fn rrca(&mut self) {
        self._rrc(7);
        self.set_f_z(false);
    }

    fn rra(&mut self) {
        self._rr(7);
        self.set_f_z(false);
    }

    fn inc_r16(&mut self, reg: u8) {
        let val = self.read_r16(reg);
        self.write_r16(reg, val.wrapping_add(1));

        self.cycle += 4;
    }

    fn dec_r16(&mut self, reg: u8) {
        let val = self.read_r16(reg);
        self.write_r16(reg, val.wrapping_sub(1));

        self.cycle += 4;
    }

    fn ld_ind_d16_a(&mut self) {
        let addr = self.read_d16();
        let a = self.a;
        self.write_mem8(addr, a);
    }

    fn ld_a_ind_d16(&mut self) {
        let addr = self.read_d16();
        self.a = self.read_mem8(addr);
    }

    // Disable interrupt
    fn di(&mut self) {
        self.ime = false;
    }

    // Enable interrupt
    fn ei(&mut self) {
        self.ime = true;
    }

    // Enable interrupt and return
    fn reti(&mut self) {
        self.ime = true;
        self._ret();
    }

    // Prefixed instructions
    fn prefix(&mut self) {
        let opcode = self.read_d8();
        let pos = opcode >> 3 & 0x7;
        let reg = opcode & 0x7;

        match opcode {
            0x00..=0x07 => self.rlc(reg),
            0x08..=0x0f => self.rrc(reg),
            0x10..=0x17 => self.rl(reg),
            0x18..=0x1f => self.rr(reg),
            0x20..=0x27 => self.sla(reg),
            0x28..=0x2f => self.sra(reg),
            0x30..=0x37 => self.swap(reg),
            0x38..=0x3f => self.srl(reg),
            0x40..=0x7f => self.bit(pos, reg),
            0x80..=0xbf => self.res(pos, reg),
            0xc0..=0xff => self.set(pos, reg),
        }
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