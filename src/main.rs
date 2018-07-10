#![feature(plugin)]

#![plugin(phf_macros)]
extern crate phf;
extern crate piston;
extern crate opengl_graphics;
extern crate image;
extern crate graphics;
extern crate glutin_window;

use piston::input::*;
use std::time::Instant;
use piston::window::WindowSettings;
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use piston::event_loop::*;
use glutin_window::GlutinWindow as Window;

mod cpu;
mod ines;
mod controller;
mod nes;
mod memory;
mod ppu;
mod settings;

mod mapper_0;

use ines::*;
use nes::*;
use settings::*;
use ppu::{make_canvas, NesImageBuffer};

trait ControllerMethod {
    fn do_input(&mut self, nes: &mut Nes, e: &Event);
}

struct User {
    dump_count: u8,
}

impl ControllerMethod for User {
    fn do_input(&mut self, nes: &mut Nes, e: &Event) {
        if let Some(button) = e.press_args() {
            match button {
                Button::Keyboard(Key::D) => nes.cpu.debug = DEBUG,
                Button::Keyboard(Key::R) => {
                    if DEBUG {
                        write_bytes_to_file(format!("{}.bin", self.dump_count), &nes.chipset.mem.ram);
                        self.dump_count += 1;
                    }
                },
                Button::Keyboard(Key::Up) => nes.chipset.controller1.up = true,
                Button::Keyboard(Key::Left) => nes.chipset.controller1.left = true,
                Button::Keyboard(Key::Down) => nes.chipset.controller1.down = true,
                Button::Keyboard(Key::Right) => nes.chipset.controller1.right = true,
                Button::Keyboard(Key::A) => nes.chipset.controller1.a = true,
                Button::Mouse(MouseButton::Left) => nes.chipset.controller1.a = true,
                Button::Keyboard(Key::S) => nes.chipset.controller1.b = true,
                Button::Keyboard(Key::Return) => nes.chipset.controller1.start = true,
                Button::Keyboard(Key::Space) => nes.chipset.controller1.select = true,
                _ => ()
            }
        }

        if let Some(button) = e.release_args() {
            match button {
                Button::Keyboard(Key::Up) => nes.chipset.controller1.up = false,
                Button::Keyboard(Key::Left) => nes.chipset.controller1.left = false,
                Button::Keyboard(Key::Down) => nes.chipset.controller1.down = false,
                Button::Keyboard(Key::Right) => nes.chipset.controller1.right = false,
                Button::Keyboard(Key::A) => nes.chipset.controller1.a = false,
                Button::Mouse(MouseButton::Left) => nes.chipset.controller1.a = false,
                Button::Keyboard(Key::S) => nes.chipset.controller1.b = false,
                Button::Keyboard(Key::Return) => nes.chipset.controller1.start = false,
                Button::Keyboard(Key::Space) => nes.chipset.controller1.select = false,
                _ => ()
            }
        }
    }
}

struct App {
    nes: Nes,
    frames: u64,
    last_time: Instant,

    gl_graphics: GlGraphics,
    controller_method: Box<ControllerMethod>,
    texture: Texture,
    canvas: NesImageBuffer,
}

fn emulate((flags, prg, chr) : (Flags, Vec<u8>, Vec<u8>), controller_method: Box<ControllerMethod>) {
    println!("Loaded rom with {:?}", flags);

    let size = [256, 240];

    let mut window: Window =
        WindowSettings::new("Emulator", size)
            .opengl(OpenGL::V2_1)
            .exit_on_esc(true).build().unwrap();
    let gl_graphics = GlGraphics::new(OpenGL::V2_1);

    let nes = Nes::new(prg, chr, flags.mapper, flags.prg_ram_size, flags.horiz_mirroring);

    let canvas = make_canvas(size[0], size[1]);
    let tex = Texture::from_image(&canvas, &TextureSettings::new());

    let mut app = App {
        nes: nes,
        frames: 0,
        last_time:Instant::now(),
        controller_method: controller_method,

        gl_graphics: gl_graphics,
        texture: tex,
        canvas: canvas,
    };


    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        handle_event(e, &mut app);
    }
}

fn handle_event(e: Event, app: &mut App) {
    if let Some(size) = e.resize_args() {
        app.canvas = make_canvas(size[0] as u32, size[1] as u32);
        app.texture = Texture::from_image(&app.canvas, &TextureSettings::new());
    }

    if let Some(args) = e.render_args() {
        app.frames += 1;

        if app.frames > 60 {
            let elapsed = app.last_time.elapsed();
            let ms = (elapsed.as_secs() * 1_000) + (elapsed.subsec_nanos() / 1_000_000) as u64;
            println!("MS per frame: {}", ms/app.frames);
            app.frames = 0;
            app.last_time = Instant::now();
        }

        app.nes.tick();
        app.nes.prepare_draw(&mut app.canvas);

        app.texture.update(&app.canvas);
        let tex = &app.texture;

        app.gl_graphics.draw(args.viewport(),
                              |ctx, g2d| graphics::image(tex, ctx.transform, g2d));

        //app.canvas.save(format!("{}.png", app.frames)).unwrap();
    }

    app.controller_method.as_mut().do_input(&mut app.nes, &e);
}

fn main() {
    let input: Box<ControllerMethod> = Box::new(User { dump_count: 0 });
    match load_file("assets/smb.nes") {
        Ok(rom) => emulate(rom, input),
        Err(e) => panic!("Error: {:?}", e)
    }
}


