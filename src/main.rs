use std::fs;
use std::borrow::Borrow;

use serde_json;
use serde::{Deserialize, Serialize};

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{WindowCanvas, Texture};
use sdl2::rect::{Point, Rect};
// "self" imports the "image" module itself as well as everything else we listed
use sdl2::image::{self, LoadTexture, InitFlag};
use std::time::Duration;

/* 
 * shrimp model
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
) -> Result<(), String> {
    canvas.set_draw_color(color);
    canvas.clear();

    let (width, height) = canvas.output_size()?;

    let (frame_width, frame_height) = shrimp.sprite.size();
    let current_frame = Rect::new(
        shrimp.sprite.x() as i32,
        //shrimp.sprite.y() + frame_height as i32 * direction_spritesheet_row(shrimp.direction),
        shrimp.sprite.y() as i32,
        frame_width,
        frame_height,
        );
    println!("current_frame rect: {:?}", current_frame);


    //canvas.copy(texture, None, None)?;
    //canvas.copy(texture, Rect::new(0, 0, 286, 602), Rect::new(0, 0, 286, 602))?;
        // Treat the center of the screen as the (0, 0) coordinate
    let screen_position = shrimp.position + Point::new(width as i32 / 2, height as i32 / 2);
    let screen_rect = Rect::from_center(screen_position, frame_width, frame_height);
    canvas.copy(texture, current_frame, screen_rect)?;

    canvas.present();

    Ok(())
}

fn update_shrimp(shrimp: &mut Shrimp, sprites: &Vec<Sprite>) {
    shrimp.current_frame = (shrimp.current_frame + 1) % 18;
    println!("current_frame: {}", shrimp.current_frame);
    let current_sprite = sprites.get(shrimp.current_frame as usize).expect("index out of bounds");
    shrimp.sprite.x = current_sprite.region.x;
    shrimp.sprite.y = current_sprite.region.y;
    shrimp.sprite.w = current_sprite.region.w;
    shrimp.sprite.h = current_sprite.region.h;
}

fn main() -> Result<(), String> {
    let path = "assets/shrimpy.tpsheet";

    let sprites: Vec<Sprite> = parse_sprites_from_json(&path).expect("Couldn't parse spritesheet");

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    // Leading "_" tells Rust that this is an unused variable that we don't care about. It has to
    // stay unused because if we don't have any variable at all then Rust will treat it as a
    // temporary value and drop it right away!
    let _image_context = image::init(InitFlag::PNG | InitFlag::JPG)?;

    let window = video_subsystem.window("game tutorial", 800, 600)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let mut canvas = window.into_canvas().build()
        .expect("could not make a canvas");

    let texture_creator = canvas.texture_creator();
    let texture = texture_creator.load_texture("assets/shrimpy-0.png")?;


    let mut shrimp = Shrimp {
        position: Point::new(0, 0),
        sprite: Rect::new(0, 0, 286, 602),
        current_frame: 0,
    };

    let mut event_pump = sdl_context.event_pump()?;
    let mut i = 0;
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

        // Render
        render(&mut canvas, Color::RGB(i, 64, 255 - i), &texture, &shrimp)?;

        // Time management!
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
