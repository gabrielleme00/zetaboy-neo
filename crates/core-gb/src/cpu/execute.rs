use crate::Bus;
use crate::cpu::Cpu;

macro_rules! chain_mut {
    // self.fn1(self.fn2(bus), bus)
    ($obj:ident . $method:ident ( $obj_inner:ident . $method_inner:ident ( $($inner_args:expr),* ) , $($after:expr),+ $(,)? )) => {{
        let inner_res = $obj_inner.$method_inner($($inner_args),*);
        $obj.$method(inner_res, $($after),+)
    }};

    // self.fn1(self.fn2())
    ($obj:ident . $method:ident ( $obj_inner:ident . $method_inner:ident ( $($inner_args:expr),* ) $(,)? )) => {{
        let inner_res = $obj_inner.$method_inner($($inner_args),*);
        $obj.$method(inner_res)
    }};
}

/// Implementation of the main CPU instruction execution logic.
impl Cpu {
    pub(super) fn execute(&mut self, bus: &mut Bus) {
        let opcode = self.fetch_byte(bus);
        // println!("Fetched opcode: {:02X}, next bytes: {:02X}, {:02X}", opcode, bus.raw_read(self.pc), bus.raw_read(self.pc + 1));

        match opcode {
            0x00 => { /* NOP */ }
            0x01 => chain_mut!(self.set_bc(self.fetch_word(bus))),
            0x02 => self.write_at_bc(self.a, bus),
            0x03 => chain_mut!(self.set_bc(self.inc_word(self.bc(), bus))),
            0x04 => self.b = self.inc(self.b),
            0x05 => self.b = self.dec(self.b),
            0x06 => self.b = self.fetch_byte(bus),
            0x07 => self.rlca(),
            0x08 => chain_mut!(bus.write_word(self.fetch_word(bus), self.sp)),
            0x09 => self.add_hl(self.bc(), bus),
            0x0A => self.a = self.fetch_bc(bus),
            0x0B => chain_mut!(self.set_bc(self.dec_word(self.bc(), bus))),
            0x0C => self.c = self.inc(self.c),
            0x0D => self.c = self.dec(self.c),
            0x0E => self.c = self.fetch_byte(bus),
            0x0F => self.rrca(),
            0x10 => self.stop(bus),
            0x11 => chain_mut!(self.set_de(self.fetch_word(bus))),
            0x12 => self.write_at_de(self.a, bus),
            0x13 => chain_mut!(self.set_de(self.inc_word(self.de(), bus))),
            0x14 => self.d = self.inc(self.d),
            0x15 => self.d = self.dec(self.d),
            0x16 => self.d = self.fetch_byte(bus),
            0x17 => self.rla(),
            0x18 => self.jr(true, bus),
            0x19 => self.add_hl(self.de(), bus),
            0x1A => self.a = self.fetch_de(bus),
            0x1B => chain_mut!(self.set_de(self.dec_word(self.de(), bus))),
            0x1C => self.e = self.inc(self.e),
            0x1D => self.e = self.dec(self.e),
            0x1E => self.e = self.fetch_byte(bus),
            0x1F => self.rra(),
            0x20 => self.jr(!self.flag_z, bus),
            0x21 => chain_mut!(self.set_hl(self.fetch_word(bus))),
            0x22 => self.write_at_hl_inc(self.a, bus),
            0x23 => chain_mut!(self.set_hl(self.inc_word(self.hl(), bus))),
            0x24 => self.h = self.inc(self.h),
            0x25 => self.h = self.dec(self.h),
            0x26 => self.h = self.fetch_byte(bus),
            0x27 => self.daa(),
            0x28 => self.jr(self.flag_z, bus),
            0x29 => self.add_hl(self.hl(), bus),
            0x2A => self.a = self.fetch_hl_inc(bus),
            0x2B => chain_mut!(self.set_hl(self.dec_word(self.hl(), bus))),
            0x2C => self.l = self.inc(self.l),
            0x2D => self.l = self.dec(self.l),
            0x2E => self.l = self.fetch_byte(bus),
            0x2F => self.cpl(),
            0x30 => self.jr(!self.flag_c, bus),
            0x31 => chain_mut!(self.set_sp(self.fetch_word(bus))),
            0x32 => self.write_at_hl_dec(self.a, bus),
            0x33 => chain_mut!(self.set_sp(self.inc_word(self.sp, bus))),
            0x34 => self.inc_at_hl(bus),
            0x35 => self.dec_at_hl(bus),
            0x36 => chain_mut!(self.write_at_hl(self.fetch_byte(bus), bus)),
            0x37 => self.scf(),
            0x38 => self.jr(self.flag_c, bus),
            0x39 => self.add_hl(self.sp, bus),
            0x3A => self.a = self.fetch_hl_dec(bus),
            0x3B => chain_mut!(self.set_sp(self.dec_word(self.sp, bus))),
            0x3C => self.a = self.inc(self.a),
            0x3D => self.a = self.dec(self.a),
            0x3E => self.a = self.fetch_byte(bus),
            0x3F => self.ccf(),
            0x40 => { /* LD B,B */ }
            0x41 => self.b = self.c,
            0x42 => self.b = self.d,
            0x43 => self.b = self.e,
            0x44 => self.b = self.h,
            0x45 => self.b = self.l,
            0x46 => self.b = self.fetch_hl(bus),
            0x47 => self.b = self.a,
            0x48 => self.c = self.b,
            0x49 => { /* LD C,C */ }
            0x4A => self.c = self.d,
            0x4B => self.c = self.e,
            0x4C => self.c = self.h,
            0x4D => self.c = self.l,
            0x4E => self.c = self.fetch_hl(bus),
            0x4F => self.c = self.a,
            0x50 => self.d = self.b,
            0x51 => self.d = self.c,
            0x52 => { /* LD D,D */ }
            0x53 => self.d = self.e,
            0x54 => self.d = self.h,
            0x55 => self.d = self.l,
            0x56 => self.d = self.fetch_hl(bus),
            0x57 => self.d = self.a,
            0x58 => self.e = self.b,
            0x59 => self.e = self.c,
            0x5A => self.e = self.d,
            0x5B => { /* LD E,E */ }
            0x5C => self.e = self.h,
            0x5D => self.e = self.l,
            0x5E => self.e = self.fetch_hl(bus),
            0x5F => self.e = self.a,
            0x60 => self.h = self.b,
            0x61 => self.h = self.c,
            0x62 => self.h = self.d,
            0x63 => self.h = self.e,
            0x64 => { /* LD H,H */ }
            0x65 => self.h = self.l,
            0x66 => self.h = self.fetch_hl(bus),
            0x67 => self.h = self.a,
            0x68 => self.l = self.b,
            0x69 => self.l = self.c,
            0x6A => self.l = self.d,
            0x6B => self.l = self.e,
            0x6C => self.l = self.h,
            0x6D => { /* LD L,L */ }
            0x6E => self.l = self.fetch_hl(bus),
            0x6F => self.l = self.a,
            0x70 => self.write_at_hl(self.b, bus),
            0x71 => self.write_at_hl(self.c, bus),
            0x72 => self.write_at_hl(self.d, bus),
            0x73 => self.write_at_hl(self.e, bus),
            0x74 => self.write_at_hl(self.h, bus),
            0x75 => self.write_at_hl(self.l, bus),
            0x76 => self.halted = true,
            0x77 => self.write_at_hl(self.a, bus),
            0x78 => self.a = self.b,
            0x79 => self.a = self.c,
            0x7A => self.a = self.d,
            0x7B => self.a = self.e,
            0x7C => self.a = self.h,
            0x7D => self.a = self.l,
            0x7E => self.a = self.fetch_hl(bus),
            0x7F => { /* LD A,A */ }
            0x80 => self.add(self.b),
            0x81 => self.add(self.c),
            0x82 => self.add(self.d),
            0x83 => self.add(self.e),
            0x84 => self.add(self.h),
            0x85 => self.add(self.l),
            0x86 => chain_mut!(self.add(self.fetch_hl(bus))),
            0x87 => self.add(self.a),
            0x88 => self.adc(self.b),
            0x89 => self.adc(self.c),
            0x8A => self.adc(self.d),
            0x8B => self.adc(self.e),
            0x8C => self.adc(self.h),
            0x8D => self.adc(self.l),
            0x8E => chain_mut!(self.adc(self.fetch_hl(bus))),
            0x8F => self.adc(self.a),
            0x90 => self.sub(self.b),
            0x91 => self.sub(self.c),
            0x92 => self.sub(self.d),
            0x93 => self.sub(self.e),
            0x94 => self.sub(self.h),
            0x95 => self.sub(self.l),
            0x96 => chain_mut!(self.sub(self.fetch_hl(bus))),
            0x97 => self.sub(self.a),
            0x98 => self.sbc(self.b),
            0x99 => self.sbc(self.c),
            0x9A => self.sbc(self.d),
            0x9B => self.sbc(self.e),
            0x9C => self.sbc(self.h),
            0x9D => self.sbc(self.l),
            0x9E => chain_mut!(self.sbc(self.fetch_hl(bus))),
            0x9F => self.sbc(self.a),
            0xA0 => self.and(self.b),
            0xA1 => self.and(self.c),
            0xA2 => self.and(self.d),
            0xA3 => self.and(self.e),
            0xA4 => self.and(self.h),
            0xA5 => self.and(self.l),
            0xA6 => chain_mut!(self.and(self.fetch_hl(bus))),
            0xA7 => self.and(self.a),
            0xA8 => self.xor(self.b),
            0xA9 => self.xor(self.c),
            0xAA => self.xor(self.d),
            0xAB => self.xor(self.e),
            0xAC => self.xor(self.h),
            0xAD => self.xor(self.l),
            0xAE => chain_mut!(self.xor(self.fetch_hl(bus))),
            0xAF => self.xor(self.a),
            0xB0 => self.or(self.b),
            0xB1 => self.or(self.c),
            0xB2 => self.or(self.d),
            0xB3 => self.or(self.e),
            0xB4 => self.or(self.h),
            0xB5 => self.or(self.l),
            0xB6 => chain_mut!(self.or(self.fetch_hl(bus))),
            0xB7 => self.or(self.a),
            0xB8 => self.cp(self.b),
            0xB9 => self.cp(self.c),
            0xBA => self.cp(self.d),
            0xBB => self.cp(self.e),
            0xBC => self.cp(self.h),
            0xBD => self.cp(self.l),
            0xBE => chain_mut!(self.cp(self.fetch_hl(bus))),
            0xBF => self.cp(self.a),
            0xC0 => self.ret(!self.flag_z, bus),
            0xC1 => chain_mut!(self.set_bc(self.pop(bus))),
            0xC2 => self.jp(!self.flag_z, bus),
            0xC3 => chain_mut!(self.branch(self.fetch_word(bus), bus)),
            0xC4 => self.call(!self.flag_z, bus),
            0xC5 => chain_mut!(self.push_rr(self.bc(), bus)),
            0xC6 => chain_mut!(self.add(self.fetch_byte(bus))),
            0xC7 => self.rst(0x00, bus),
            0xC8 => self.ret(self.flag_z, bus),
            0xC9 => self.ret(true, bus),
            0xCA => self.jp(self.flag_z, bus),
            0xCB => self.execute_cb(bus),
            0xCC => self.call(self.flag_z, bus),
            0xCD => self.call(true, bus),
            0xCE => chain_mut!(self.adc(self.fetch_byte(bus))),
            0xCF => self.rst(0x08, bus),
            0xD0 => self.ret(!self.flag_c, bus),
            0xD1 => chain_mut!(self.set_de(self.pop(bus))),
            0xD2 => self.jp(!self.flag_c, bus),
            0xD4 => self.call(!self.flag_c, bus),
            0xD5 => chain_mut!(self.push_rr(self.de(), bus)),
            0xD6 => chain_mut!(self.sub(self.fetch_byte(bus))),
            0xD7 => self.rst(0x10, bus),
            0xD8 => self.ret(self.flag_c, bus),
            0xD9 => self.reti(bus),
            0xDA => self.jp(self.flag_c, bus),
            0xDC => self.call(self.flag_c, bus),
            0xDE => chain_mut!(self.sbc(self.fetch_byte(bus))),
            0xDF => self.rst(0x18, bus),
            0xE0 => chain_mut!(self.ld_ff00_a(self.fetch_byte(bus), bus)),
            0xE1 => chain_mut!(self.set_hl(self.pop(bus))),
            0xE2 => self.ld_ff00_a(self.c, bus),
            0xE5 => chain_mut!(self.push_rr(self.hl(), bus)),
            0xE6 => chain_mut!(self.and(self.fetch_byte(bus))),
            0xE7 => self.rst(0x20, bus),
            0xE8 => self.add_sp_offset(bus),
            0xE9 => self.pc = self.hl(),
            0xEA => chain_mut!(self.ld_addr_a(self.fetch_word(bus), bus)),
            0xEE => chain_mut!(self.xor(self.fetch_byte(bus))),
            0xEF => self.rst(0x28, bus),
            0xF0 => chain_mut!(self.ld_a_ff00(self.fetch_byte(bus), bus)),
            0xF1 => chain_mut!(self.set_af(self.pop(bus))),
            0xF2 => self.ld_a_ff00(self.c, bus),
            0xF3 => self.ime.disable(),
            0xF5 => chain_mut!(self.push_rr(self.af(), bus)),
            0xF6 => chain_mut!(self.or(self.fetch_byte(bus))),
            0xF7 => self.rst(0x30, bus),
            0xF8 => self.ld_hl_sp_offset(bus),
            0xF9 => chain_mut!(self.set_sp_and_tick(self.hl(), bus)),
            0xFA => chain_mut!(self.ld_a_addr(self.fetch_word(bus), bus)),
            0xFB => self.ime.enable(),
            0xFE => chain_mut!(self.cp(self.fetch_byte(bus))),
            0xFF => self.rst(0x38, bus),
            _ => unimplemented!("Opcode {:02X} not implemented", opcode),
        }
    }

