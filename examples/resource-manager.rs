extern crate sdl3;

use sdl3::event::{Event, KeyState, KeyboardEvent};
use sdl3::image::{InitFlag, LoadTexture};
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::render::{Texture, TextureCreator};
use sdl3::ttf::{Font, Sdl3TtfContext};

use std::borrow::Borrow;
use std::collections::HashMap;
use std::env;
use std::hash::Hash;
use std::rc::Rc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: cargo run /path/to/image.(png|jpg) /path/to/font.(ttf|ttc|fon)")
    } else {
        let image_path = &args[1];
        let font_path = &args[2];

        let sdl_context = sdl3::init()?;
        let video_subsystem = sdl_context.video()?;
        let font_context = sdl3::ttf::init().map_err(|e| e.to_string())?;
        let _image_context = sdl3::image::init(InitFlag::PNG | InitFlag::JPG)?;
        let window = video_subsystem
            .window("rust-sdl3 resource-manager demo", 800, 600)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;
        let mut canvas = window
            .into_canvas()
            .software()
            .build()
            .map_err(|e| e.to_string())?;
        let texture_creator = canvas.texture_creator();
        let mut texture_manager = TextureManager::new(&texture_creator);
        let mut font_manager = FontManager::new(&font_context);
        let details = FontDetails {
            path: font_path.clone(),
            size: 32,
        };

        'mainloop: loop {
            for event in sdl_context.event_pump()?.poll_iter() {
                match event {
                    Event::Quit(_)
                    | Event::Keyboard(KeyboardEvent {
                        keycode: Some(Keycode::Escape),
                        state: KeyState::Down,
                        ..
                    }) => break 'mainloop,
                    _ => {}
                }
            }
            // will load the image texture + font only once
            let texture = texture_manager.load(image_path)?;
            let font = font_manager.load(&details)?;

            // not recommended to create a texture from the font each iteration
            // but it is the simplest thing to do for this example
            let surface = font
                .render("Hello Rust!")
                .blended(Color::RGBA(255, 0, 0, 255))
                .map_err(|e| e.to_string())?;
            let font_texture = texture_creator
                .create_texture_from_surface(&surface)
                .map_err(|e| e.to_string())?;

            //draw all
            canvas.clear();
            canvas.copy(&texture, None, None)?;
            canvas.copy(&font_texture, None, None)?;
            canvas.present();
        }
    }

    Ok(())
}

type TextureManager<'l, T> =
    ResourceManager<'l, Box<dyn std::error::Error>, Texture<'l>, TextureCreator<T>>;
type FontManager<'l> = ResourceManager<'l, FontDetails, Font<'l, 'static>, Sdl3TtfContext>;

// Generic struct to cache any resource loaded by a ResourceLoader
pub struct ResourceManager<'l, K, R, L>
where
    K: Hash + Eq,
    L: 'l + ResourceLoader<'l, R>,
{
    loader: &'l L,
    cache: HashMap<K, Rc<R>>,
}

impl<'l, K, R, L> ResourceManager<'l, K, R, L>
where
    K: Hash + Eq,
    L: ResourceLoader<'l, R>,
{
    pub fn new(loader: &'l L) -> Self {
        ResourceManager {
            cache: HashMap::new(),
            loader: loader,
        }
    }

    // Generics magic to allow a HashMap to use String as a key
    // while allowing it to use &str for gets
    pub fn load<D>(&mut self, details: &D) -> Result<Rc<R>, Box<dyn std::error::Error>>
    where
        L: ResourceLoader<'l, R, Args = D>,
        D: Eq + Hash + ?Sized,
        K: Borrow<D> + for<'a> From<&'a D>,
    {
        self.cache.get(details).cloned().map_or_else(
            || {
                let resource = Rc::new(self.loader.load(details)?);
                self.cache.insert(details.into(), resource.clone());
                Ok(resource)
            },
            Ok,
        )
    }
}

// TextureCreator knows how to load Textures
impl<'l, T> ResourceLoader<'l, Texture<'l>> for TextureCreator<T> {
    type Args = str;
    fn load(&'l self, path: &str) -> Result<Texture, Box<dyn std::error::Error>> {
        println!("LOADED A TEXTURE");
        self.load_texture(path)
    }
}

// Font Context knows how to load Fonts
impl<'l> ResourceLoader<'l, Font<'l, 'static>> for Sdl3TtfContext {
    type Args = FontDetails;
    fn load(
        &'l self,
        details: &FontDetails,
    ) -> Result<Font<'l, 'static>, Box<dyn std::error::Error>> {
        println!("LOADED A FONT");
        self.load_font(&details.path, details.size)
    }
}

// Generic trait to Load any Resource Kind
pub trait ResourceLoader<'l, R> {
    type Args: ?Sized;
    fn load(&'l self, data: &Self::Args) -> Result<R, Box<dyn std::error::Error>>;
}

// Information needed to load a Font
#[derive(PartialEq, Eq, Hash)]
pub struct FontDetails {
    pub path: String,
    pub size: u16,
}

impl<'a> From<&'a FontDetails> for FontDetails {
    fn from(details: &'a FontDetails) -> FontDetails {
        FontDetails {
            path: details.path.clone(),
            size: details.size,
        }
    }
}
