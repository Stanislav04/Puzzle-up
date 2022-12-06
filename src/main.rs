use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, -50.0),
            ..Default::default()
        })
        .add_startup_system(setup_system)
        .add_system(player_movement_system)
        .run();
}

#[derive(Component)]
struct Player;

fn setup_system(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("Kenney/PNG/Player/Poses/player_idle.png"),
            transform: Transform {
                scale: Vec3::new(0.5, 0.5, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Player)
        .insert(Velocity::default())
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(30.0, 55.0))
        .insert(LockedAxes::ROTATION_LOCKED);
}

fn player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_info: Query<&mut Velocity, With<Player>>,
) {
    for mut velocity in &mut player_info {
        let up = keyboard_input.any_pressed([KeyCode::Up, KeyCode::W]);
        let down = keyboard_input.any_pressed([KeyCode::Down, KeyCode::S]);
        let left = keyboard_input.any_pressed([KeyCode::Left, KeyCode::A]);
        let right = keyboard_input.any_pressed([KeyCode::Right, KeyCode::D]);

        velocity.linvel.x += -(left as i8 as f32) + right as i8 as f32;
        velocity.linvel.y += -(down as i8 as f32) + up as i8 as f32;
    }
}