    fn stop(&mut self, bus: &mut Bus) {
        if bus.any_button_pressed() && bus.pending_interrupts() == 0 {
            self.halted = true;
            self.pc = self.pc.wrapping_add(1);
        } else {
            let speed_switch = false; // TODO: Implement speed switch key (CGB)
            if speed_switch {
                // CGB-only feature
                if bus.pending_interrupts() != 0 {
                    if self.ime.is_enabled() {
                        // TODO: Reset DIV and change speed
                    }
                } else {
                    self.halted = true;
                    // TODO: Reset DIV and change speed
                }
            } else {
                if bus.pending_interrupts() != 0 {
                    // TODO: Enter STOP mode and reset DIV
                } else {
                    self.pc = self.pc.wrapping_add(1);
                    // TODO: Enter STOP mode and reset DIV
                }
            }
        }
    }

    fn daa(&mut self) {
        let mut correction = 0;
        if self.flag_h || (!self.flag_n && (self.a & 0xF) > 9) {
            correction |= 0x6;
        }
        if self.flag_c || (!self.flag_n && self.a > 0x99) {
            correction |= 0x60;
            self.flag_c = true;
        }
        if self.flag_n {
            self.a = self.a.wrapping_sub(correction);
        } else {
            self.a = self.a.wrapping_add(correction);
        }
        self.flag_z = self.a == 0;
        self.flag_h = false;
    }

