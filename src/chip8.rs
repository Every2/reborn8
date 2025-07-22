use rand::Rng;

use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};

const START_ADDRESS: u16 = 0x200;
const FONTSET_SIZE: usize = 80;

pub struct Chip8 {
    stack: [u16; 16],
    pc: u16,
    sp: u16,
    index: u16,
    memory: [u8; 4096],
    registers: [u8; 16],
    video: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    keyboard: [bool; 16],
    delay_timer: u8,
    sound_timer: u8,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut emu = Self {
            stack: [0; 16],
            pc: START_ADDRESS,
            sp: 0,
            index: 0,
            memory: [0; 4096],
            registers: [0; 16],
            video: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            keyboard: [false; 16],
            delay_timer: 0,
            sound_timer: 0,
        };

        emu.init_memory();

        emu
    }

    fn init_memory(&mut self) {
        let fontset: [u8; FONTSET_SIZE] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80,
            0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0,
            0x10, 0xF0, 0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90,
            0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0,
            0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0,
            0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
        ];

        self.memory[..FONTSET_SIZE].copy_from_slice(&fontset);
    }

    fn push(&mut self, value: u16) {
        self.stack[self.sp as usize] = value;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn clock(&mut self) {
        let fetch = self.fetch();
        self.execute(fetch);
    }

    pub fn get_display(&self) -> &[bool] {
        &self.video
    }

    pub fn is_key_pressed(&mut self, index: usize, pressed: bool) {
        self.keyboard[index] = pressed;
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        let start = START_ADDRESS as usize;
        let end = (START_ADDRESS as usize) + data.len();
        self.memory[start..end].copy_from_slice(data);
    }

    pub fn tick(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // Implement sound in the future
            }
            self.sound_timer -= 1;
        }
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.memory[self.pc as usize] as u16;
        let lower_byte = self.memory[(self.pc + 1) as usize] as u16;
        let opcode = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        opcode
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0, 0) => return,

            (0, 0, 0xE, 0) => {
                self.video = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }

            (0, 0, 0xE, 0xE) => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            }

            (1, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = nnn;
            }

            (2, _, _, _) => {
                let nnn = op & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            }

            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.registers[x] == nn {
                    self.pc += 2;
                }
            }

            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.registers[x] != nn {
                    self.pc += 2;
                }
            }

            (5, _, _, _) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.registers[x] == self.registers[y] {
                    self.pc += 2;
                }
            }

            (6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.registers[x] = nn;
            }

            (7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.registers[x] = self.registers[x].wrapping_add(nn);
            }

            (8, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.registers[x] = self.registers[y];
            }

            (8, _, _, 1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.registers[x] |= self.registers[y];
            }

            (8, _, _, 2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.registers[x] &= self.registers[y];
            }

            (8, _, _, 3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.registers[x] ^= self.registers[y];
            }

            (8, _, _, 4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, carry) = self.registers[x].overflowing_add(self.registers[y]);
                let new_vf = if carry { 1 } else { 0 };

                self.registers[x] = new_vx;
                self.registers[0xF] = new_vf;
            }

            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.registers[x].overflowing_sub(self.registers[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.registers[x] = new_vx;
                self.registers[0xF] = new_vf;
            }

            (8, _, _, 6) => {
                let x = digit2 as usize;
                let lsb = self.registers[x] & 1;
                self.registers[x] >>= 1;
                self.registers[0xF] = lsb;
            }

            (8, _, _, 7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.registers[y].overflowing_sub(self.registers[x]);
                let new_vf = if borrow { 0 } else { 1 };

                self.registers[x] = new_vx;
                self.registers[0xF] = new_vf;
            }

            (8, _, _, 0xE) => {
                let x = digit2 as usize;
                let msb = (self.registers[x] >> 7) & 1;
                self.registers[x] <<= 1;
                self.registers[0xF] = msb;
            }

            (9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.registers[x] != self.registers[y] {
                    self.pc += 2;
                }
            }

            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;
                self.index = nnn;
            }

            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = (self.registers[0] as u16) + nnn;
            }

            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = rand::rng().random_range(0..255);
                self.registers[x] = rng & nn;
            }

            (0xD, _, _, _) => {
                let x_coord = self.registers[digit2 as usize] as u16;
                let y_coord = self.registers[digit3 as usize] as u16;

                let num_rows = digit4;

                let mut flipped = false;

                for y_line in 0..num_rows {
                    let addr = self.index + y_line as u16;
                    let pixels = self.memory[addr as usize];

                    for x_line in 0..8 {
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            let idx = x + SCREEN_WIDTH * y;

                            flipped |= self.video[idx];
                            self.video[idx] ^= true;
                        }
                    }
                }

                if flipped {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
            }
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let vx = self.registers[x];
                let key = self.keyboard[vx as usize];
                if key {
                    self.pc += 2;
                }
            }
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;
                let vx = self.registers[x];
                let key = self.keyboard[vx as usize];
                if !key {
                    self.pc += 2;
                }
            }
            (0xF, _, 0, 7) => {
                let x = digit2 as usize;
                self.registers[x] = self.delay_timer;
            }
            (0xF, _, 0, 0xA) => {
                let x = digit2 as usize;
                let mut pressed = false;
                for i in 0..self.keyboard.len() {
                    if self.keyboard[i] {
                        self.registers[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    self.pc -= 2;
                }
            }

            (0xF, _, 1, 5) => {
                let x = digit2 as usize;
                self.delay_timer = self.registers[x];
            }

            (0xF, _, 1, 8) => {
                let x = digit2 as usize;
                self.sound_timer = self.registers[x];
            }

            (0xF, _, 1, 0xE) => {
                let x = digit2 as usize;
                let vx = self.registers[x] as u16;
                self.index = self.index.wrapping_add(vx);
            }

            (0xF, _, 2, 9) => {
                let x = digit2 as usize;
                let c = self.registers[x] as u16;
                self.index = c * 5;
            }

            (0xF, _, 3, 3) => {
                let x = digit2 as usize;
                let vx = self.registers[x] as f32;

                let hundreds = (vx / 100.0).floor() as u8;

                let tens = ((vx / 10.0) % 10.0).floor() as u8;

                let ones = (vx % 10.0) as u8;

                self.memory[self.index as usize] = hundreds;
                self.memory[(self.index + 1) as usize] = tens;
                self.memory[(self.index + 2) as usize] = ones;
            }

            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i = self.index as usize;
                for idx in 0..=x {
                    self.memory[i + idx] = self.registers[idx];
                }
            }

            (0xF, _, 6, 5) => {
                let x = digit2 as usize;
                let i = self.index as usize;
                for idx in 0..=x {
                    self.registers[idx] = self.memory[i + idx];
                }
            }
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {:#04x}", op),
        }
    }
}
