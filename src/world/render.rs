extern crate ggez;

use std::time::Duration;
use self::ggez::{Context, GameResult};
use self::ggez::event;
use self::ggez::event::{Keycode, Mod};
use self::ggez::graphics;
use self::ggez::graphics::{Color, DrawParam, Drawable, Point2, Vector2};
use self::ggez::timer;
use self::ggez::mouse;
use world::World;
use agents::Agent;
use util::Assets;

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
    WorldRender {
      world,
      fastmode: true,
      assets,
      debug: true,
      level_finished: false,
    }
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
      let i = if self.fastmode { 12 } else { 2 };
      for _ in 1..i {
        self.world.level.tick_level_gen();
      }
    } else if !self.level_finished{
      self.world.add_level_contents_to_collision();
      self.level_finished = true
    }
    Ok(())
  }

  fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
    graphics::clear(ctx);

    // First thing that is drawn is the level itself
    self.world.level.draw(ctx)?;
    let scaler = self.world.level.lscale(ctx);

    graphics::set_transform(ctx, DrawParam::default().into_matrix());
    graphics::apply_transformations(ctx)?;
    graphics::set_color(ctx, Color::new(1.0, 1.0, 1.0, 1.0))?;
    self.world.player.draw(ctx, &mut self.assets, scaler.scale)?;

    // Debug info
    if self.debug {
      let mouse_p = mouse::get_position(ctx)?;
      let w_mouse_p = self.world.level.sspace_to_lspace(ctx, mouse_p);
      let dbg_txt = self.assets.txt(
        &format!("Mouse pos scrn: {} world: {}", mouse_p, w_mouse_p),
        ctx,
      );
      dbg_txt.draw(ctx, Point2::new(0.0, 0.0), 0.0)?;
      self.world.collision_test(&w_mouse_p);
    }

    graphics::present(ctx);
    timer::sleep(Duration::from_secs(0));
    Ok(())
  }

  // Handle key events. These just map keyboard events and alter our input
  // state appropriately.
  fn key_down_event(
    &mut self,
    _ctx: &mut Context,
    keycode: Keycode,
    _keymod: Mod,
    _repeat: bool,
  ) {
    match keycode {
      Keycode::Space => {
        self.stop_render();
      }
      Keycode::Plus | Keycode::KpPlus => {
        self.fastmode = !self.fastmode;
      }
      Keycode::Up => {
        self.world.player.trans(Vector2::new(0.0, -1.0));
      }
      Keycode::Down => {
        self.world.player.trans(Vector2::new(0.0, 1.0));
      }
      Keycode::Left => {
        self.world.player.trans(Vector2::new(-1.0, 0.0));
      }
      Keycode::Right => {
        self.world.player.trans(Vector2::new(1.0, 0.0));
      }
      Keycode::Backquote => {
        self.debug = !self.debug;
        println!("Debug mode now {}", self.debug);
      }
      _ => (), // Do nothing
    }
  }
}
