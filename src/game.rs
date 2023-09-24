use sdl2::event::Event;
use sdl2::image::Sdl2ImageContext;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::{image, EventPump};
use std::time::{Duration, Instant};

pub struct GameConfig {
    target_fps: u32,
    target_frame_duration: Duration,
    game_size: (u32, u32),
}

pub struct Game {
    event_pump: EventPump,
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    img_ctx: Sdl2ImageContext,
    prev_frame: Option<Instant>,
    is_running: bool,
    game_config: GameConfig,
}

impl Game {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().expect("Couldn't create SDL2 context");

        let video_subsystem = sdl_context.video().expect("Couldn't start video subsystem");

        let img_ctx =
            image::init(image::InitFlag::all()).expect("Couldn't initialize image context");

        let game_size = (1280, 720);

        let window = video_subsystem
            .window("Rust sdl2", game_size.0, game_size.1)
            .fullscreen_desktop()
            .build()
            .expect("Couldn't initialize window");

        let canvas = window
            .into_canvas()
            .present_vsync()
            .accelerated()
            .build()
            .expect("Couldn't create canvas");
        println!("{:?}", canvas.info());

        let texture_creator = canvas.texture_creator();

        let event_pump = sdl_context
            .event_pump()
            .expect("Couldn't create event_pump");

        let target_fps = 60;

        Self {
            canvas,
            texture_creator,
            img_ctx,
            event_pump,
            is_running: true,
            prev_frame: None,
            game_config: GameConfig {
                target_fps,
                game_size,
                target_frame_duration: Duration::from_millis(1000 / target_fps as u64),
            },
        }
    }

    fn process_input(&mut self) {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    self.is_running = false;
                }
                _ => {}
            }
        }
    }

    fn load(&mut self) {
        // TODO:
        // let registry = Registry::new();
        // let tank = registry.create_entity();
        // tank.add_component::<TransformComponent>();
        // tank.add_component::<BoxColliderComponent>();
        // tank.add_component::<SpriteComponent>("./assets/images/tank.png");
    }

    fn update(&mut self) {
        if self.prev_frame.is_none() {
            self.prev_frame = Some(Instant::now());
        }
        let elapsed = self.prev_frame.unwrap().elapsed();
        let dt = elapsed.as_secs_f32();

        // TODO:
        // movement_system.update();
        // movement_system.update();
        // movement_system.update();

        if elapsed < self.game_config.target_frame_duration {
            std::thread::sleep(self.game_config.target_frame_duration - elapsed);
        }
        self.prev_frame = Some(Instant::now());
    }

    fn draw(&mut self) {
        self.canvas.set_draw_color(Color::RGBA(21, 21, 21, 255));
        self.canvas.clear();

        // TODO: render game objects...

        self.canvas.present();
    }

    pub fn run(&mut self) {
        self.load();
        while self.is_running {
            self.process_input();
            self.update();
            self.draw();
        }
    }
}
