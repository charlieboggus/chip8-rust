use sdl2::keyboard::Keycode;
use std::collections::HashMap;

pub struct Keypad
{
    keys: [bool; 16],
}

impl Keypad
{
    pub fn new() -> Self
    {
        Keypad {
            keys: [false; 16],
        }
    }

    pub fn get_key_state(&self, index: usize) -> bool
    {
        self.keys[index]
    }

    pub fn set_key_state(&mut self, index: usize, state: bool)
    {
        self.keys[index] = state;
    }
}

pub fn get_sdl_keybinds() -> HashMap< Keycode, usize >
{
    let mut hm = HashMap::new();
    hm.insert(Keycode::Num1, 0x1);
    hm.insert(Keycode::Num2, 0x2);
    hm.insert(Keycode::Num3, 0x3);
    hm.insert(Keycode::Num4, 0xC);
    hm.insert(Keycode::Q, 0x4);
    hm.insert(Keycode::W, 0x5);
    hm.insert(Keycode::E, 0x6);
    hm.insert(Keycode::R, 0xD);
    hm.insert(Keycode::A, 0x7);
    hm.insert(Keycode::S, 0x8);
    hm.insert(Keycode::D, 0x9);
    hm.insert(Keycode::F, 0xE);
    hm.insert(Keycode::Z, 0xA);
    hm.insert(Keycode::X, 0x0);
    hm.insert(Keycode::C, 0xB);
    hm.insert(Keycode::V, 0xF);

    hm
}