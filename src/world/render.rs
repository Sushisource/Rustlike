use crate::agents::Agent;
use crate::collision::Compound2D;
use crate::util::context_help::ContextHelp;
use crate::util::Assets;
use crate::world::World;
use ggez::event;
use ggez::event::KeyMods;
use ggez::graphics;
use ggez::graphics::{Color, DrawParam, Drawable};
use ggez::input::keyboard::KeyCode;
use ggez::input::mouse;
use ggez::timer;
use ggez::{Context, GameResult};
use nalgebra::Vector2;
use std;
use std::time::Duration;

pub struct WorldRender<'a> {
  world: &'a mut World,
  fastmode: bool,
  assets: Assets,
  debug: bool,
  // TODO: Move
  level_finished: bool,
}

impl<'a> WorldRender<'a> {
  pub fn new(world: &'a mut World, ctx: &mut Context) -> WorldRender<'a> {
    let assets = Assets::new(ctx);
    WorldRender { world, fastmode: true, assets, debug: false, level_finished: false }
  }

  fn stop_render(&mut self) -> () {
    self.world.level.level_gen_finished = true
  }
}

impl<'a> event::EventHandler for WorldRender<'a> {
  fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
    const DESIRED_FPS: u32 = 60;
    if !timer::check_update_time(ctx, DESIRED_FPS) {
      return Ok(());
    }

    // Tick the simulation TODO: Move elsewhere.
    if !self.world.level.level_gen_finished {
      // TODO: Configurable fastmode speed
      let i = if self.fastmode { 40 } else { 2 };
      for _ in 1..i {
        self.world.level.tick_level_gen();
      }
    } else if !self.level_finished {
      self.world.add_level_contents_to_collision();
      self.level_finished = true
    }
    Ok(())
  }

  fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
    graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 1.0));

    // First thing that is drawn is the level itself
    self.world.level.draw(ctx)?;
    // Render debug info that needs to be drawn at level scale
    if self.debug {
      // Render all collision bounding volumes
      for c in self.world.collision.collision_objects() {
        let shape_h = c.shape();
        if shape_h.is_shape::<Compound2D>() {
          let o_comp = shape_h.as_shape::<Compound2D>();
          if let Some(comp) = o_comp {
            for s in comp.shapes() {
              ctx.draw_bb(&s.1.aabb(&(s.0 * c.position())))?;
            }
          }
        } else {
          ctx.draw_bb(&c.shape().aabb(c.position()))?;
        }
      }
    }

    let scaler = self.world.level.lscale(ctx);
    graphics::set_transform(ctx, DrawParam::default().to_matrix());
    graphics::apply_transformations(ctx)?;
    self.world.player.draw(ctx, &mut self.assets, scaler.scale.into())?;

    // Textual debug info
    if self.debug {
      let mouse_p = mouse::position(ctx);
      let w_mouse_p = self.world.level.sspace_to_lspace(ctx, mouse_p.into());
      let dbg_txt = self.assets.txt(&format!("Mouse pos scrn: {:?} world: {}", mouse_p, w_mouse_p));
      dbg_txt.draw(ctx, DrawParam::default())?;
      self.world.collision_test(w_mouse_p);
    }

    graphics::present(ctx)?;
    timer::sleep(Duration::from_secs(0));
    Ok(())
  }

  // Handle key events. These just map keyboard events and alter our input
  // state appropriately.
  fn key_down_event(
    &mut self,
    _ctx: &mut Context,
    keycode: KeyCode,
    keymod: KeyMods,
    _repeat: bool,
  ) {
    match keycode {
      KeyCode::Space => {
        self.stop_render();
      }
      KeyCode::Add => {
        self.fastmode = !self.fastmode;
      }
      KeyCode::Up => {
        self.world.player.trans(Vector2::new(0.0, -1.0));
      }
      KeyCode::Down => {
        self.world.player.trans(Vector2::new(0.0, 1.0));
      }
      KeyCode::Left => {
        self.world.player.trans(Vector2::new(-1.0, 0.0));
      }
      KeyCode::Right => {
        self.world.player.trans(Vector2::new(1.0, 0.0));
      }
      KeyCode::Grave => {
        self.debug = !self.debug;
        info!("Debug mode now {}", self.debug);
      }
      KeyCode::Q if keymod.contains(KeyMods::CTRL) => {
        std::process::exit(0);
      }
      _ => (), // Do nothing
    }
  }
}
