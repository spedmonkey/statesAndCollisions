//! This example illustrates how to use [`States`] for high-level app control flow.
//! States are a powerful but intuitive tool for controlling which logic runs when.
//! You can have multiple independent states, and the [`OnEnter`] and [`OnExit`] schedules
//! can be used to great effect to ensure that you handle setup and teardown appropriately.
//!
//! In this case, we're transitioning from a `Menu` state to an `InGame` state.

// This lint usually gives bad advice in the context of Bevy -- hiding complex queries behind
// type aliases tends to obfuscate code while offering no improvement in code cleanliness.
#![allow(clippy::type_complexity)]
use bevy::{prelude::*, tasks::ParallelSlice};
use bevy_asset_loader::prelude::*;
use bevy_rapier3d::prelude::*;
#[derive(AssetCollection, Resource)]
pub struct Models {
    #[asset(path = "models/floor/floor.gltf#Mesh0/Primitive0")]
    pub floor: Handle<Mesh>,
}
#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}

fn main() {
    App::new()
        .add_state::<MyStates>()
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
        ))
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<Models>(),
        )
        .add_systems(OnEnter(MyStates::Next), expectations)
        .run();
}

fn expectations(
    mut commands: Commands,
    assets: Res<Models>,
    asset_server: Res<AssetServer>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,

    images: Res<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut quit: EventWriter<bevy::app::AppExit>,
) {
    let floor = meshes
        .get(&assets.floor)
        .expect("Image should be added to its asset resource");

    let a = floor;
    let verts = floor
        .get_index_buffer_bytes()
        .unwrap()
        .chunks_exact(3)
        .map(|chunk| Vec3 {
            x: chunk[0] as f32,
            y: chunk[1] as f32,
            z: chunk[2] as f32,
        })
        .collect::<Vec<Vec3>>();

    let index: Vec<_> = floor
        .get_index_buffer_bytes()
        .unwrap()
        .chunks_exact(3)
        .map(|chunk| [chunk[0] as u32, chunk[1] as u32, chunk[2] as u32])
        .collect();

    let x_shape = Collider::from_bevy_mesh(a, &ComputedColliderShape::TriMesh).unwrap();

    commands
        .spawn(
            (PbrBundle {
                mesh: assets.floor.clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                material: standard_materials.add(Color::SILVER.into()),
                ..default()
            }),
        )
        .insert(
            // If you use a different collider that isn't a bevy mesh here it no longer panics
            x_shape,
        )
        .insert(RigidBody::Fixed);

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 3.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
    });

    //spawn box:
    commands
        .spawn(
            (PbrBundle {
                mesh: meshes.add(shape::Cube::new(2.0).into()),
                transform: Transform::from_xyz(0.0, 5.0, 0.0),
                material: standard_materials.add(Color::SILVER.into()),
                ..default()
            }),
        )
        .insert(RigidBody::Dynamic)
        .insert(Damping {
            linear_damping: 0.5,
            angular_damping: 1.0,
        })
        .insert(GravityScale(5.5))
        .insert(Collider::cuboid(1.0, 1.0, 1.0));

    //commands
    //    .spawn(RigidBody::KinematicPositionBased)
    //    .insert(Collider::ball(0.5))
    //    .insert(KinematicCharacterController::default());

    /* Apply forces when the rigid-body is created. */
    commands
        .spawn(RigidBody::Dynamic)
        .insert(ExternalForce {
            force: Vec3::new(10.0, 20.0, 30.0),
            torque: Vec3::new(1.0, 2.0, 3.0),
        })
        .insert(ExternalImpulse {
            impulse: Vec3::new(1.0, 2.0, 3.0),
            torque_impulse: Vec3::new(0.1, 0.2, 0.3),
        });
}
