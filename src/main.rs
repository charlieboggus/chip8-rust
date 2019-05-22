extern crate sdl2;
extern crate rand;
extern crate time;

mod cpu;
mod display;
mod keypad;

use crate::cpu::CPU;
use crate::display::{ 
    DISPLAY_WIDTH, 
    DISPLAY_HEIGHT, 
    DISPLAY_PIXEL_SCALE, 
    DISPLAY_COLOR_PIXEL_ON, 
    DISPLAY_COLOR_PIXEL_OFF 
};

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::WindowCanvas;
use std::thread;
use time::{ Duration, SteadyTime };

fn main() -> Result< (), String >
{
    // Initialize SDL
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let mut timer_subsys = sdl_context.timer()?;

    // Create the SDL window
    let window = video_subsys
        .window("chip8-rs", display::DISPLAY_WIDTH as u32 * display::DISPLAY_PIXEL_SCALE as u32, display::DISPLAY_HEIGHT as u32 * display::DISPLAY_PIXEL_SCALE as u32)
        .opengl()
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    // Create the SDL drawing canvas and texture
    let mut canvas = window
        .into_canvas()
        .accelerated()
        .build()
        .map_err(|e| e.to_string())?;
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    // Create input stuff
    let mut event_pump = sdl_context.event_pump().map_err(|e| e.to_string())?;
    let key_binds = keypad::get_sdl_keybinds();

    // Time handling
    let mut time = SteadyTime::now();
    let mut last_cpu_time = SteadyTime::now();
    let mut last_timers_time = time;
    let timers_step = Duration::nanoseconds(10i64.pow(9) / (cpu::TIMER_CLOCK as i64));
    let cpu_step = Duration::nanoseconds(10i64.pow(9) / (cpu::CPU_CLOCK as i64));

    // Framerate handling
    let fps = 60.0;
    let mut fps_time = timer_subsys.ticks();
    let mut prev_fps_time;
    let mut dt;
    let mut update_timer = 0.0;
    let max_dt = 1000.0 / fps;

    // Create the Chip-8 CPU & load a rom
    let mut cpu = cpu::CPU::new();
    cpu.load_rom(std::path::Path::new("ROMS/PONG.ch8"));

    // Main application loop
    'running: loop
    {
        // Update FPS time variables
        prev_fps_time = fps_time;
        fps_time = timer_subsys.ticks();
        dt = fps_time - prev_fps_time;

        // Handle SDL events
        for event in event_pump.poll_iter()
        {
            match event
            {
                // Quit events
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,

                // Keydown events
                Event::KeyDown { keycode, .. } => 
                {
                    // Send the key down event to the CPU
                    if let Some(value) = key_binds.get(&keycode.unwrap())
                    {
                        let key = *value;
                        cpu.keypad.set_key_state(key, true);
                        if cpu.is_waiting_for_key()
                        {
                            cpu.stop_waiting_for_key(key);
                        }
                    }
                },

                // Keyup events
                Event::KeyUp { keycode, .. } =>
                {
                    if let Some(value) = key_binds.get(&keycode.unwrap())
                    {
                        let key = *value;
                        cpu.keypad.set_key_state(key, false);
                    }
                },

                _ => { continue; }
            }
        }

        // CPU cycle
        time = SteadyTime::now();
        if time - last_cpu_time >= cpu_step
        {
            last_cpu_time = time;
            cpu.cpu_cycle();
        }

        // Update CPU timers
        time = SteadyTime::now();
        if time - last_timers_time >= timers_step
        {
            last_timers_time = time;
            cpu.update_cpu_timers();
        }

        // Render
        draw_display(&mut canvas, &mut cpu);
        while update_timer >= max_dt
        {
            update_timer -= max_dt;
            canvas.present();
        }
        update_timer += dt as f32;

        // Avoid overloading CPU by sleeping thread
        thread::sleep(::std::time::Duration::from_millis(1));
    }

    Ok(())
}

fn draw_display(canvas: &mut WindowCanvas, cpu: &mut CPU)
{
    canvas.set_draw_color(DISPLAY_COLOR_PIXEL_OFF);
    canvas.clear();
    canvas.set_draw_color(DISPLAY_COLOR_PIXEL_ON);
    for y in 0..DISPLAY_HEIGHT as i32
    {
        for x in 0..DISPLAY_WIDTH as i32
        {
            if cpu.display.memory[y as usize][x as usize] == 1u8
            {
                canvas.fill_rect(Rect::new(x * DISPLAY_PIXEL_SCALE, y * DISPLAY_PIXEL_SCALE, DISPLAY_PIXEL_SCALE as u32, DISPLAY_PIXEL_SCALE as u32)).unwrap();
            }
        }
    }
}
