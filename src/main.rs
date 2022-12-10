use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

const TILE_SIZE: f32 = 16.0;
const MAP_WIDTH: f32 = 608.0;
const MAP_HEIGHT: f32 = 272.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, -50.0),
            ..Default::default()
        })
        .add_plugin(LdtkPlugin)
        .insert_resource(LevelSelection::Index(0))
        .add_startup_system(setup_system)
        .add_system(player_movement_system)
        .register_ldtk_entity::<GroundTile>("Ground")
        .register_ldtk_entity::<GroundTile>("LevelBorder")
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
                translation: Vec3::new(0.0, 0.0, 10.0),
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

    commands.spawn_bundle(LdtkWorldBundle {
        ldtk_handle: asset_server.load("map.ldtk"),
        transform: Transform::from_xyz(-MAP_WIDTH / 2.0, -MAP_HEIGHT / 2.0, 0.0),
        ..default()
    });
}

fn player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    rapier_context: Res<RapierContext>,
    mut player_info: Query<(Entity, &mut Velocity), With<Player>>,
    tile_info: Query<Entity, With<Ground>>,
) {
    for (player, mut velocity) in &mut player_info {
        let up = keyboard_input.any_pressed([KeyCode::Up, KeyCode::W]);
        let left = keyboard_input.any_pressed([KeyCode::Left, KeyCode::A]);
        let right = keyboard_input.any_pressed([KeyCode::Right, KeyCode::D]);

        velocity.linvel.x += -(left as i8 as f32) + right as i8 as f32;

        if up {
            for tile in tile_info.iter() {
                if let Some(contact_pair) = rapier_context.contact_pair(player, tile) {
                    for manifold in contact_pair.manifolds() {
                        if manifold.normal().y == -1.0 {
                            velocity.linvel.y += (up as i8 as f32) * 100.0;
                            break;
                        }
                    }
                }
            }
        }
    }
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
            _ => Self::default(),
        }
    }
}
