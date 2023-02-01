use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use map::MapPlugin;
use player::PlayerPlugin;
use riddles::RiddlesPlugin;

mod map;
mod player;
mod riddles;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum GameState {
    MapExploring,
    RiddleSolving,
    LevelLoading,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_state(GameState::LevelLoading)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, -400.0),
            ..Default::default()
        })
        .add_plugin(LdtkPlugin)
        .add_startup_system(setup_system)
        .add_plugin(PlayerPlugin)
        .add_plugin(MapPlugin)
        .add_plugin(RiddlesPlugin)
        .run();
}

fn setup_system(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}
