use std::fs;
use std::borrow::Borrow;
use std::i16;
use std::time::Duration;

use std::ffi::c_void;

use serde_json;
use serde::{Deserialize, Serialize};

use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{WindowCanvas, Texture};
use sdl2::rect::{Point, Rect};
// "self" imports the "image" module itself as well as everything else we listed
use sdl2::image::{self, LoadTexture, InitFlag};

extern crate x11_dl;
extern crate gl;
use sdl2_sys;

const GL_TRUE: i32 = 1;
const GL_FALSE: i32 = 0;

const GL_COLOR_BUFFER_BIT: GLenum = 0x00004000;

type GLenum = u32;
type GLboolean = u8;
type GLbitfield =   u32;
type GLbyte =       i8;
type GLshort =      i16;
type GLint =        i32;
type GLsizei =      i32;
type GLubyte =      u8;
type GLushort =     u16;
type GLuint =       u8;
type GLfloat =      f32;
type GLclampf =     f32;
type GLdouble =     f64;
type GLclampd =     f64;
type GLvoid =       ();

#[link(kind = "dylib", name = "GL")]
extern {
    fn glEnable(cap: GLenum) -> ();
    fn glViewport(x: GLint, y: GLint, width: GLsizei, height: GLsizei) -> ();
    fn glClearColor(red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat) -> ();
    fn glClear(mask: GLbitfield) -> ();
}

const FRAME_SIZE: usize = 1024;

struct MicCapture {
    audio_buffer: Vec<i16>,
    pos: usize,
    //average_level: f32
}

impl AudioCallback for MicCapture {
    type Channel = i16;

    fn callback(&mut self, input: &mut [i16]) {
        for &sample in input.iter() {
            self.audio_buffer[self.pos] = sample;
            self.pos += 1;

            if self.pos >= self.audio_buffer.len() {
                // self.average_level = calculate_average_volume(&self.audio_buffer);
                // println!("Average Input Level: {:.2}%", self.average_level);
                self.pos = 0;
            }
        }
    }
}

/// Returns a percent value
impl MicCapture {
    fn calculate_average_volume(recorded_vec: &[i16]) -> f32 {
        let sum: i64 = recorded_vec.iter().map(|&x| (x as i64).abs()).sum();
        (sum as f32) / (recorded_vec.len() as f32) / (i16::MAX as f32) * 100.0
    }
}


/*
 * shrimp model r
 *
 * computer
 *
 * phone
 */

