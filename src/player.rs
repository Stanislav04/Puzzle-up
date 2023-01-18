use crate::{map::Ground, GameState};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(player_setup_system).add_system_set(
            SystemSet::on_update(GameState::MapExploring).with_system(player_movement_system),
        );
    }
}

#[derive(Component)]
pub struct Player;

const JUMP_POWER: f32 = 100.0;

fn player_setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("player/player_idle.png"),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 10.0),
                scale: Vec3::new(0.5, 0.5, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Player)
        .insert(Velocity::default())
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(30.0, 55.0))
        .insert(LockedAxes::ROTATION_LOCKED);
}

fn player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    rapier_context: Res<RapierContext>,
    mut player_info: Query<(Entity, &mut Velocity), With<Player>>,
    tile_info: Query<Entity, With<Ground>>,
) {
    let (player, mut velocity) = player_info.single_mut();
    let up: bool = keyboard_input.any_pressed([KeyCode::Up, KeyCode::W]);
    let left: bool = keyboard_input.any_pressed([KeyCode::Left, KeyCode::A]);
    let right: bool = keyboard_input.any_pressed([KeyCode::Right, KeyCode::D]);

    velocity.linvel.x += -(left as i8 as f32) + right as i8 as f32;

    if up {
        'outer: for tile in tile_info.iter() {
            if let Some(contact_pair) = rapier_context.contact_pair(player, tile) {
                for manifold in contact_pair.manifolds() {
                    if manifold.normal().y == -1.0 {
                        velocity.linvel.y += (up as i8 as f32) * JUMP_POWER;
                        break 'outer;
                    }
                }
            }
        }
    }
}
