use crate::riddles::RiddleInfo;
use crate::GameState;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(map_setup_system)
            .insert_resource(CurrentLevel::from("6c6ef290-5110-11ed-90f2-ab2793fe3460"))
            .add_system_set(
                SystemSet::on_enter(GameState::LevelLoading).with_system(level_loading_system),
            )
            .add_system_set(
                SystemSet::on_update(GameState::LevelLoading).with_system(level_loaded_system),
            )
            .add_system_set(SystemSet::on_exit(GameState::LevelLoading).with_system(center_map))
            .register_ldtk_entity::<GroundTile>("Ground")
            .register_ldtk_entity::<GroundTile>("LevelBorder")
            .register_ldtk_entity::<Door>("Door")
            .register_ldtk_entity::<BoxTile>("Box");
    }
}

const SMALL_TILE_SIZE: f32 = 16.0;
const LARGE_TILE_SIZE: f32 = 64.0;

pub type CurrentLevel = String;

fn map_setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(LdtkWorldBundle {
        ldtk_handle: asset_server.load("map.ldtk"),
        ..default()
    });
}

fn level_loading_system(
    current_level: Res<CurrentLevel>,
    mut level_set_info: Query<&mut LevelSet>,
) {
    let mut level_set = level_set_info.single_mut();
    level_set.iids.clear();
    level_set.iids.insert(current_level.clone());
}

fn level_loaded_system(mut state: ResMut<State<GameState>>, mut events: EventReader<LevelEvent>) {
    for event in events.iter() {
        if let LevelEvent::Spawned(_) = event {
            state.set(GameState::MapExploring).unwrap();
        }
    }
}

fn center_map(
    levels: Res<Assets<LdtkLevel>>,
    mut map_info: Query<(&Handle<LdtkLevel>, &mut Transform)>,
) {
    let (handle, mut map) = map_info.single_mut();
    let level = levels.get(handle).unwrap();

    map.translation.x = -level.level.px_wid as f32 / 2.0;
    map.translation.y = -level.level.px_hei as f32 / 2.0;
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
