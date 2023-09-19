use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::{WindowCanvas, Texture};
use sdl2::rect::{Point, Rect};
// "self" imports the "image" module itself as well as everything else we listed
use sdl2::image::{self, LoadTexture, InitFlag};
use std::time::Duration;

#[derive(Debug)]
struct Shrimp {
    position: Point,
    sprite: Rect,
    current_frame: i32,
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
        shrimp.sprite.x() + frame_width as i32 * shrimp.current_frame,
        //shrimp.sprite.y() + frame_height as i32 * direction_spritesheet_row(shrimp.direction),
        shrimp.sprite.y() + frame_height as i32,
        frame_width,
        frame_height,
        );


    //canvas.copy(texture, None, None)?;
    //canvas.copy(texture, Rect::new(0, 0, 286, 602), Rect::new(0, 0, 286, 602))?;
        // Treat the center of the screen as the (0, 0) coordinate
    let screen_position = shrimp.position + Point::new(width as i32 / 2, height as i32 / 2);
    let screen_rect = Rect::from_center(screen_position, frame_width, frame_height);
    canvas.copy(texture, current_frame, screen_rect)?;


    canvas.present();

    Ok(())
}

fn update_shrimp(shrimp: &mut Shrimp) {
    shrimp.current_frame = (shrimp.current_frame + 1) % 18;
}

fn main() -> Result<(), String> {
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
        update_shrimp(&mut shrimp);

        // Render
        render(&mut canvas, Color::RGB(i, 64, 255 - i), &texture, &shrimp)?;

        // Time management!
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
