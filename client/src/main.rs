use avian3d::prelude::*;
use bevy::{
  dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
  prelude::*,
  text::FontSmoothing,
  window::{PresentMode, WindowPlugin, WindowTheme},
};

use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod game_states;
mod systems;
use game_states::game::GamePlugin;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
  #[default]
  Game,
}
struct OverlayColor;

impl OverlayColor {
  const RED: Color = Color::srgb(1.0, 0.0, 0.0);
  const GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
}

fn main() {
  App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
      primary_window: Some(Window {
        title: "I am a window!".into(),
        name: Some("bevy.app".into()),
        //mode: WindowMode::Fullscreen(MonitorSelection::Primary),
        present_mode: PresentMode::AutoVsync,
        window_theme: Some(WindowTheme::Dark),
        ..default()
      }),
      ..default()
    }))
    .init_state::<GameState>()
    .add_plugins((
      FpsOverlayPlugin {
        config: FpsOverlayConfig {
          text_config: TextFont {
            // Here we define size of our overlay
            font_size: 42.0,
            // If we want, we can use a custom font
            font: default(),
            // We could also disable font smoothing,
            font_smoothing: FontSmoothing::default(),
          },
          // We can also change color of the overlay
          text_color: OverlayColor::GREEN,
          enabled: true,
        },
      },
      WorldInspectorPlugin::new(),
      PhysicsPlugins::default(),
      GamePlugin,
    ))
    .run();
}
