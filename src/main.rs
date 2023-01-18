use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use player::{Player, PlayerPlugin};

mod player;

const TILE_SIZE: f32 = 16.0;
const DOOR_SIZE: f32 = 64.0;
const MAP_WIDTH: f32 = 736.0;
const MAP_HEIGHT: f32 = 384.0;

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
            gravity: Vec2::new(0.0, -50.0),
            ..Default::default()
        })
        .add_plugin(LdtkPlugin)
        .insert_resource(LevelSelection::Index(0))
        .insert_resource(AnsweredRiddles::new())
        .add_startup_system(setup_system)
        .add_plugin(PlayerPlugin)
        .add_system_set(
            SystemSet::on_update(GameState::MapExploring).with_system(touch_door_system),
        )
        .add_system_set(
            SystemSet::on_update(GameState::RiddleSolving)
                .with_system(answering_riddle_system)
                .with_system(correct_answer_system)
                .with_system(close_riddle_system),
        )
        .add_system_set(
            SystemSet::on_update(GameState::LevelLoading).with_system(level_loaded_system),
        )
        .add_system_set(
            SystemSet::on_exit(GameState::LevelLoading).with_system(init_riddles_system),
        )
        .register_ldtk_entity::<GroundTile>("Ground")
        .register_ldtk_entity::<GroundTile>("LevelBorder")
        .register_ldtk_entity::<Door>("Door")
        .register_ldtk_entity::<BoxTile>("Box")
        .run();
}

fn setup_system(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    commands.spawn_bundle(LdtkWorldBundle {
        ldtk_handle: asset_server.load("map.ldtk"),
        transform: Transform::from_xyz(-MAP_WIDTH / 2.0, -MAP_HEIGHT / 2.0, 0.0),
        ..default()
    });
}

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

#[derive(Default, Component)]
struct Ground;

