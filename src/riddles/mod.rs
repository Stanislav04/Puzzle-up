use crate::{map::CurrentLevel, player::Player, GameState};
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

mod nodes;

pub struct RiddlesPlugin;

impl Plugin for RiddlesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AnsweredRiddles::new())
            .add_system_set(
                SystemSet::on_exit(GameState::LevelLoading).with_system(init_riddles_system),
            )
            .add_system_set(
                SystemSet::on_update(GameState::MapExploring).with_system(touch_door_system),
            )
            .add_system_set(
                SystemSet::on_update(GameState::RiddleSolving)
                    .with_system(answering_riddle_system)
                    .with_system(delete_digit_system)
                    .with_system(correct_answer_system)
                    .with_system(close_riddle_system),
            );
    }
}

type AnsweredRiddles = HashSet<String>;

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

#[derive(Default, Component)]
pub struct RiddleInfo {
    question: String,
    answer: String,
    riddle: Option<Entity>,
    active: bool,
    next_level: String,
}

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
            next_level: fields
                .get("next_level")
                .expect("A next level is required for a riddle!")
                .clone(),
            ..Default::default()
        }
    }
}

fn init_riddles_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    answered_riddles: Res<AnsweredRiddles>,
    mut doors: Query<(&mut RiddleInfo, &mut TextureAtlasSprite)>,
) {
    use nodes::*;

    for (mut door, mut sprite) in doors.iter_mut() {
        if answered_riddles.contains(&door.question) {
            sprite.index = 75;
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
    rapier_context: Res<RapierContext>,
    answered_riddles: Res<AnsweredRiddles>,
    mut keyboard_input: ResMut<Input<KeyCode>>,
    mut state: ResMut<State<GameState>>,
    mut current_level: ResMut<CurrentLevel>,
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
                    return;
                }
                keyboard_input.reset(KeyCode::Space);
                current_level.clone_from(&riddle_info.next_level);
                state.set(GameState::LevelLoading).unwrap();
                return;
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
            .find(|(_, visibility)| visibility.is_visible())
            .expect("A visible container is expected while this system is running!");
        let (mut answer, _, _) = answer_nodes
            .iter_mut()
            .find(|(_, visibility, answer)| visibility.is_visible() && answer.position == container.index)
            .expect("The container is expected to have answer positions and the container's index is always valid!");
        answer.sections[0].value = character.char.to_string();
        container.index = (container.index + 1) % container.answer_length;
    }
}

fn delete_digit_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut container_info: Query<(&mut AnswerContainer, &ComputedVisibility)>,
    mut answer_nodes: Query<(&mut Text, &ComputedVisibility, &Answer)>,
) {
    if !keyboard_input.just_pressed(KeyCode::Back) {
        return;
    }
    let (mut container, _) = container_info
        .iter_mut()
        .find(|(_, visibility)| visibility.is_visible())
        .expect("A visible container is expected while this system is running!");
    if container.index == 0 {
        container.index = container.answer_length;
    }
    container.index -= 1;
    let (mut answer, _, _) = answer_nodes
            .iter_mut()
            .find(|(_, visibility, answer)| visibility.is_visible() && answer.position == container.index)
            .expect("The container is expected to have answer positions and the container's index is always valid!");
    answer.sections[0].value = "_".to_string();
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
            .find(|(door, _)| door.active)
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
            .find(|door| door.active)
            .expect("Exactly one active door is expected while this system is running!");
        door.active = false;
        let (mut riddle_style, mut riddle_visibility) = riddle_nodes
            .iter_mut()
            .find(|(_, visibility)| visibility.is_visible)
            .expect("Exactly one visible riddle node is expected while this system is running!");
        riddle_style.display = Display::None;
        riddle_visibility.is_visible = false;
        state.set(GameState::MapExploring).unwrap();
    }
}