    fn ld_addr_a(&mut self, addr: u16, bus: &mut Bus) {
        bus.write_byte(addr, self.a);
    }

    fn ld_ff00_a(&mut self, offset: u8, bus: &mut Bus) {
        self.ld_addr_a(0xFF00 + offset as u16, bus);
    }

    fn ld_a_addr(&mut self, addr: u16, bus: &mut Bus) {
        self.a = bus.read_byte(addr);
    }

    fn ld_a_ff00(&mut self, offset: u8, bus: &mut Bus) {
        self.ld_a_addr(0xFF00 + offset as u16, bus);
    }

    fn ld_hl_sp_offset(&mut self, bus: &mut Bus) {
        let offset = self.fetch_byte(bus) as i8;
        let sp = self.sp;
        let result = sp.wrapping_add(offset as u16);
        self.set_hl(result);
        self.flag_z = false;
        self.flag_n = false;
        self.flag_h = ((sp & 0xF) + (offset as u16 & 0xF)) > 0xF;
        self.flag_c = ((sp & 0xFF) + (offset as u16 & 0xFF)) > 0xFF;
    }

    fn add(&mut self, value: u8) {
        let (result, carry) = self.a.overflowing_add(value);
        let a = self.a;
        self.a = result;
        self.flag_z = self.a == 0;
        self.flag_n = false;
        self.flag_h = ((a & 0xF) + (value & 0xF)) > 0xF;
        self.flag_c = carry;
    }

