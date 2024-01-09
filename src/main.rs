//! This example illustrates how to use [`States`] for high-level app control flow.
//! States are a powerful but intuitive tool for controlling which logic runs when.
//! You can have multiple independent states, and the [`OnEnter`] and [`OnExit`] schedules
//! can be used to great effect to ensure that you handle setup and teardown appropriately.
//!
//! In this case, we're transitioning from a `Menu` state to an `InGame` state.

// This lint usually gives bad advice in the context of Bevy -- hiding complex queries behind
// type aliases tends to obfuscate code while offering no improvement in code cleanliness.
#![allow(clippy::type_complexity)]
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
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
    InGame,
}
#[derive(Component)]
struct FpsText;

fn main() {
    App::new()
        .add_state::<MyStates>()
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
            FrameTimeDiagnosticsPlugin,
        ))
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<Models>(),
        )
        .add_systems(Startup, infotext_system)
        .add_systems(OnEnter(MyStates::Next), expectations)
        .add_systems(Update, movement.run_if(in_state(MyStates::Next)))
        .add_systems(Update, change_text_system.run_if(in_state(MyStates::Next)))
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

    let x_shape = Collider::from_bevy_mesh(floor, &ComputedColliderShape::TriMesh).unwrap();

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
        .insert(GravityScale(0.50))
        .insert(Collider::ball(1.0));

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

    //character
    //spawn box:
    commands
        .spawn(
            (PbrBundle {
                mesh: meshes.add(shape::Cube::new(2.0).into()),
                transform: Transform::from_xyz(1.5, 2.0, 1.0),
                material: standard_materials.add(Color::SILVER.into()),
                ..default()
            }),
        )
        .insert(Collider::cuboid(0.9, 0.9, 0.9))
        .insert(KinematicCharacterController {
            // The character offset is set to 0.01.
            offset: CharacterLength::Absolute(0.1),
            ..default()
        })
        .insert(ColliderMassProperties::Density(199.0));
}

fn movement(
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut KinematicCharacterController>,
) {
    let mut player = query.single_mut();

    let mut translation = Vec3::new(0.0, 0.0, 0.0);

    if input.pressed(KeyCode::Right) {
        translation.x += time.delta_seconds() * 5.0;
    }

    if input.pressed(KeyCode::Left) {
        translation.x += time.delta_seconds() * 5.0 * -1.0;
    }

    if input.pressed(KeyCode::Down) {
        translation.z += time.delta_seconds() * 5.0;
    }

    if input.pressed(KeyCode::Up) {
        translation.z += time.delta_seconds() * 5.0 * -1.0;
    }

    if input.just_pressed(KeyCode::W) {
        translation.y += time.delta_seconds() * 10.0 * 1.0;
    }
    if input.just_pressed(KeyCode::S) {
        translation.y += time.delta_seconds() * 10.0 * -1.0;
    }
    translation.y = time.delta_seconds() * 10.0 * (translation.y - 10.0);
    player.translation = Some(translation);
}

#[derive(Component)]
struct TextChanges;

fn change_text_system(
    time: Res<Time>,
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<TextChanges>>,
) {
    for mut text in &mut query {
        let mut fps = 0.0;
        if let Some(fps_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(fps_smoothed) = fps_diagnostic.smoothed() {
                fps = fps_smoothed;
            }
        }

        let mut frame_time = time.delta_seconds_f64();
        if let Some(frame_time_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FRAME_TIME)
        {
            if let Some(frame_time_smoothed) = frame_time_diagnostic.smoothed() {
                frame_time = frame_time_smoothed;
            }
        }

        text.sections[0].value = format!(
            "This text changes in the bottom right - {fps:.1} fps, {frame_time:.3} ms/frame",
        );

        //text.sections[2].value = format!("{fps:.1}");
        //text.sections[4].value = format!("{frame_time:.3}");
    }
}

fn infotext_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TextBundle::from_sections([TextSection::new(
            "This text changes in the bottom right",
            TextStyle {
                color: Color::WHITE,
                ..default()
            },
        )])
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(15.0),
            ..default()
        }),
        TextChanges,
    ));
    commands.spawn(
        TextBundle::from_section(
            "This\ntext has\nline breaks and also a set width in the bottom left",
            TextStyle {
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            left: Val::Px(15.0),
            width: Val::Px(200.0),
            ..default()
        }),
    );
}
