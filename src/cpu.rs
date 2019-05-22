use crate::display::{ Display, CHIP8_FONT };
use crate::keypad::Keypad;

use rand::random;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// The CPU clock speed in Hz
pub const CPU_CLOCK: i32 = 600;

/// The timer clock speed in Hz
pub const TIMER_CLOCK: i32 = 60;

/// The index of the carry flag register
const CARRY_FLAG: usize = 15;

/// The default stack size
const STACK_SIZE: usize = 16;

pub struct CPU
{
    /// The current opcode
    opcode: u16,

    /// The 16 registers, V0 to VF, available to the CPU. Register VF is
    /// used for the carry flag
    v: [u8; 16],

    /// Index register
    i: usize,

    /// Program counter
    pc: usize,

    /// Chip-8's 4096 bytes of memory
    pub memory: [u8; 4096],

    /// Chip-8 display
    pub display: Display,
    
    /// Chip-8 keypad
    pub keypad: Keypad,

    /// Is the CPU waiting for a keypress? If so any key pressed is stored in Vx
    pub wait_for_key: Option< u8 >,

    /// The stack. Has 16 levels of nesting. Used for subroutine operations.
    pub stack: [u16; STACK_SIZE],
    
    /// Stack Pointer
    pub sp: usize,

    /// Delay timer register
    pub delay_timer: u8,

    /// Sound timer register
    pub sound_timer: u8,
}

impl CPU
{
    /// Creates and returns a new instance of a Chip-8 CPU
    pub fn new() -> Self
    {
        let mut cpu = CPU {
            opcode: 0u16,
            v: [0u8; 16],
            i: 0usize,
            pc: 0usize,
            memory: [0u8; 4096],
            display: Display::new(),
            keypad: Keypad::new(),
            wait_for_key: None,
            stack: [0u16; STACK_SIZE],
            sp: 0usize,
            delay_timer: 0u8,
            sound_timer: 0u8
        };

        // Load the font into memory
        for i in 0..80
        {
            cpu.memory[i] = CHIP8_FONT[i];
        }

        // program space starts at 0x200
        cpu.pc = 0x200;

        cpu
    }

    /// Loads a Chip-8 ROM from file into the CPU's memory
    pub fn load_rom(&mut self, path: &Path) -> Option< String >
    {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(ref e) => return Some(format!("Could not open ROM file \"{}\". Error: {}", path.display(), Error::description(e)))
        };

        for (i, b) in file.bytes().enumerate()
        {
            match b
            {
                Ok(byte) => self.memory[self.pc + i] = byte,
                Err(e) => return Some(format!("Error reading ROM file: {}", e.to_string()))
            }
        }