    fn add_hl(&mut self, value: u16, bus: &mut Bus) {
        let hl = self.hl();
        let (result, carry) = hl.overflowing_add(value);
        self.set_hl(result);
        self.flag_n = false;
        self.flag_h = ((hl & 0x0FFF) + (value & 0x0FFF)) > 0x0FFF;
        self.flag_c = carry;
        bus.tick_m_cycle();
    }

    fn add_sp_offset(&mut self, bus: &mut Bus) {
        let offset = self.fetch_byte(bus) as i8;
        let sp = self.sp;
        let result = sp.wrapping_add(offset as u16);
        self.sp = result;
        self.flag_z = false;
        self.flag_n = false;
        self.flag_h = ((sp & 0xF) + (offset as u16 & 0xF)) > 0xF;
        self.flag_c = ((sp & 0xFF) + (offset as u16 & 0xFF)) > 0xFF;
        bus.tick_m_cycle(); // Probably writes to SP:lower
        bus.tick_m_cycle(); // Probably writes to SP:upper
    }

    fn adc(&mut self, value: u8) {
        let carry_in = if self.flag_c { 1 } else { 0 };
        let (intermediate, carry1) = self.a.overflowing_add(value);
        let (result, carry2) = intermediate.overflowing_add(carry_in);
        let a = self.a;
        self.a = result;
        self.flag_z = self.a == 0;
        self.flag_n = false;
        self.flag_h = ((a & 0xF) + (value & 0xF) + carry_in) > 0xF;
        self.flag_c = carry1 || carry2;
    }

    fn sub(&mut self, value: u8) {
        let (result, carry) = self.a.overflowing_sub(value);
        let a = self.a;
        self.a = result;
        self.flag_z = self.a == 0;
        self.flag_n = true;
        self.flag_h = ((a & 0xF) as i8 - (value & 0xF) as i8) < 0;
        self.flag_c = carry;
    }

    fn sbc(&mut self, value: u8) {
        let carry_in = if self.flag_c { 1 } else { 0 };
        let (intermediate, carry1) = self.a.overflowing_sub(value);
        let (result, carry2) = intermediate.overflowing_sub(carry_in);
        let a = self.a;
        self.a = result;
        self.flag_z = self.a == 0;
        self.flag_n = true;
        self.flag_h = ((a & 0xF) as i8 - (value & 0xF) as i8 - carry_in as i8) < 0;
        self.flag_c = carry1 || carry2;
    }

    fn inc(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        self.flag_z = result == 0;
        self.flag_n = false;
        self.flag_h = (value & 0x0F) == 0x0F;
        result
    }

    fn inc_word(&mut self, value: u16, bus: &mut Bus) -> u16 {
        bus.tick_m_cycle();
        value.wrapping_add(1)
    }

    fn inc_at_hl(&mut self, bus: &mut Bus) {
        let hl = self.hl();
        let value = bus.read_byte(hl);
        let result = self.inc(value);
        bus.write_byte(hl, result);
    }

    fn dec(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        self.flag_z = result == 0;
        self.flag_n = true;
        self.flag_h = (value & 0x0F) == 0x00;
        result
    }

    fn dec_word(&mut self, value: u16, bus: &mut Bus) -> u16 {
        bus.tick_m_cycle();
        value.wrapping_sub(1)
    }

    fn dec_at_hl(&mut self, bus: &mut Bus) {
        let hl = self.hl();
        let value = bus.read_byte(hl);
        let result = self.dec(value);
        bus.write_byte(hl, result);
    }

    fn push_rr(&mut self, value: u16, bus: &mut Bus) {
        bus.tick_m_cycle();
        self.push(value, bus);
    }

    fn branch(&mut self, value: u16, bus: &mut Bus) {
        self.pc = value;
        bus.tick_m_cycle();
    }

    fn call(&mut self, condition: bool, bus: &mut Bus) {
        let addr = self.fetch_word(bus);
        if condition {
            self.push(self.pc, bus);
            self.branch(addr, bus);
        }
    }

    fn ret(&mut self, condition: bool, bus: &mut Bus) {
        bus.tick_m_cycle();
        if condition {
            let addr = self.pop(bus);
            self.branch(addr, bus);
        }
    }

    fn reti(&mut self, bus: &mut Bus) {
        // In hardware, this would execute EI then RET, enabling interrupts after
        //the return.
        let addr = self.pop(bus);
        self.branch(addr, bus);
        self.ime.enable_now();
    }

    fn rst(&mut self, address: u16, bus: &mut Bus) {
        self.push(self.pc, bus);
        self.branch(address, bus);
    }

    fn jp(&mut self, condition: bool, bus: &mut Bus) {
        let addr = self.fetch_word(bus);
        if condition {
            self.branch(addr, bus);
        }
    }

    fn jr(&mut self, condition: bool, bus: &mut Bus) {
        let offset = self.fetch_byte(bus) as i8;
        if condition {
            let pc = self.pc.wrapping_add(offset as u16);
            self.branch(pc, bus);
        }
    }

