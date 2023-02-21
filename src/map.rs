use crate::riddles::RiddleInfo;
use crate::GameState;
use bevy::{prelude::*, text::Text2dBounds, utils::HashMap};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(map_setup_system)
            .insert_resource(CurrentLevel::from(STARTING_LEVEL))
            .add_system_set(
                SystemSet::on_enter(GameState::LevelLoading).with_system(level_loading_system),
            )
            .add_system_set(
                SystemSet::on_update(GameState::LevelLoading).with_system(level_loaded_system),
            )
            .add_system_set(
                SystemSet::on_exit(GameState::LevelLoading).with_system(normalize_font_system),
            )
            .add_system_set(SystemSet::on_exit(GameState::LevelLoading).with_system(center_map))
            .register_ldtk_entity::<GroundTile>("Ground")
            .register_ldtk_entity::<LevelBorder>("LevelBorder")
            .register_ldtk_entity::<Door>("Door")
            .register_ldtk_entity::<BoxTile>("Box")
            .register_ldtk_entity::<TextSignBundle>("TextSign");
    }
}

const STARTING_LEVEL: &str = "6c6ef290-5110-11ed-90f2-ab2793fe3460";
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

#[derive(Default, Bundle, LdtkEntity)]
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

#[derive(Default, Bundle, LdtkEntity)]
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

#[derive(Default, Component)]
struct StaticText;

#[derive(LdtkEntity, Bundle)]
struct TextSignBundle {
    #[from_entity_instance]
    #[bundle]
    text_sign: TextSign,
    static_text: StaticText,
}

#[derive(Bundle)]
struct TextSign {
    #[bundle]
    text_2d_bundle: Text2dBundle,
}

impl From<EntityInstance> for TextSign {
    fn from(entity_instance: EntityInstance) -> Self {
        let fields = HashMap::from_iter(entity_instance.field_instances.iter().map(|field| {
            (
                field.identifier.clone(),
                match field.value.clone() {
                    FieldValue::String(Some(value)) => value,
                    FieldValue::Float(Some(value)) => value.to_string(),
                    _ => "".to_string(),
                },
            )
        }));
        Self {
            text_2d_bundle: Text2dBundle {
                text: Text::from_section(
                    fields
                        .get("text")
                        .expect("Text is expected for a text sign!"),
                    TextStyle {
                        font_size: fields
                            .get("font_size")
                            .expect("The font size of the text is expected!")
                            .parse()
                            .expect("Font size is expected to be a number!"),
                        color: entity_instance
                            .field_instances
                            .iter()
                            .find(|field| field.identifier == "color")
                            .map(|field| {
                                if let FieldValue::Color(color) = field.value.clone() {
                                    color
                                } else {
                                    Color::default()
                                }
                            })
                            .expect("Default color is expected to be set by the editor!"),
                        ..Default::default()
                    },
                )
                .with_alignment(TextAlignment::CENTER),
                text_2d_bounds: Text2dBounds {
                    size: Vec2::new(entity_instance.width as f32, entity_instance.height as f32),
                },
                ..Default::default()
            },
        }
    }
}

fn normalize_font_system(
    asset_server: Res<AssetServer>,
    mut text_query: Query<(&mut Text, &mut Transform), With<StaticText>>,
) {
    for (mut text, mut transform) in text_query.iter_mut() {
        text.sections[0].style.font =
            asset_server.load("fonts/MontserratAlternates-MediumItalic.ttf");
        transform.scale = Vec3::new(1.0, 1.0, 1.0);
    }
}
