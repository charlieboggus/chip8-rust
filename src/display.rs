use sdl2::pixels::Color;

pub const DISPLAY_WIDTH: i32 = 64;
pub const DISPLAY_HEIGHT: i32 = 32;
pub const DISPLAY_PIXEL_SCALE: i32 = 10;

pub const DISPLAY_COLOR_PIXEL_ON: Color = Color { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF };
pub const DISPLAY_COLOR_PIXEL_OFF: Color = Color { r: 0x0, g: 0x0, b: 0x0, a: 0xFF };

pub static CHIP8_FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,   // 0
    0x20, 0x60, 0x20, 0x20, 0x70,   // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0,   // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0,   // 3
    0x90, 0x90, 0xF0, 0x10, 0x10,   // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0,   // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0,   // 6
    0xF0, 0x10, 0x20, 0x40, 0x40,   // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0,   // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0,   // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90,   // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0,   // B
    0xF0, 0x80, 0x80, 0x80, 0xF0,   // C
    0xE0, 0x90, 0x90, 0x90, 0xE0,   // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0,   // E
    0xF0, 0x80, 0xF0, 0x80, 0x80    // F
];

pub struct Display
{
    /// The display is comprised of 64x32 pixels so we represent the display 
    /// memory as an array of 2048 bytes
    /// For a single pixel: 1 means the pixel is ON and 0 means the pixel is OFF
    pub memory: [[u8; DISPLAY_WIDTH as usize]; DISPLAY_HEIGHT as usize],
}

impl Display
{
    /// Create and return a new instance of Display
    pub fn new() -> Self
    {
        Display {
            memory: [[0u8; DISPLAY_WIDTH as usize]; DISPLAY_HEIGHT as usize],
        }
    }

    pub fn clear(&mut self)
    {
        self.memory = [[0u8; DISPLAY_WIDTH as usize]; DISPLAY_HEIGHT as usize];
    }

    pub fn draw(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool
    {
        let mut collision = false;
        let h = sprite.len();
        for j in 0..h
        {
            for i in 0..8
            {
                let ypos = (y + j) % DISPLAY_HEIGHT as usize;
                let xpos = (x + i) % DISPLAY_WIDTH as usize;
                if (sprite[j] & (0x80 >> i)) != 0x00
                {
                    if self.memory[ypos][xpos] == 0x01
                    {
                        collision = true;
                    }
                    self.memory[ypos][xpos] ^= 0x01;
                }
            }
        }

        collision
    }
}