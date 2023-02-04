use crate::{map::Ground, GameState};
use animations::{AnimationInfo, AnimationType, AnimationsPlugin};
use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

mod animations;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AnimationsPlugin)
            .add_system_set(
                SystemSet::on_update(GameState::MapExploring).with_system(player_movement_system),
            )
            .register_ldtk_entity::<PlayerBundle>("Player");
    }
}

const PLAYER_WIDTH: f32 = 60.0;
const PLAYER_HEIGHT: f32 = 110.0;
const JUMP_POWER: f32 = 250.0;
const RUN_POWER: f32 = 100.0;

#[derive(Default, Component)]
pub struct Player;

#[derive(Default, Bundle, LdtkEntity)]
struct PlayerBundle {
    #[sprite_sheet_bundle("player/player_tilesheet.png", 80.0, 110.0, 9, 3, 0.0, 0.0, 24)]
    #[bundle]
    sprite_sheet_bundle: SpriteSheetBundle,
    #[from_entity_instance]
    #[bundle]
    collider_bundle: ColliderBundle,
    velocity: Velocity,
    player: Player,
}

#[derive(Default, Bundle)]
struct ColliderBundle {
    collider: Collider,
    rigid_body: RigidBody,
    locked_axes: LockedAxes,
    friction: Friction,
    animation_info: AnimationInfo,
}

impl From<EntityInstance> for ColliderBundle {
    fn from(_: EntityInstance) -> Self {
        Self {
            collider: Collider::cuboid(PLAYER_WIDTH / 2.0, PLAYER_HEIGHT / 2.0),
            rigid_body: RigidBody::Dynamic,
            locked_axes: LockedAxes::ROTATION_LOCKED,
            friction: Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            animation_info: AnimationInfo::new(
                HashMap::from_iter([
                    (AnimationType::IDLE, vec![0]),
                    (AnimationType::RUN, vec![9, 10]),
                    (AnimationType::JUMP, vec![1]),
                    (AnimationType::FALL, vec![2]),
                ]),
                AnimationType::IDLE,
                Timer::from_seconds(0.2, true),
            ),
        }
    }
}

fn player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    rapier_context: Res<RapierContext>,
    mut player_info: Query<(Entity, &mut Velocity, &mut TextureAtlasSprite), With<Player>>,
    tile_info: Query<Entity, With<Ground>>,
) {
    let (player, mut velocity, mut sprite) = player_info.single_mut();
    let up: bool = keyboard_input.any_pressed([KeyCode::Up, KeyCode::W]);
    let left: bool = keyboard_input.any_pressed([KeyCode::Left, KeyCode::A]);
    let right: bool = keyboard_input.any_pressed([KeyCode::Right, KeyCode::D]);

    velocity.linvel.x = if left {
        sprite.flip_x = true;
        -RUN_POWER
    } else if right {
        sprite.flip_x = false;
        RUN_POWER
    } else {
        0.0
    };

    if up {
        for tile in tile_info.iter() {
            if let Some(contact_pair) = rapier_context.contact_pair(player, tile) {
                for manifold in contact_pair.manifolds() {
                    let first_entity = manifold
                        .rigid_body1()
                        .expect("An entity is expected when collision is detected!");
                    if (first_entity == player && manifold.normal().y == -1.0)
                        || manifold.normal().y == 1.0
                    {
                        velocity.linvel.y = JUMP_POWER;
                        return;
                    }
                }
            }
        }
    }
}