        None
    }

    /// Executes a single Chip-8 CPU cycle
    pub fn cpu_cycle(&mut self)
    {
        self.fetch_opcode();
        self.execute_opcode();
    }

    pub fn update_cpu_timers(&mut self)
    {
        if self.delay_timer > 0
        {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0
        {
            self.sound_timer -= 1;
        }
    }

    pub fn is_waiting_for_key(&self) -> bool
    {
        self.wait_for_key.is_some()
    }

    pub fn stop_waiting_for_key(&mut self, key: usize)
    {
        if !self.is_waiting_for_key()
        {
            return;
        }

        if let Some(x) = self.wait_for_key
        {
            if self.keypad.get_key_state(key)
            {
                self.v[x as usize] = key as u8;
                self.wait_for_key = None;
            }
        }
    }

    /// Fetches the next opcode to execute from memory
    fn fetch_opcode(&mut self)
    {
        self.opcode = (self.memory[self.pc] as u16) << 8 | (self.memory[self.pc + 1] as u16);
    }

    /// Executes the currently stored opcode
    fn execute_opcode(&mut self)
    {
        // Get the value of each nibble X, Y, Z, W from opcode 0xXYZW for easy matching
        let op = (
            ((self.opcode & 0xF000) >> 12) as u8,
            ((self.opcode & 0x0F00) >> 8) as u8,
            ((self.opcode & 0x00F0) >> 4) as u8,
            (self.opcode & 0x000F) as u8
        );

        // Match the opcode to the related instruction function
        // http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
        match op
        {
            (0x0, 0x0, 0xE, 0x0) => self.instr_cls(),
            (0x0, 0x0, 0xE, 0xE) => self.instr_ret(),
            (0x1, _, _, _) => self.instr_jp_addr(self.opcode & 0x0FFF),
            (0x2, _, _, _) => self.instr_call_addr(self.opcode & 0x0FFF),
            (0x3, x, _, _) => self.instr_se_vx_nn(x, (self.opcode & 0x00FF) as u8),
            (0x4, x, _, _) => self.instr_sne_vx_nn(x, (self.opcode & 0x00FF) as u8),
            (0x5, x, y, 0x0) => self.instr_se_vx_vy(x, y),
            (0x6, x, _, _) => self.instr_ld_vx_nn(x, (self.opcode & 0x00FF) as u8),
            (0x7, x, _, _) => self.instr_add_vx_nn(x, (self.opcode & 0x00FF) as u8),
            (0x8, x, y, 0x0) => self.instr_ld_vx_vy(x, y),
            (0x8, x, y, 0x1) => self.instr_or_vx_vy(x, y),
            (0x8, x, y, 0x2) => self.instr_and_vx_vy(x, y),
            (0x8, x, y, 0x3) => self.instr_xor_vx_vy(x, y),
            (0x8, x, y, 0x4) => self.instr_add_vx_vy(x, y),
            (0x8, x, y, 0x5) => self.instr_sub_vx_vy(x, y),
            (0x8, x, y, 0x6) => self.instr_shr_vx_vy(x, y),
            (0x8, x, y, 0x7) => self.instr_subn_vx_vy(x, y),
            (0x8, x, y, 0xE) => self.instr_shl_vx_vy(x, y),
            (0x9, x, y, 0x0) => self.instr_sne_vx_vy(x, y),
            (0xA, _, _, _) => self.instr_ld_i_addr(self.opcode & 0x0FFF),
            (0xB, _, _, _) => self.instr_jp_v0_addr(self.opcode & 0x0FFF),
            (0xC, x, _, _) => self.instr_rnd_vx_nn(x, (self.opcode & 0x00FF) as u8),
            (0xD, x, y, n) => self.instr_drw_vx_vy_nn(x, y, n),
            (0xE, x, 0x9, 0xE) => self.instr_skp_vx(x),
            (0xE, x, 0xA, 0x1) => self.instr_sknp_vx(x),
            (0xF, x, 0x0, 0x7) => self.instr_ld_vx_dt(x),
            (0xF, x, 0x0, 0xA) => self.instr_ld_vx_k(x),
            (0xF, x, 0x1, 0x5) => self.instr_ld_dt_vx(x),
            (0xF, x, 0x1, 0x8) => self.instr_ld_st_vx(x),
            (0xF, x, 0x1, 0xE) => self.instr_add_i_vx(x),
            (0xF, x, 0x2, 0x9) => self.instr_ld_f_vx(x),
            (0xF, x, 0x3, 0x3) => self.instr_ld_b_vx(x),
            (0xF, x, 0x5, 0x5) => self.instr_ld_i_vx(x),
            (0xF, x, 0x6, 0x5) => self.instr_ld_vx_i(x),

            _ => {}
        }
    }

    /// Instruction executed by opcode 00E0 
    /// Clear the display
    fn instr_cls(&mut self)
    {
        self.display.clear();
        self.pc += 2;
    }

    /// Instruction executed by opcode 00EE 
    /// Return from a subroutine
    fn instr_ret(&mut self)
    {
        self.sp -= 1;
        let addr = self.stack[self.sp];
        self.instr_jp_addr(addr);
        self.pc += 2;
    }

    /// Instruction executed by opcode 1nnn 
    /// Jump to location nnn
    fn instr_jp_addr(&mut self, addr: u16)
    {
        self.pc = addr as usize;
    }

    /// Instruction executed by opcode 2nnn 
    /// Call subroutine at location nnn
    fn instr_call_addr(&mut self, addr: u16)
    {
        self.stack[self.sp] = self.pc as u16;
        self.sp += 1;
        self.instr_jp_addr(addr);
    }

    /// Instruction executed by opcode 3xnn 
    /// Skip next instruction if Vx = nn
    fn instr_se_vx_nn(&mut self, x: u8, nn: u8)
    {
        self.pc += if self.v[x as usize] == nn { 4 } else { 2 };
    }

    /// Instruction executed by opcode 4xnn 
    /// Skip next instruction if Vx != nn
    fn instr_sne_vx_nn(&mut self, x: u8, nn: u8)
    {
        self.pc += if self.v[x as usize] != nn { 4 } else { 2 };
    }

    /// Instruction executed by opcode 5xy0 
    /// Skip next instruction if Vx = Vy
    fn instr_se_vx_vy(&mut self, x: u8, y: u8)
    {
        self.pc += if self.v[x as usize] == self.v[y as usize] { 4 } else { 2 };
    }

    /// Instruction executed by opcode 6xnn 
    /// Set Vx = nn
    fn instr_ld_vx_nn(&mut self, x: u8, nn: u8)
    {
        self.v[x as usize] = nn;
        self.pc += 2;
    }

    /// Instruction executed by opcode 7xnn 
    /// Set Vx = Vx + nn
    fn instr_add_vx_nn(&mut self, x: u8, nn: u8)
    {
        self.v[x as usize] = (self.v[x as usize] as u16 + nn as u16) as u8;
        self.pc += 2;
    }

    /// Instruction executed by opcode 8xy0 
    /// Set Vx = Vy
    fn instr_ld_vx_vy(&mut self, x: u8, y: u8)
    {
        self.v[x as usize] = self.v[y as usize];
        self.pc += 2;
    }

    /// Instruction executed by opcode 8xy1 
    /// Set Vx = Vx OR Vy
    fn instr_or_vx_vy(&mut self, x: u8, y: u8)
    {
        self.v[x as usize] = self.v[x as usize] | self.v[y as usize];
        self.pc += 2;
    }

    /// Instruction executed by opcode 8xy2 
    /// Set Vx = Vx AND Vy
    fn instr_and_vx_vy(&mut self, x: u8, y: u8)
    {
        self.v[x as usize] = self.v[x as usize] & self.v[y as usize];
        self.pc += 2;
    }

    /// Instruction executed by opcode 8xy3 
    /// Set Vx = Vx XOR Vy
    fn instr_xor_vx_vy(&mut self, x: u8, y: u8)
    {
        self.v[x as usize] = self.v[x as usize] ^ self.v[y as usize];
        self.pc += 2;
    }

    /// Instruction executed by opcode 8xy4 
    /// Set Vx = Vx + Vy 
    /// Sets carry flag to 0x1 if the result is greater than 8 bits (i.e. > 255)
    fn instr_add_vx_vy(&mut self, x: u8, y: u8)
    {
        let value = self.v[x as usize] as u16 + self.v[y as usize] as u16;
        self.v[x as usize] = value as u8;
        self.v[CARRY_FLAG] = if value > 255 { 0x1 } else { 0x0 };
        self.pc += 2;
    }

    /// Instruction executed by opcode 8xy5
    /// Set Vx = Vx - Vy 
    /// Sets carry flag to 0x0 if a borrow occurs 
    /// and 0x1 if a borrow does not occur
    fn instr_sub_vx_vy(&mut self, x: u8, y: u8)
    {
        let value = self.v[x as usize] as i8 - self.v[y as usize] as i8;
        self.v[x as usize] = value as u8;
        self.v[CARRY_FLAG] = if value < 0 { 0x0 } else { 0x1 };   
        self.pc += 2;
    }

    /// Instruction executed by opcode 8xy6 
    /// Store the value of Vy shifted right one bit in Vx then 
    /// set the carry flag to the most significant bit prior to the shift
    fn instr_shr_vx_vy(&mut self, x: u8, y: u8)
    {
        self.v[CARRY_FLAG] = self.v[y as usize] & 0x80;
        self.v[x as usize] = self.v[y as usize] >> 1;
        self.pc += 2;
    }

    /// Instruction executed by opcode 8xy7 
    /// Set Vx = Vy - Vx 
    /// Sets carry flag to 0x0 if a borrow occurs 
    /// and 0x1 if a borrow does not occur
    fn instr_subn_vx_vy(&mut self, x: u8, y: u8)
    {
        let value = self.v[y as usize] as i8 - self.v[x as usize] as i8;
        self.v[x as usize] = value as u8;
        self.v[CARRY_FLAG] = if value < 0 { 0x0 } else { 0x1 };   
        self.pc += 2;
    }

    /// Instruction executed by opcode 8xyE 
    /// Store the value of Vy shifted left one bit in Vx then 
    /// set the carry flag to the least significant bit prior to the shift
    fn instr_shl_vx_vy(&mut self, x: u8, y: u8)
    {
        self.v[CARRY_FLAG] = self.v[y as usize] & 0x01;
        self.v[x as usize] = self.v[y as usize] << 1;
        self.pc += 2;
    }

    /// Instruction executed by opcode 9xy0 
    /// Skip next instruction if Vx != Vy
    fn instr_sne_vx_vy(&mut self, x: u8, y: u8)
    {
        self.pc += if self.v[x as usize] != self.v[y as usize] { 4 } else { 2 };
    }

    /// Instruction executed by opcode Annn 
    /// Set I = nnn
    fn instr_ld_i_addr(&mut self, addr: u16)
    {
        self.i = addr as usize;
        self.pc += 2;
    }

    /// Instruction executed by opcode Bnnn 
    /// Jump to location nnn + V0
    fn instr_jp_v0_addr(&mut self, addr: u16)
    {
        let v0 = self.v[0] as u16;
        self.instr_jp_addr(addr + v0);
    }
    
    /// Instruction executed by opcode Cxnn 
    /// Set Vx = random byte AND nn
    fn instr_rnd_vx_nn(&mut self, x: u8, nn: u8)
    {
        self.v[x as usize] = random::< u8 >() & nn;
        self.pc += 2;
    }
    
    /// Instruction executed by opcode Dxyn 
    /// Display n-byte sprite starting at memory location I at (Vx, Vy) 
    /// Sets the carry flag to 0x1 if a collision occurs
    fn instr_drw_vx_vy_nn(&mut self, x: u8, y: u8, nn: u8)
    {
        let x = self.v[x as usize] as usize;
        let y = self.v[y as usize] as usize;
        let mem_start = self.i;
        let mem_end = self.i + nn as usize;
        
        if self.display.draw(x, y, &self.memory[mem_start..mem_end])
        {
            self.v[CARRY_FLAG] = 0x1;
        }
        else
        {
            self.v[CARRY_FLAG] = 0x0;
        }

        self.pc += 2;
    }
    
    /// Instruction executed by opcode Ex9E 
    /// Skip next instruction if key with the value Vx is pressed
    fn instr_skp_vx(&mut self, x: u8)
    {
        self.pc += if self.keypad.get_key_state(self.v[x as usize] as usize) { 4 } else { 2 };
    }
    
    /// Instruction executed by opcode ExA1 
    /// Skip next instruction if key with the value Vx is not pressed
    fn instr_sknp_vx(&mut self, x: u8)
    {
        self.pc += if self.keypad.get_key_state(self.v[x as usize] as usize) { 2 } else { 4 };
    }
    
    /// Instruction executed by opcode Fx07 
    /// Set Vx = delay timer value
    fn instr_ld_vx_dt(&mut self, x: u8)
    {
        self.v[x as usize] = self.delay_timer;
        self.pc += 2;
    }
    
    /// Instruction executed by opcode Fx0A 
    /// Wait for a key press and store the value of the key in Vx
    fn instr_ld_vx_k(&mut self, x: u8)
    {
        self.wait_for_key = Some(x);
    }
    
    /// Instruction executed by opcode Fx15 
    /// Set delay timer = Vx
    fn instr_ld_dt_vx(&mut self, x: u8)
    {
        self.delay_timer = self.v[x as usize];
        self.pc += 2;
    }
    
    /// Instruction executed by opcode Fx18 
    /// Set sound timer = Vx
    fn instr_ld_st_vx(&mut self, x: u8)
    {
        self.sound_timer = self.v[x as usize];
        self.pc += 2;
    }
    
    /// Instruction executed by opcode Fx1E 
    /// Set I = I + Vx
    fn instr_add_i_vx(&mut self, x: u8)
    {
        self.i = self.i + self.v[x as usize] as usize;
        self.pc += 2;
    }
    
    /// Instruction executed by opcode Fx29 
    /// Set I = location of sprite for digit Vx
    fn instr_ld_f_vx(&mut self, x: u8)
    {
        self.i = (self.v[x as usize] * 5) as usize;
        self.pc += 2;
    }
    
    /// Instruction executed by opcode Fx33 
    /// Store BCD representation of Vx in memory locations I, I + 1, and I + 2
    fn instr_ld_b_vx(&mut self, x: u8)
    {
        let vx = self.v[x as usize];
        self.memory[self.i] = vx / 100;
        self.memory[self.i + 1] = (vx / 10) % 10;
        self.memory[self.i + 2] = vx % 10;
        self.pc += 2;
    }
    
    /// Instruction executed by opcode Fx55 
    /// Stores registers V0 through Vx in memory starting at location I
    fn instr_ld_i_vx(&mut self, x: u8)
    {
        for i in 0..(x as usize + 1)
        {
            self.memory[self.i + i] = self.v[i];
        }
        self.i = self.i + x as usize + 1;
        self.pc += 2;
    }
    
    /// Instruction executed by opcode Fx65 
    /// Reads registers V0 through Vx from memory starting at location I
    fn instr_ld_vx_i(&mut self, x: u8)
    {
        for i in 0..(x as usize + 1)
        {
            self.v[i] = self.memory[self.i + i];
        }
        self.i = self.i + x as usize + 1;
        self.pc += 2;
    }
}