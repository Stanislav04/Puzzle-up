use super::ColliderBundle;
use bevy::{prelude::*, text::Text2dBounds, utils::HashMap};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Default, Component)]
pub struct StaticText;

#[derive(LdtkEntity, Bundle)]
pub struct TextSignBundle {
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

pub fn normalize_font_system(
    asset_server: Res<AssetServer>,
    mut text_query: Query<(&mut Text, &mut Transform), With<StaticText>>,
) {
    for (mut text, mut transform) in text_query.iter_mut() {
        text.sections[0].style.font =
            asset_server.load("fonts/MontserratAlternates-MediumItalic.ttf");
        transform.scale = Vec3::new(1.0, 1.0, 1.0);
    }
}

fn zone_text_node() -> NodeBundle {
    NodeBundle {
        style: Style {
            display: Display::None,
            align_items: AlignItems::Center,
            size: Size::new(Val::Percent(100.0), Val::Percent(20.0)),
            padding: UiRect::all(Val::Percent(5.0)),
            position: UiRect {
                bottom: Val::Percent(0.0),
                ..Default::default()
            },
            ..Default::default()
        },
        color: UiColor::from(Color::rgb(0.5, 0.5, 0.85)),
        visibility: Visibility { is_visible: false },
        ..Default::default()
    }
}

fn zone_text_content(asset_server: &Res<AssetServer>) -> TextBundle {
    TextBundle {
        text: Text::from_section(
            "",
            TextStyle {
                font: asset_server.load("fonts/MontserratAlternates-MediumItalic.ttf"),
                font_size: 40.0,
                color: Color::WHITE,
            },
        ),
        ..Default::default()
    }
}

pub fn zone_text_setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(zone_text_node())
        .insert(CommonTextContainer)
        .with_children(|parent| {
            parent
                .spawn_bundle(zone_text_content(&asset_server))
                .insert(CommonTextContent);
        });
}

#[derive(Default, Component)]
pub struct ZoneText;

#[derive(Default, Component)]
pub struct CommonTextContainer;

#[derive(Default, Component)]
pub struct CommonTextContent;

#[derive(Default, Bundle, LdtkEntity)]
pub struct ZoneTextBundle {
    #[from_entity_instance]
    #[bundle]
    collider_bundle: ColliderBundle,
    sensor: Sensor,
    #[from_entity_instance]
    text_info: TextInfo,
    zone_text: ZoneText,
}

#[derive(Default, Component)]
pub struct TextInfo {
    text: String,
}

impl From<EntityInstance> for TextInfo {
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
            text: fields
                .get("text")
                .expect("Text to be displayed is expected!")
                .clone(),
        }
    }
}

pub fn show_zone_text_system(
    mut events: EventReader<CollisionEvent>,
    mut text_container_info: Query<(&mut Style, &mut Visibility), With<CommonTextContainer>>,
    mut text_content_info: Query<&mut Text, With<CommonTextContent>>,
    zone_texts: Query<&TextInfo, With<ZoneText>>,
) {
    for event in events.iter() {
        if let CollisionEvent::Started(entity, other, _) = event {
            if !(zone_texts.contains(*entity) || zone_texts.contains(*other)) {
                continue;
            }

            let (mut container_style, mut container_visibility) = text_container_info.single_mut();
            let mut text_content = text_content_info.single_mut();
            let zone_text = zone_texts.get(*entity).unwrap_or_else(|_| {
                zone_texts
                    .get(*other)
                    .expect("One of the colliders is expected to be a zone text!")
            });
            text_content.sections[0].value = zone_text.text.clone();
            container_style.display = Display::Flex;
            container_visibility.is_visible = true;
        }
    }
}

pub fn hide_zone_text_system(
    mut events: EventReader<CollisionEvent>,
    mut text_container_info: Query<(&mut Style, &mut Visibility), With<CommonTextContainer>>,
    mut text_content_info: Query<&mut Text, With<CommonTextContent>>,
    zone_texts: Query<&TextInfo, With<ZoneText>>,
) {
    for event in events.iter() {
        if let CollisionEvent::Stopped(entity, other, _) = event {
            if !(zone_texts.contains(*entity) || zone_texts.contains(*other)) {
                continue;
            }

            let (mut container_style, mut container_visibility) = text_container_info.single_mut();
            let mut text_content = text_content_info.single_mut();
            container_style.display = Display::None;
            container_visibility.is_visible = false;
            text_content.sections[0].value = "".to_string();
        }
    }
}
