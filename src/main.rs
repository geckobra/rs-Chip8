use std::fs;
use std::io::ErrorKind;
use std::time::{Duration, Instant};
use ggez::{
    event,
    glam::*,
    graphics::{self,Color},
    Context,GameResult,
    graphics::DrawMode,
};

use ggez::input::keyboard::{KeyCode, KeyMods, KeyInput};
use ggez::audio::{self,SoundSource};

mod cpu;
mod font;

const MEM_OFFSET: usize = 0x200;
const FONT_BASE_ADDRESS: usize = 0x50;

struct MainState{
    chip8: cpu::Device,
    display_mesh: graphics::Mesh,
    needs_redraw: bool,
    beep_sound: audio::Source,
}

impl MainState{
    fn new(ctx: &mut ggez::Context) -> Self{
        let dummy_mesh = graphics::Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            Vec2::ZERO,
            0.0,
            0.1,
            Color::WHITE,
        ).unwrap();

        let beep_sound = audio::Source::new(ctx,"/assets/beep.wav").unwrap();
        
        MainState{
            chip8: cpu::Device::new(),
            display_mesh: dummy_mesh,
            needs_redraw: false,
            beep_sound,
        }
    }

    fn build_display_mesh(&mut self, ctx: &mut ggez::Context) -> GameResult{

        let mut mb = graphics::MeshBuilder::new();

        for x in 0..64{
           for y in 0..32{
               if (self.chip8.display[x][y]) == 1{
                    let rect = ggez::graphics::Rect::new(
                       (x as f32) * 10.0,
                       (y as f32) * 10.0 + 20.0,
                       10.0,
                       10.0,
                    );
                    mb.rectangle(DrawMode::fill(),rect,graphics::Color::WHITE,)?; 
               }       
           }
        } 

        let mesh_data = mb.build(); // returns MeshData
        self.display_mesh = graphics::Mesh::from_data(ctx, mesh_data);

        Ok(())
    }

    fn update_keyboard(&mut self, ctx:&ggez::Context){
        let k_ctx = &ctx.keyboard;

        self.chip8.keyboard[0x0] = k_ctx.is_key_pressed(KeyCode::Key1);
        self.chip8.keyboard[0x1] = k_ctx.is_key_pressed(KeyCode::Key2);
        self.chip8.keyboard[0x2] = k_ctx.is_key_pressed(KeyCode::Up);
        self.chip8.keyboard[0x3] = k_ctx.is_key_pressed(KeyCode::Key4);
        self.chip8.keyboard[0x4] = k_ctx.is_key_pressed(KeyCode::Left);
        self.chip8.keyboard[0x5] = k_ctx.is_key_pressed(KeyCode::W);
        self.chip8.keyboard[0x6] = k_ctx.is_key_pressed(KeyCode::Right);
        self.chip8.keyboard[0x7] = k_ctx.is_key_pressed(KeyCode::R);
        self.chip8.keyboard[0x8] = k_ctx.is_key_pressed(KeyCode::Down);
        self.chip8.keyboard[0x9] = k_ctx.is_key_pressed(KeyCode::S);
        self.chip8.keyboard[0xA] = k_ctx.is_key_pressed(KeyCode::D);
        self.chip8.keyboard[0xB] = k_ctx.is_key_pressed(KeyCode::F);
        self.chip8.keyboard[0xC] = k_ctx.is_key_pressed(KeyCode::Z);
        self.chip8.keyboard[0xD] = k_ctx.is_key_pressed(KeyCode::X);
        self.chip8.keyboard[0xF] = k_ctx.is_key_pressed(KeyCode::V);
    }
}

impl event::EventHandler<ggez::GameError> for MainState{
    fn update(&mut self, ctx: &mut ggez::Context) -> GameResult{
        self.update_keyboard(ctx);

        for i in 0..10{
            let instruction = self.chip8.fetch();
            self.chip8.decode(instruction);
        }
        
        self.chip8.update_timers();

        if self.chip8.display_changed{
            self.needs_redraw = true;
            self.chip8.display_changed = false;
        }

        if self.chip8.sound_timer>0{
            if !self.beep_sound.playing(){
                self.beep_sound.play(ctx);
            }
            else{
                self.beep_sound.stop(ctx);
            }
        }
        
        Ok(())
    }
    
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        if self.needs_redraw{
            self.build_display_mesh(ctx)?;
            self.needs_redraw = false;
        }

        graphics::draw(&mut canvas, &self.display_mesh, graphics::DrawParam::default());
        canvas.finish(ctx)?;
        Ok(())
    }
}

fn load_ROM(rom_name: &str, chip8: &mut cpu::Device){
    let f = fs::read("ROMS/".to_owned() + rom_name + ".ch8").unwrap_or_else(|error| {
        if error.kind() == ErrorKind::NotFound{
            panic!("That ROM is not present in the ROMS/ folder.\
                 Check the name provided or move the ROM there");
        }
        else{
            panic!("Error while reading the file: {error:?}");
        }
    });

    if f.len() > chip8.memory.len() - MEM_OFFSET{
        panic!("THAT'S A BIG FILE RIGHT THERE, WON'T FIT");
    }

    chip8.memory[FONT_BASE_ADDRESS..(FONT_BASE_ADDRESS + font::CHIP8_FONTSET.len())].copy_from_slice(&font::CHIP8_FONTSET);

    chip8.memory[MEM_OFFSET..(MEM_OFFSET + f.len())].copy_from_slice(&f);
}


fn main() -> GameResult{
    let rom_name = std::env::args().nth(1).expect("No ROM name provided");

    let (mut ctx, event_loop) = ggez::ContextBuilder::new("Chip8-Emu", "Ferni")
        .add_resource_path("./resources")
        .build()?;
   
    //make mutable reference to main state
    let mut main_state = MainState::new(&mut ctx);

    load_ROM(rom_name.as_str(),&mut main_state.chip8);

    //start the game
    ggez::event::run(ctx, event_loop, main_state);
}