    fn and(&mut self, value: u8) {
        self.a &= value;
        self.flag_z = self.a == 0;
        self.flag_n = false;
        self.flag_h = true;
        self.flag_c = false;
    }

    fn xor(&mut self, value: u8) {
        self.a ^= value;
        self.flag_z = self.a == 0;
        self.flag_n = false;
        self.flag_h = false;
        self.flag_c = false;
    }

    fn or(&mut self, value: u8) {
        self.a |= value;
        self.flag_z = self.a == 0;
        self.flag_n = false;
        self.flag_h = false;
        self.flag_c = false;
    }

    fn cp(&mut self, value: u8) {
        let result = self.a.wrapping_sub(value);
        self.flag_z = result == 0;
        self.flag_n = true;
        self.flag_h = ((self.a & 0xF) as i8 - (value & 0xF) as i8) < 0;
        self.flag_c = self.a < value;
    }

    fn ccf(&mut self) {
        self.flag_n = false;
        self.flag_h = false;
        self.flag_c = !self.flag_c;
    }

    fn scf(&mut self) {
        self.flag_n = false;
        self.flag_h = false;
        self.flag_c = true;
    }

    fn cpl(&mut self) {
        self.a = !self.a;
        self.flag_n = true;
        self.flag_h = true;
    }

    fn rlca(&mut self) {
        let bit_7 = (self.a >> 7) & 1;
        self.a = (self.a << 1) | bit_7;
        self.flag_z = false;
        self.flag_n = false;
        self.flag_h = false;
        self.flag_c = bit_7 == 1;
    }

    fn rla(&mut self) {
        let old_carry = self.flag_c as u8;
        let new_carry = (self.a >> 7) & 1;
        self.a = (self.a << 1) | old_carry;
        self.flag_z = false;
        self.flag_n = false;
        self.flag_h = false;
        self.flag_c = new_carry == 1;
    }

    fn rrca(&mut self) {
        let bit_0 = self.a & 1;
        self.a = (self.a >> 1) | (bit_0 << 7);
        self.flag_z = false;
        self.flag_n = false;
        self.flag_h = false;
        self.flag_c = bit_0 == 1;
    }

    pub fn rra(&mut self) {
        let old_carry = self.flag_c as u8;
        let new_carry = self.a & 1;
        self.a = (self.a >> 1) | (old_carry << 7);
        self.flag_z = false;
        self.flag_n = false;
        self.flag_h = false;
        self.flag_c = new_carry == 1;
    }
}

