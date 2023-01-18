use crate::riddles::RiddleInfo;
use crate::GameState;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(map_setup_system)
            .insert_resource(LevelSelection::Index(0))
            .add_system_set(
                SystemSet::on_update(GameState::LevelLoading).with_system(level_loaded_system),
            )
            .register_ldtk_entity::<GroundTile>("Ground")
            .register_ldtk_entity::<GroundTile>("LevelBorder")
            .register_ldtk_entity::<Door>("Door")
            .register_ldtk_entity::<BoxTile>("Box");
    }
}

const SMALL_TILE_SIZE: f32 = 16.0;
const LARGE_TILE_SIZE: f32 = 64.0;
const MAP_WIDTH: f32 = 736.0;
const MAP_HEIGHT: f32 = 384.0;

fn map_setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(LdtkWorldBundle {
        ldtk_handle: asset_server.load("map.ldtk"),
        transform: Transform::from_xyz(-MAP_WIDTH / 2.0, -MAP_HEIGHT / 2.0, 0.0),
        ..default()
    });
}

fn level_loaded_system(mut state: ResMut<State<GameState>>, doors: Query<&RiddleInfo>) {
    if doors.iter().count() == 2 {
        state.set(GameState::MapExploring).unwrap();
    }
}

#[derive(Default, Component)]
pub struct Ground;

#[derive(Default, Bundle, LdtkEntity, Component)]
struct GroundTile {
    #[from_entity_instance]
    #[bundle]
    collider_bundle: ColliderBundle,
    ground: Ground,
}

#[derive(Default, Bundle, LdtkEntity)]
struct LevelBorder {
    #[from_entity_instance]
    #[bundle]
    collider_bundle: ColliderBundle,
}

#[derive(Default, Bundle, LdtkEntity)]
struct BoxTile {
    #[sprite_sheet_bundle]
    #[bundle]
    sprite_sheet_bundle: SpriteSheetBundle,
    #[from_entity_instance]
    #[bundle]
    collider_bundle: ColliderBundle,
    ground: Ground,
}

#[derive(Default, Bundle, LdtkEntity, Component)]
struct Door {
    #[sprite_sheet_bundle]
    #[bundle]
    sprite_sheet_bundle: SpriteSheetBundle,
    #[from_entity_instance]
    #[bundle]
    collider_bundle: ColliderBundle,
    sensor: Sensor,
    #[from_entity_instance]
    riddle_info: RiddleInfo,
}

#[derive(Default, Bundle)]
struct ColliderBundle {
    collider: Collider,
    rigid_body: RigidBody,
}

impl From<EntityInstance> for ColliderBundle {
    fn from(entity_instance: EntityInstance) -> Self {
        match entity_instance.identifier.as_ref() {
            "Ground" | "LevelBorder" => Self {
                collider: Collider::cuboid(SMALL_TILE_SIZE / 2.0, SMALL_TILE_SIZE / 2.0),
                rigid_body: RigidBody::Fixed,
            },
            "Door" | "Box" => Self {
                collider: Collider::cuboid(LARGE_TILE_SIZE / 2.0, LARGE_TILE_SIZE / 2.0),
                rigid_body: RigidBody::Fixed,
            },
            _ => Self::default(),
        }
    }
}
