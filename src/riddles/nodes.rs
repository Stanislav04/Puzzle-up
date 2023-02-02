use bevy::prelude::*;

pub fn root_node() -> NodeBundle {
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

pub fn question_text(asset_server: &Res<AssetServer>, question: &String) -> TextBundle {
    TextBundle {
        text: Text::from_section(
            question,
            TextStyle {
                font: asset_server.load("fonts/MontserratAlternates-MediumItalic.ttf"),
                font_size: 60.0,
                color: Color::WHITE,
            },
        )
        .with_alignment(TextAlignment::CENTER),
        ..Default::default()
    }
}

pub fn answer_container() -> NodeBundle {
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

pub fn answer_position(asset_server: &Res<AssetServer>, color: Color) -> TextBundle {
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