/// Implementation of CB-prefixed instructions for the CPU.
impl Cpu {
    fn execute_cb(&mut self, bus: &mut Bus) {
        let opcode = self.fetch_byte(bus);
        match opcode {
            0x00 => self.b = self.rlc(self.b),
            0x01 => self.c = self.rlc(self.c),
            0x02 => self.d = self.rlc(self.d),
            0x03 => self.e = self.rlc(self.e),
            0x04 => self.h = self.rlc(self.h),
            0x05 => self.l = self.rlc(self.l),
            0x06 => self.write_at_hl_with(|cpu, value| cpu.rlc(value), bus),
            0x07 => self.a = self.rlc(self.a),
            0x08 => self.b = self.rrc(self.b),
            0x09 => self.c = self.rrc(self.c),
            0x0A => self.d = self.rrc(self.d),
            0x0B => self.e = self.rrc(self.e),
            0x0C => self.h = self.rrc(self.h),
            0x0D => self.l = self.rrc(self.l),
            0x0E => self.write_at_hl_with(|cpu, value| cpu.rrc(value), bus),
            0x0F => self.a = self.rrc(self.a),
            0x10 => self.b = self.rl(self.b),
            0x11 => self.c = self.rl(self.c),
            0x12 => self.d = self.rl(self.d),
            0x13 => self.e = self.rl(self.e),
            0x14 => self.h = self.rl(self.h),
            0x15 => self.l = self.rl(self.l),
            0x16 => self.write_at_hl_with(|cpu, value| cpu.rl(value), bus),
            0x17 => self.a = self.rl(self.a),
            0x18 => self.b = self.rr(self.b),
            0x19 => self.c = self.rr(self.c),
            0x1A => self.d = self.rr(self.d),
            0x1B => self.e = self.rr(self.e),
            0x1C => self.h = self.rr(self.h),
            0x1D => self.l = self.rr(self.l),
            0x1E => self.write_at_hl_with(|cpu, value| cpu.rr(value), bus),
            0x1F => self.a = self.rr(self.a),
            0x20 => self.b = self.sla(self.b),
            0x21 => self.c = self.sla(self.c),
            0x22 => self.d = self.sla(self.d),
            0x23 => self.e = self.sla(self.e),
            0x24 => self.h = self.sla(self.h),
            0x25 => self.l = self.sla(self.l),
            0x26 => self.write_at_hl_with(|cpu, value| cpu.sla(value), bus),
            0x27 => self.a = self.sla(self.a),
            0x28 => self.b = self.sra(self.b),
            0x29 => self.c = self.sra(self.c),
            0x2A => self.d = self.sra(self.d),
            0x2B => self.e = self.sra(self.e),
            0x2C => self.h = self.sra(self.h),
            0x2D => self.l = self.sra(self.l),
            0x2E => self.write_at_hl_with(|cpu, value| cpu.sra(value), bus),
            0x2F => self.a = self.sra(self.a),
            0x30 => self.b = self.swap(self.b),
            0x31 => self.c = self.swap(self.c),
            0x32 => self.d = self.swap(self.d),
            0x33 => self.e = self.swap(self.e),
            0x34 => self.h = self.swap(self.h),
            0x35 => self.l = self.swap(self.l),
            0x36 => self.write_at_hl_with(|cpu, value| cpu.swap(value), bus),
            0x37 => self.a = self.swap(self.a),
            0x38 => self.b = self.srl(self.b),
            0x39 => self.c = self.srl(self.c),
            0x3A => self.d = self.srl(self.d),
            0x3B => self.e = self.srl(self.e),
            0x3C => self.h = self.srl(self.h),
            0x3D => self.l = self.srl(self.l),
            0x3E => self.write_at_hl_with(|cpu, value| cpu.srl(value), bus),
            0x3F => self.a = self.srl(self.a),
            0x40 => self.b = self.bit(0, self.b),
            0x41 => self.c = self.bit(0, self.c),
            0x42 => self.d = self.bit(0, self.d),
            0x43 => self.e = self.bit(0, self.e),
            0x44 => self.h = self.bit(0, self.h),
            0x45 => self.l = self.bit(0, self.l),
            0x46 => self.bit_hl(0, bus),
            0x47 => self.a = self.bit(0, self.a),
            0x48 => self.b = self.bit(1, self.b),
            0x49 => self.c = self.bit(1, self.c),
            0x4A => self.d = self.bit(1, self.d),
            0x4B => self.e = self.bit(1, self.e),
            0x4C => self.h = self.bit(1, self.h),
            0x4D => self.l = self.bit(1, self.l),
            0x4E => self.bit_hl(1, bus),
            0x4F => self.a = self.bit(1, self.a),
            0x50 => self.b = self.bit(2, self.b),
            0x51 => self.c = self.bit(2, self.c),
            0x52 => self.d = self.bit(2, self.d),
            0x53 => self.e = self.bit(2, self.e),
            0x54 => self.h = self.bit(2, self.h),
            0x55 => self.l = self.bit(2, self.l),
            0x56 => self.bit_hl(2, bus),
            0x57 => self.a = self.bit(2, self.a),
            0x58 => self.b = self.bit(3, self.b),
            0x59 => self.c = self.bit(3, self.c),
            0x5A => self.d = self.bit(3, self.d),
            0x5B => self.e = self.bit(3, self.e),
            0x5C => self.h = self.bit(3, self.h),
            0x5D => self.l = self.bit(3, self.l),
            0x5E => self.bit_hl(3, bus),
            0x5F => self.a = self.bit(3, self.a),
            0x60 => self.b = self.bit(4, self.b),
            0x61 => self.c = self.bit(4, self.c),
            0x62 => self.d = self.bit(4, self.d),
            0x63 => self.e = self.bit(4, self.e),
            0x64 => self.h = self.bit(4, self.h),
            0x65 => self.l = self.bit(4, self.l),
            0x66 => self.bit_hl(4, bus),
            0x67 => self.a = self.bit(4, self.a),
            0x68 => self.b = self.bit(5, self.b),
            0x69 => self.c = self.bit(5, self.c),
            0x6A => self.d = self.bit(5, self.d),
            0x6B => self.e = self.bit(5, self.e),
            0x6C => self.h = self.bit(5, self.h),
            0x6D => self.l = self.bit(5, self.l),
            0x6E => self.bit_hl(5, bus),
            0x6F => self.a = self.bit(5, self.a),
            0x70 => self.b = self.bit(6, self.b),
            0x71 => self.c = self.bit(6, self.c),
            0x72 => self.d = self.bit(6, self.d),
            0x73 => self.e = self.bit(6, self.e),
            0x74 => self.h = self.bit(6, self.h),
            0x75 => self.l = self.bit(6, self.l),
            0x76 => self.bit_hl(6, bus),
            0x77 => self.a = self.bit(6, self.a),
            0x78 => self.b = self.bit(7, self.b),
            0x79 => self.c = self.bit(7, self.c),
            0x7A => self.d = self.bit(7, self.d),
            0x7B => self.e = self.bit(7, self.e),
            0x7C => self.h = self.bit(7, self.h),
            0x7D => self.l = self.bit(7, self.l),
            0x7E => self.bit_hl(7, bus),
            0x7F => self.a = self.bit(7, self.a),
            0x80 => self.b = self.res(0, self.b),
            0x81 => self.c = self.res(0, self.c),
            0x82 => self.d = self.res(0, self.d),
            0x83 => self.e = self.res(0, self.e),
            0x84 => self.h = self.res(0, self.h),
            0x85 => self.l = self.res(0, self.l),
            0x86 => self.write_at_hl_with(|cpu, value| cpu.res(0, value), bus),
            0x87 => self.a = self.res(0, self.a),
            0x88 => self.b = self.res(1, self.b),
            0x89 => self.c = self.res(1, self.c),
            0x8A => self.d = self.res(1, self.d),
            0x8B => self.e = self.res(1, self.e),
            0x8C => self.h = self.res(1, self.h),
            0x8D => self.l = self.res(1, self.l),
            0x8E => self.write_at_hl_with(|cpu, value| cpu.res(1, value), bus),
            0x8F => self.a = self.res(1, self.a),
            0x90 => self.b = self.res(2, self.b),
            0x91 => self.c = self.res(2, self.c),
            0x92 => self.d = self.res(2, self.d),
            0x93 => self.e = self.res(2, self.e),
            0x94 => self.h = self.res(2, self.h),
            0x95 => self.l = self.res(2, self.l),
            0x96 => self.write_at_hl_with(|cpu, value| cpu.res(2, value), bus),
            0x97 => self.a = self.res(2, self.a),
            0x98 => self.b = self.res(3, self.b),
            0x99 => self.c = self.res(3, self.c),
            0x9A => self.d = self.res(3, self.d),
            0x9B => self.e = self.res(3, self.e),
            0x9C => self.h = self.res(3, self.h),
            0x9D => self.l = self.res(3, self.l),
            0x9E => self.write_at_hl_with(|cpu, value| cpu.res(3, value), bus),
            0x9F => self.a = self.res(3, self.a),
            0xA0 => self.b = self.res(4, self.b),
            0xA1 => self.c = self.res(4, self.c),
            0xA2 => self.d = self.res(4, self.d),
            0xA3 => self.e = self.res(4, self.e),
            0xA4 => self.h = self.res(4, self.h),
            0xA5 => self.l = self.res(4, self.l),
            0xA6 => self.write_at_hl_with(|cpu, value| cpu.res(4, value), bus),
            0xA7 => self.a = self.res(4, self.a),
            0xA8 => self.b = self.res(5, self.b),
            0xA9 => self.c = self.res(5, self.c),
            0xAA => self.d = self.res(5, self.d),
            0xAB => self.e = self.res(5, self.e),
            0xAC => self.h = self.res(5, self.h),
            0xAD => self.l = self.res(5, self.l),
            0xAE => self.write_at_hl_with(|cpu, value| cpu.res(5, value), bus),
            0xAF => self.a = self.res(5, self.a),
            0xB0 => self.b = self.res(6, self.b),
            0xB1 => self.c = self.res(6, self.c),
            0xB2 => self.d = self.res(6, self.d),
            0xB3 => self.e = self.res(6, self.e),
            0xB4 => self.h = self.res(6, self.h),
            0xB5 => self.l = self.res(6, self.l),
            0xB6 => self.write_at_hl_with(|cpu, value| cpu.res(6, value), bus),
            0xB7 => self.a = self.res(6, self.a),
            0xB8 => self.b = self.res(7, self.b),
            0xB9 => self.c = self.res(7, self.c),
            0xBA => self.d = self.res(7, self.d),
            0xBB => self.e = self.res(7, self.e),
            0xBC => self.h = self.res(7, self.h),
            0xBD => self.l = self.res(7, self.l),
            0xBE => self.write_at_hl_with(|cpu, value| cpu.res(7, value), bus),
            0xBF => self.a = self.res(7, self.a),
            0xC0 => self.b = self.set(0, self.b),
            0xC1 => self.c = self.set(0, self.c),
            0xC2 => self.d = self.set(0, self.d),
            0xC3 => self.e = self.set(0, self.e),
            0xC4 => self.h = self.set(0, self.h),
            0xC5 => self.l = self.set(0, self.l),
            0xC6 => self.write_at_hl_with(|cpu, value| cpu.set(0, value), bus),
            0xC7 => self.a = self.set(0, self.a),
            0xC8 => self.b = self.set(1, self.b),
            0xC9 => self.c = self.set(1, self.c),
            0xCA => self.d = self.set(1, self.d),
            0xCB => self.e = self.set(1, self.e),
            0xCC => self.h = self.set(1, self.h),
            0xCD => self.l = self.set(1, self.l),
            0xCE => self.write_at_hl_with(|cpu, value| cpu.set(1, value), bus),
            0xCF => self.a = self.set(1, self.a),
            0xD0 => self.b = self.set(2, self.b),
            0xD1 => self.c = self.set(2, self.c),
            0xD2 => self.d = self.set(2, self.d),
            0xD3 => self.e = self.set(2, self.e),
            0xD4 => self.h = self.set(2, self.h),
            0xD5 => self.l = self.set(2, self.l),
            0xD6 => self.write_at_hl_with(|cpu, value| cpu.set(2, value), bus),
            0xD7 => self.a = self.set(2, self.a),
            0xD8 => self.b = self.set(3, self.b),
            0xD9 => self.c = self.set(3, self.c),
            0xDA => self.d = self.set(3, self.d),
            0xDB => self.e = self.set(3, self.e),
            0xDC => self.h = self.set(3, self.h),
            0xDD => self.l = self.set(3, self.l),
            0xDE => self.write_at_hl_with(|cpu, value| cpu.set(3, value), bus),
            0xDF => self.a = self.set(3, self.a),
            0xE0 => self.b = self.set(4, self.b),
            0xE1 => self.c = self.set(4, self.c),
            0xE2 => self.d = self.set(4, self.d),
            0xE3 => self.e = self.set(4, self.e),
            0xE4 => self.h = self.set(4, self.h),
            0xE5 => self.l = self.set(4, self.l),
            0xE6 => self.write_at_hl_with(|cpu, value| cpu.set(4, value), bus),
            0xE7 => self.a = self.set(4, self.a),
            0xE8 => self.b = self.set(5, self.b),
            0xE9 => self.c = self.set(5, self.c),
            0xEA => self.d = self.set(5, self.d),
            0xEB => self.e = self.set(5, self.e),
            0xEC => self.h = self.set(5, self.h),
            0xED => self.l = self.set(5, self.l),
            0xEE => self.write_at_hl_with(|cpu, value| cpu.set(5, value), bus),
            0xEF => self.a = self.set(5, self.a),
            0xF0 => self.b = self.set(6, self.b),
            0xF1 => self.c = self.set(6, self.c),
            0xF2 => self.d = self.set(6, self.d),
            0xF3 => self.e = self.set(6, self.e),
            0xF4 => self.h = self.set(6, self.h),
            0xF5 => self.l = self.set(6, self.l),
            0xF6 => self.write_at_hl_with(|cpu, value| cpu.set(6, value), bus),
            0xF7 => self.a = self.set(6, self.a),
            0xF8 => self.b = self.set(7, self.b),
            0xF9 => self.c = self.set(7, self.c),
            0xFA => self.d = self.set(7, self.d),
            0xFB => self.e = self.set(7, self.e),
            0xFC => self.h = self.set(7, self.h),
            0xFD => self.l = self.set(7, self.l),
            0xFE => self.write_at_hl_with(|cpu, value| cpu.set(7, value), bus),
            0xFF => self.a = self.set(7, self.a),
        }
    }

