use super::Player;
use crate::GameState;
use bevy::{prelude::*, utils::HashMap};
use bevy_rapier2d::prelude::*;

pub struct AnimationsPlugin;

impl Plugin for AnimationsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::MapExploring)
                .with_system(animate_player_system)
                .with_system(idle_animation_trigger_system)
                .with_system(run_animation_trigger_system)
                .with_system(jump_animation_trigger_system)
                .with_system(fall_animation_trigger_system),
        );
    }
}

#[derive(Default, Eq, PartialEq, Hash)]
pub enum AnimationType {
    #[default]
    IDLE,
    RUN,
    JUMP,
    FALL,
}

#[derive(Default, Component)]
pub struct AnimationInfo {
    animations: HashMap<AnimationType, Vec<usize>>,
    current_animation_type: AnimationType,
    current_animation: Vec<usize>,
    index: usize,
    timer: Timer,
}

impl AnimationInfo {
    pub fn new(
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

fn idle_animation_trigger_system(
    rapier_context: Res<RapierContext>,
    mut animation_info: Query<(Entity, &mut AnimationInfo, &Velocity), With<Player>>,
) {
    let (player, mut animation_info, velocity) = animation_info.single_mut();
    if [AnimationType::IDLE, AnimationType::JUMP].contains(&animation_info.current_animation_type) {
        return;
    }
    if velocity.linvel.x != 0.0 {
        return;
    }
    for contact_pair in rapier_context.contacts_with(player) {
        for manifold in contact_pair.manifolds() {
            if manifold.normal().y == 0.0 {
                continue;
            }
            animation_info.set_animation(AnimationType::IDLE);
            return;
        }
    }
}

fn run_animation_trigger_system(
    rapier_context: Res<RapierContext>,
    mut animation_info: Query<(Entity, &mut AnimationInfo, &Velocity), With<Player>>,
) {
    let (player, mut animation_info, velocity) = animation_info.single_mut();
    if animation_info.current_animation_type == AnimationType::JUMP {
        return;
    }
    if velocity.linvel.x == 0.0 {
        return;
    }
    for contact_pair in rapier_context.contacts_with(player) {
        for manifold in contact_pair.manifolds() {
            if manifold.normal().y == 0.0 {
                continue;
            }
            animation_info.set_animation(AnimationType::RUN);
            return;
        }
    }
}

fn jump_animation_trigger_system(
    mut events: EventReader<CollisionEvent>,
    mut animation_info: Query<(&mut AnimationInfo, &Velocity), With<Player>>,
) {
    let (mut animation_info, velocity) = animation_info.single_mut();
    if animation_info.current_animation_type == AnimationType::JUMP {
        return;
    }
    for event in events.iter() {
        if let CollisionEvent::Stopped(_, _, flag) = event {
            if !flag.is_empty() {
                continue;
            }
            if velocity.linvel.y < 0.0 {
                continue;
            }
            animation_info.set_animation(AnimationType::JUMP);
            return;
        }
    }
}

fn fall_animation_trigger_system(
    rapier_context: Res<RapierContext>,
    mut animation_info: Query<(Entity, &mut AnimationInfo, &Velocity), With<Player>>,
) {
    let (player, mut animation_info, velocity) = animation_info.single_mut();
    if animation_info.current_animation_type == AnimationType::FALL {
        return;
    }
    if velocity.linvel.y > 0.0 {
        return;
    }
    for contact_pair in rapier_context.contacts_with(player) {
        for manifold in contact_pair.manifolds() {
            if manifold.normal().y != 0.0 {
                return;
            }
        }
    }
    animation_info.set_animation(AnimationType::FALL);
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
