use crate::{map::Ground, GameState};
use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::MapExploring)
                .with_system(player_movement_system)
                .with_system(animate_player_system),
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

#[derive(Default, Eq, PartialEq, Hash)]
enum AnimationType {
    #[default]
    IDLE,
    RUN,
    JUMP,
    FALL,
}

#[derive(Default, Component)]
struct AnimationInfo {
    animations: HashMap<AnimationType, Vec<usize>>,
    current_animation_type: AnimationType,
    current_animation: Vec<usize>,
    index: usize,
    timer: Timer,
}

impl AnimationInfo {
    fn new(
        animations: HashMap<AnimationType, Vec<usize>>,
        animation_type: AnimationType,
        timer: Timer,
    ) -> Self {
        let current_animation = animations
            .get(&animation_type)
            .expect("Animation type should have value in the map!")
            .clone();
        Self {
            animations,
            current_animation_type: animation_type,
            current_animation,
            index: 0,
            timer,
        }
    }

    fn set_animation(&mut self, animation_type: AnimationType) {
        if animation_type == self.current_animation_type {
            return;
        }
        if let Some(animation) = self.animations.get(&animation_type) {
            self.current_animation_type = animation_type;
            self.current_animation = animation.clone();
            self.index = 0;
            self.timer.set_elapsed(self.timer.duration());
        }
    }
}

fn animate_player_system(
    time: Res<Time>,
    mut animation_info: Query<(&mut TextureAtlasSprite, &mut AnimationInfo), With<Player>>,
) {
    let (mut sprite, mut animation_info) = animation_info.single_mut();
    if animation_info.timer.tick(time.delta()).just_finished() {
        animation_info.index = (animation_info.index + 1) % animation_info.current_animation.len();
        sprite.index = animation_info.current_animation[animation_info.index];
    }
}

fn player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    rapier_context: Res<RapierContext>,
    mut player_info: Query<
        (
            Entity,
            &mut Velocity,
            &mut TextureAtlasSprite,
            &mut AnimationInfo,
        ),
        With<Player>,
    >,
    tile_info: Query<Entity, With<Ground>>,
) {
    let (player, mut velocity, mut sprite, mut animation_info) = player_info.single_mut();
    let up: bool = keyboard_input.any_pressed([KeyCode::Up, KeyCode::W]);
    let left: bool = keyboard_input.any_pressed([KeyCode::Left, KeyCode::A]);
    let right: bool = keyboard_input.any_pressed([KeyCode::Right, KeyCode::D]);

    velocity.linvel.x = if left {
        sprite.flip_x = true;
        animation_info.set_animation(AnimationType::RUN);
        -RUN_POWER
    } else if right {
        sprite.flip_x = false;
        animation_info.set_animation(AnimationType::RUN);
        RUN_POWER
    } else {
        animation_info.set_animation(AnimationType::IDLE);
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