    /// Writes a value to the memory location pointed to by the HL register.
    /// The value is obtained by applying the provided closure `f` to the current CPU state and the value at HL.
    /// The closure should return the new value to be written.
    fn write_at_hl_with<F>(&mut self, f: F, bus: &mut Bus)
    where
        F: FnOnce(&mut Self, u8) -> u8,
    {
        let hl = self.hl();
        let value = bus.read_byte(hl);
        let result = f(self, value);
        bus.write_byte(hl, result);
    }

    /// Performs a rotate left circular (RLC) operation on the given value.
    /// Updates the CPU flags accordingly.
    /// Returns the result of the operation.
    fn rlc(&mut self, value: u8) -> u8 {
        let result = (value << 1) | (value >> 7);
        self.flag_z = result == 0;
        self.flag_n = false;
        self.flag_h = false;
        self.flag_c = (value & 0x80) != 0;
        result
    }

    /// Performs a rotate right circular (RRC) operation on the given value.
    /// Updates the CPU flags accordingly.
    /// Returns the result of the operation.
    fn rrc(&mut self, value: u8) -> u8 {
        let result = (value >> 1) | (value << 7);
        self.flag_z = result == 0;
        self.flag_n = false;
        self.flag_h = false;
        self.flag_c = (value & 0x01) != 0;
        result
    }