#[derive(Debug)]
struct Shrimp {
    position: Point,
    sprite: Rect,
    current_frame: i32,
    forward: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct Region {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Margin {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Sprite {
    filename: String,
    region: Region,
    margin: Margin
}

fn parse_sprites_from_json(path: &str) -> Result<Vec<Sprite>, serde_json::Error> {
    let data = fs::read_to_string(path).expect("Unable to read file");
    let res: serde_json::Value = serde_json::from_str(&data).expect("Unable to parse");
    let sprites: Result<Vec<Sprite>, serde_json::Error> = serde_json::from_value(res["textures"][0]["sprites"].borrow().clone());
    return sprites;
}

fn render(
    canvas: &mut WindowCanvas,
    color: Color,
    texture: &Texture,
    shrimp: &Shrimp,
    computer_texture: &Texture,
    scale_factor: f32,
) -> Result<(), String> {
    //canvas.set_draw_color(color);
    canvas.clear();

    let (width, height) = canvas.output_size()?;

    let (frame_width, frame_height) = shrimp.sprite.size();
    let current_frame = Rect::new(
        shrimp.sprite.x() as i32,
        shrimp.sprite.y() as i32,
        frame_width,
        frame_height,
        );

    let scaled_width = (frame_width as f32 * scale_factor) as u32;
    let scaled_height = (frame_height as f32 * scale_factor) as u32;
    //println!("current_frame rect: {:?}", current_frame);

    //canvas.copy(texture, None, None)?;
    //canvas.copy(texture, Rect::new(0, 0, 286, 602), Rect::new(0, 0, 286, 602))?;
        // Treat the center of the screen as the (0, 0) coordinate
    let screen_position = shrimp.position + Point::new(width as i32 / 2, height as i32 / 2);
    let screen_rect = Rect::from_center(screen_position, scaled_width, scaled_height);
    canvas.copy(texture, current_frame, screen_rect)?;
    canvas.copy(computer_texture, Rect::new(0, 0, 1920, 1080), Rect::new(-200, 200, 1920, 1080))?;

    canvas.present();

    Ok(())
}

fn update_shrimp(shrimp: &mut Shrimp, sprites: &Vec<Sprite>) {
    if shrimp.forward {
        shrimp.current_frame = (shrimp.current_frame + 1) % 18;
    } else {
        shrimp.current_frame = (shrimp.current_frame - 1) % 18;
    }
    if shrimp.current_frame as usize == sprites.len() - 1 {
        shrimp.forward = false;
    } else if shrimp.current_frame == 0 {
        shrimp.forward = true;
    }
    //println!("current_frame: {}", shrimp.current_frame);
    let current_sprite = sprites.get(shrimp.current_frame as usize).expect("index out of bounds");
    //println!("current_sprite: {:?}", current_sprite);
    shrimp.sprite.x = current_sprite.region.x;
    shrimp.sprite.y = current_sprite.region.y;
    shrimp.sprite.w = current_sprite.region.w;
    shrimp.sprite.h = current_sprite.region.h;
    // let y: f64 = 1.0;
    // if shrimp.forward {
    //     shrimp.position.y = shrimp.position.y + (y.sin() * shrimp.current_frame as f64) as i32;
    // } else {
    //     shrimp.position.y = shrimp.position.y - (y.sin() * shrimp.current_frame as f64) as i32;
    // }
}

fn create_x_window() -> u64 {
    let xlib = match x11_dl::xlib::Xlib::open() {
        Ok(x) => x,
        Err(xerr) => panic!("Error: {}", xerr.detail()),
    };

    let display_int = 0_i8;
    let dpy = unsafe { (xlib.XOpenDisplay)(&display_int) };

    let display = {
        if dpy.is_null() {
            panic!("Error opening connection to X Server!");
        } else {
            unsafe { &mut*dpy }
        }
    };

    // get root window
    let root = unsafe { (xlib.XDefaultRootWindow)(display) };

    let glx_ext = match x11_dl::glx::Glx::open() {
        Ok(ext) => ext,
        Err(xerr) => panic!("Error: {}", xerr.detail()),
    };

    let mut visual_info: x11_dl::xlib::XVisualInfo = unsafe { std::mem::MaybeUninit::zeroed().assume_init() } ;
    unsafe {
        if (xlib.XMatchVisualInfo)(
            display,
            (xlib.XDefaultScreen)(display),
            32,
            x11_dl::xlib::TrueColor,
            &mut visual_info) == 0 {
            eprintln!("Failed to match visual info");
            std::process::exit(1);
        }
    }

    let cmap = unsafe { (xlib.XCreateColormap)(display, root, visual_info.visual, x11_dl::xlib::AllocNone) };

    let mut window_attributes: x11_dl::xlib::XSetWindowAttributes = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
    window_attributes.background_pixmap = 0;
    window_attributes.border_pixmap = 0;
    window_attributes.event_mask = x11_dl::xlib::ExposureMask | x11_dl::xlib::KeyPressMask;
    window_attributes.colormap = cmap;

    // construct window
    let window = unsafe { (xlib.XCreateWindow)(
            display,
            root,
            50, 300, 400, 100, 0,
            visual_info.depth,

            1 /* InputOutput */,
            visual_info.visual,

            x11_dl::xlib::CWColormap | x11_dl::xlib::CWEventMask | x11_dl::xlib::CWBackPixmap | x11_dl::xlib::CWBorderPixel,
            &mut window_attributes) };

    unsafe { (xlib.XMapWindow)(display, window) };
    //unsafe { (xlib.XStoreName)(display, window, window_title.as_ptr()) };

    let visual_info_ptr = &mut visual_info as *mut x11_dl::xlib::XVisualInfo;

    let glc = unsafe { (glx_ext.glXCreateContext)(display, visual_info_ptr, ::std::ptr::null_mut(), GL_TRUE) };

    unsafe { (glx_ext.glXMakeCurrent)(display, window, glc) };

    return window;
}

fn main() -> Result<(), String> {
    let path = "assets/shrimpy.tpsheet";

    let sprites: Vec<Sprite> = parse_sprites_from_json(&path).expect("Couldn't parse spritesheet");

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let audio_subsystem = sdl_context.audio()?;

    let desired_spec = AudioSpecDesired {
        freq: None,
        channels: Some(1),
        samples: Some(FRAME_SIZE as u16),
    };

    let mut mic_capture = MicCapture {
        audio_buffer: vec![0; FRAME_SIZE],
        pos: 0,
        //average_level: 1.0
    };

    //let mut average_level_clone = mic_capture.borrow().average_level.clone();

    let mut capture_device = audio_subsystem.open_capture(None, &desired_spec, |spec| {
        println!("Capture Spec = {:?}", spec);
        mic_capture.audio_buffer.resize(spec.samples as usize, 0);
        mic_capture.pos = 0;
        mic_capture
    })?;

    capture_device.resume();

    // Leading "_" tells Rust that this is an unused variable that we don't care about. It has to
    // stay unused because if we don't have any variable at all then Rust will treat it as a
    // temporary value and drop it right away!
    let _image_context = image::init(InitFlag::PNG | InitFlag::JPG)?;

    let x_window = create_x_window();

    let w: *mut sdl2_sys::SDL_Window = unsafe { sdl2_sys::SDL_CreateWindowFrom(x_window as *mut c_void) };

    let window: sdl2::video::Window = {
        unsafe { sdl2::video::Window::from_ll(video_subsystem, w) }
    };

    // let mut window = video_subsystem.window("shrimpius", 1920, 1080)
    //     //window
    //     .position_centered()
    //     .build()
    //     .expect("could not initialize video subsystem");

    // let result = window.set_opacity(0.0);
    // println!("{:?}", result);

    let mut canvas = window.into_canvas().build()
        .expect("could not make a canvas");

    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.load_texture("assets/shrimpy-0.png")?;
    let computer_texture = texture_creator.load_texture("assets/computer.png")?;

    let mut shrimp = Shrimp {
        position: Point::new(0, 0),
        sprite: Rect::new(0, 0, 286, 602),
        current_frame: 0,
        forward: true,
    };

    let mut event_pump = sdl_context.event_pump()?;
    let mut i = 0;

    let mut scale_factor = 0.7;

    'running: loop {
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },
                _ => {}
            }
        }

        // Update
        i = (i + 1) % 255;
        update_shrimp(&mut shrimp, &sprites);

        //let average_level_clone = mic_capture.average_level.clone();
        //scale_factor = mic_capture.average_level.clone();
        //println!("average_level_clone: {:?}", average_level_clone);
        //
        let average_level: f32 = MicCapture::calculate_average_volume(&capture_device.lock().audio_buffer);
        //println!("Average Input Level: {:.2}", average_level);

        // Render
        render(
            &mut canvas,
            Color::RGBA(255, 255, 255, 0),
            &texture,
            &shrimp,
            &computer_texture,
            average_level,
            //average_level_clone
        )?;

        // Time management!
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