#[derive(Default, Bundle)]
struct ColliderBundle {
    collider: Collider,
    rigid_body: RigidBody,
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

impl From<EntityInstance> for ColliderBundle {
    fn from(entity_instance: EntityInstance) -> Self {
        match entity_instance.identifier.as_ref() {
            "Ground" => Self {
                collider: Collider::cuboid(TILE_SIZE / 2.0, TILE_SIZE / 2.0),
                rigid_body: RigidBody::Fixed,
            },
            "LevelBorder" => Self {
                collider: Collider::cuboid(TILE_SIZE / 2.0, TILE_SIZE / 2.0),
                rigid_body: RigidBody::Fixed,
            },
            "Door" | "Box" => Self {
                collider: Collider::cuboid(DOOR_SIZE / 2.0, DOOR_SIZE / 2.0),
                rigid_body: RigidBody::Fixed,
            },
            _ => Self::default(),
        }
    }
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

#[derive(Default, Component)]
struct RiddleInfo {
    question: String,
    answer: String,
    riddle: Option<Entity>,
    active: bool,
}

#[derive(Component)]
struct RiddleNode;

#[derive(Component)]
struct AnswerContainer {
    index: usize,
    answer_length: usize,
}

#[derive(Component)]
struct Answer {
    position: usize,
}

type AnsweredRiddles = HashSet<String>;

impl From<EntityInstance> for RiddleInfo {
    fn from(entity_instance: EntityInstance) -> Self {
        let fields = HashMap::from_iter(entity_instance.field_instances.iter().map(|field| {
            (
                field.identifier.clone(),
                match field.value.clone() {
                    FieldValue::String(Some(value)) => value,
                    _ => "".to_string(),
                },
            )
        }));
        Self {
            question: fields
                .get("question")
                .expect("A question is required for a riddle!")
                .clone(),
            answer: fields
                .get("answer")
                .expect("An answer is required for a riddle!")
                .clone(),
            ..Default::default()
        }
    }
}

fn root_node() -> NodeBundle {
    NodeBundle {
        style: Style {
            display: Display::None,
            flex_direction: FlexDirection::ColumnReverse,
            justify_content: JustifyContent::SpaceAround,
            align_items: AlignItems::Center,
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            ..Default::default()
        },
        color: UiColor::from(Color::rgb(0.5, 0.5, 0.85)),
        visibility: Visibility { is_visible: false },
        ..Default::default()
    }
}

fn question_text(asset_server: &Res<AssetServer>, question: &String) -> TextBundle {
    TextBundle {
        text: Text::from_section(
            question,
            TextStyle {
                font: asset_server.load("MontserratAlternates-MediumItalic.ttf"),
                font_size: 60.0,
                color: Color::WHITE,
            },
        )
        .with_alignment(TextAlignment::CENTER),
        ..Default::default()
    }
}

fn answer_container() -> NodeBundle {
    NodeBundle {
        color: UiColor::from(Color::NONE),
        style: Style {
            justify_content: JustifyContent::SpaceAround,
            min_size: Size::new(Val::Percent(30.0), Val::Auto),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn answer_position(asset_server: &Res<AssetServer>, color: Color) -> TextBundle {
    TextBundle {
        text: Text::from_section(
            "_".to_string(),
            TextStyle {
                font: asset_server.load("fonts/MontserratAlternates-MediumItalic.ttf"),
                font_size: 60.0,
                color,
            },
        ),
        ..Default::default()
    }
}

fn level_loaded_system(mut state: ResMut<State<GameState>>, doors: Query<&RiddleInfo>) {
    if doors.iter().count() == 2 {
        state.set(GameState::MapExploring).unwrap();
    }
}

fn init_riddles_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    answered_riddles: Res<AnsweredRiddles>,
    mut doors: Query<&mut RiddleInfo>,
) {
    for mut door in doors.iter_mut() {
        if answered_riddles.contains(&door.question) {
            continue;
        }
        door.riddle = Some(
            commands
                .spawn_bundle(root_node())
                .insert(RiddleNode)
                .with_children(|parent| {
                    parent.spawn_bundle(question_text(&asset_server, &door.question));
                    parent
                        .spawn_bundle(answer_container())
                        .insert(AnswerContainer {
                            index: 0,
                            answer_length: door.answer.len(),
                        })
                        .with_children(|parent| {
                            parent
                                .spawn_bundle(answer_position(&asset_server, Color::RED))
                                .insert(Answer { position: 0 });
                            parent
                                .spawn_bundle(answer_position(&asset_server, Color::BLUE))
                                .insert(Answer { position: 1 });
                            parent
                                .spawn_bundle(answer_position(&asset_server, Color::YELLOW))
                                .insert(Answer { position: 2 });
                        });
                })
                .id(),
        );
    }
}

fn touch_door_system(
    keyboard_input: Res<Input<KeyCode>>,
    rapier_context: Res<RapierContext>,
    answered_riddles: Res<AnsweredRiddles>,
    mut state: ResMut<State<GameState>>,
    player_info: Query<Entity, With<Player>>,
    mut doors: Query<(Entity, &mut RiddleInfo)>,
    mut riddle_nodes: Query<(&mut Style, &mut Visibility), With<RiddleNode>>,
) {
    let player = player_info.single();
    if keyboard_input.just_pressed(KeyCode::Space) {
        for (door, mut riddle_info) in doors.iter_mut() {
            if let Some(contact_pair) = rapier_context.intersection_pair(player, door) {
                if !contact_pair {
                    continue;
                }
                if !answered_riddles.contains(&riddle_info.question) {
                    let (mut node_style, mut node_visibility) = riddle_nodes
                        .get_mut(riddle_info.riddle.expect(
                            "The riddle entity is supposed to be set by the init_riddles_system!",
                        ))
                        .unwrap();
                    node_style.display = Display::Flex;
                    node_visibility.is_visible = true;
                    riddle_info.active = true;
                    state.set(GameState::RiddleSolving).unwrap();
                } else {
                    todo!("Going to the next level!");
                }
            }
        }
    }
}

fn answering_riddle_system(
    mut input: EventReader<ReceivedCharacter>,
    mut container_info: Query<(&mut AnswerContainer, &ComputedVisibility)>,
    mut answer_nodes: Query<(&mut Text, &ComputedVisibility, &Answer)>,
) {
    for character in input.iter() {
        if !('0'..='9').contains(&character.char) {
            continue;
        }
        let (mut container, _) = container_info
            .iter_mut()
            .filter(|(_, visibility)| visibility.is_visible())
            .next()
            .expect("A visible container is expected while this system is running!");
        let (mut answer, _, _) = answer_nodes
            .iter_mut()
            .filter(|(_, visibility, _)| visibility.is_visible())
            .find(|(_, _, answer)| answer.position == container.index)
            .expect("The container is expected to have answer positions and the container's index is always valid!");
        answer.sections[0].value = character.char.to_string();
        container.index = (container.index + 1) % container.answer_length;
    }
}

fn correct_answer_system(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut answered_riddles: ResMut<AnsweredRiddles>,
    mut state: ResMut<State<GameState>>,
    mut doors: Query<(&mut RiddleInfo, &mut TextureAtlasSprite)>,
    answer_nodes: Query<(&Text, &ComputedVisibility, &Answer)>,
) {
    if keyboard_input.any_just_pressed([KeyCode::Return, KeyCode::NumpadEnter]) {
        let mut answer_nodes = Vec::from_iter(
            answer_nodes
                .iter()
                .filter(|(_, visibility, _)| visibility.is_visible())
                .map(|(text, _, pos)| (pos.position, text.sections[0].value.clone())),
        );
        answer_nodes.sort_by_key(|(pos, _)| *pos);
        let answer = answer_nodes
            .into_iter()
            .map(|(_, value)| value)
            .collect::<String>();
        let (mut door, mut sprite) = doors
            .iter_mut()
            .filter(|(door, _)| door.active)
            .next()
            .expect("Only one door should be active while answering a riddle!");
        if answer != door.answer {
            return;
        }
        answered_riddles.insert(door.question.clone());
        commands
            .entity(
                door.riddle
                    .expect("The riddle entity is supposed to be set by the init_riddles_system!"),
            )
            .despawn_recursive();
        sprite.index = 75;
        door.active = false;
        state.set(GameState::MapExploring).unwrap();
    }
}

fn close_riddle_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut state: ResMut<State<GameState>>,
    mut doors: Query<&mut RiddleInfo>,
    mut riddle_nodes: Query<(&mut Style, &mut Visibility), With<RiddleNode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        let mut door = doors
            .iter_mut()
            .filter(|door| door.active)
            .next()
            .expect("Exactly one active door is expected while this system is running!");
        door.active = false;
        let (mut riddle_style, mut riddle_visibility) = riddle_nodes
            .iter_mut()
            .filter(|(_, visibility)| visibility.is_visible)
            .next()
            .expect("Exactly one visible riddle node is expected while this system is running!");
        riddle_style.display = Display::None;
        riddle_visibility.is_visible = false;
        state.set(GameState::MapExploring).unwrap();
    }
}