    /// Performs a rotate left (RL) operation on the given value.
    /// Updates the CPU flags accordingly.
    /// Returns the result of the operation.
    fn rl(&mut self, value: u8) -> u8 {
        let result = (value << 1) | (if self.flag_c { 1 } else { 0 });
        self.flag_z = result == 0;
        self.flag_n = false;
        self.flag_h = false;
        self.flag_c = (value & 0x80) != 0;
        result
    }

    /// Performs a rotate right (RR) operation on the given value.
    /// Updates the CPU flags accordingly.
    /// Returns the result of the operation.
    fn rr(&mut self, value: u8) -> u8 {
        let result = (value >> 1) | (if self.flag_c { 0x80 } else { 0 });
        self.flag_z = result == 0;
        self.flag_n = false;
        self.flag_h = false;
        self.flag_c = (value & 0x01) != 0;
        result
    }

    /// Tests if the specified bit in the given value is set.
    /// Updates the CPU flags accordingly.
    /// Returns the original value.
    fn bit(&mut self, bit: u8, value: u8) -> u8 {
        self.flag_z = (value & (1 << bit)) == 0;
        self.flag_n = false;
        self.flag_h = true;
        value
    }

    /// Tests if the specified bit in the memory location pointed to by HL is set.
    /// Updates the CPU flags accordingly.
    fn bit_hl(&mut self, bit: u8, bus: &mut Bus) {
        let hl = self.hl();
        let value = bus.read_byte(hl);
        self.bit(bit, value);
    }

    /// Performs a shift left arithmetic (SLA) operation on the given value.
    /// Updates the CPU flags accordingly.
    /// Returns the result of the operation.
    fn sla(&mut self, value: u8) -> u8 {
        let result = value << 1;
        self.flag_z = result == 0;
        self.flag_n = false;
        self.flag_h = false;
        self.flag_c = (value & 0x80) != 0;
        result
    }

    /// Performs a shift right arithmetic (SRA) operation on the given value.
    /// Updates the CPU flags accordingly.
    /// Returns the result of the operation.
    fn sra(&mut self, value: u8) -> u8 {
        let result = (value >> 1) | (value & 0x80);
        self.flag_z = result == 0;
        self.flag_n = false;
        self.flag_h = false;
        self.flag_c = (value & 0x01) != 0;
        result
    }

    /// Swaps the upper and lower nibbles of the given value
    /// and updates the CPU flags accordingly.
    fn swap(&mut self, value: u8) -> u8 {
        let result = (value >> 4) | (value << 4);
        self.flag_z = result == 0;
        self.flag_n = false;
        self.flag_h = false;
        self.flag_c = false;
        result
    }

    /// Performs a shift right logical (SRL) operation on the given value.
    /// Updates the CPU flags accordingly.
    /// Returns the result of the operation.
    fn srl(&mut self, value: u8) -> u8 {
        let result = value >> 1;
        self.flag_z = result == 0;
        self.flag_n = false;
        self.flag_h = false;
        self.flag_c = (value & 0x01) != 0;
        result
    }

    /// Set bit in value to 0.
    fn res(&mut self, bit: u8, value: u8) -> u8 {
        value & !(1 << bit)
    }

    /// Sets the specified bit in the given value to 1.
    fn set(&mut self, bit: u8, value: u8) -> u8 {
        value | (1 << bit)
    }
}
