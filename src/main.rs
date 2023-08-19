use std::collections::HashMap;

use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::window::PrimaryWindow;
use hexx::shapes;
use hexx::*;
use bevy_mod_raycast::{prelude::*, print_intersections};

const HEX_SIZE: Vec2 = Vec2::splat(1.0);
const CAMERA_TRANSFORM: Transform = Transform::from_translation(Vec3::new(0.0, 60.0, 60.0)).looking_at(Vec3::ZERO, Vec3::Y);

#[warn(non_snake_case)]
fn main() {
    App::new().add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            fit_canvas_to_parent: true,
            ..default()
        }),
        ..Default::default()
    }))
    .add_systems(First, update_raycast_with_cursor.before(RaycastSystem::BuildRays::<RaytraceableTile>),)
    .add_systems(Startup, (setup_camera, setup_grid, print_intersections::<RaytraceableTile>))
    .add_systems(Update, handle_input)
    .run();
}

#[derive(Reflect)]
struct RaytraceableTile;

#[derive(Debug, Default, Resource)]
struct HighlightedHexes {
    pub selected: Hex,
}

#[derive(Debug, Resource)]
struct Map {
    layout: HexLayout,
    entities: HashMap<Hex, Entity>,
    selected_material: Handle<StandardMaterial>,
    default_material: Handle<StandardMaterial>,
}

/// camera setup
fn setup_camera(mut commands: Commands) {
    
    commands.spawn(Camera3dBundle {
        transform: CAMERA_TRANSFORM,
        ..default()
    });
    commands.spawn(DirectionalLightBundle {
        transform: CAMERA_TRANSFORM,
        ..default()
    });
}

/// Hex grid setup
fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let layout = HexLayout {
        hex_size: HEX_SIZE,
        ..default()
    };
    // materials
    let selected_material = materials.add(Color::RED.into());
    let default_material = materials.add(Color::WHITE.into());
    // mesh
    let mesh = create_hexagon_mesh(&layout);
    let mesh_handle = meshes.add(mesh);

    let entities = shapes::flat_rectangle([-5, 5, -4, 4])
        .map(|hex| {
            let pos = layout.hex_to_world_pos(hex);
            let id = commands
                .spawn( PbrBundle{
                    transform: Transform::from_xyz(pos.x, hex.length() as f32 / 2.0, pos.y).with_scale(Vec3::splat(0.9)),
                    mesh: mesh_handle.clone(),
                    material: default_material.clone(),
                    ..default()
                })
                .insert(RaycastMesh::<RaytraceableTile>::default())
                .id();
            (hex, id)
        })
        .collect();
    commands.insert_resource(Map {
        layout,
        entities,
        selected_material,
        default_material,
    });
}

/// Input interaction
fn update_raycast_with_cursor(mut cursor: EventReader<CursorMoved>, mut query: Query<&mut RaycastSource<RaytraceableTile>>) {
     // Grab the most recent cursor event if it exists:
     let Some(cursor_moved) = cursor.iter().last() else { return };
     for mut pick_source in &mut query {
         pick_source.cast_method = RaycastMethod::Screenspace(cursor_moved.position);
     }
}

fn handle_input(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    map: Res<Map>,
    mut highlighted_hexes: Local<HighlightedHexes>,
) {
    let window = windows.single();
    let (camera, cam_transform) = cameras.single();
    if let Some(pos) = window
        .cursor_position()
        .and_then(|p| Ray3d::from_screenspace(p, camera, cam_transform))
    {
        let coord = map.layout.world_pos_to_hex(pos);
        
        if let Some(entity) = map.entities.get(&coord).copied() {
            if coord == highlighted_hexes.selected {
                return;
            }
            // Clear highlighted hexes materials
            for entity in [&highlighted_hexes.selected].iter().filter_map(|h| map.entities.get(h)) {
                commands
                    .entity(*entity)
                    .insert(map.default_material.clone());
            }
            
            commands
                .entity(entity)
                .insert(map.selected_material.clone());
            highlighted_hexes.selected = coord;
        }
    }
}

/// Compute a bevy mesh from the layout
fn create_hexagon_mesh(hex_layout: &HexLayout) -> Mesh {
    let mesh_info = ColumnMeshBuilder::new(hex_layout, 10.0).without_bottom_face().build();
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_info.vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_info.normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_info.uvs);
    mesh.set_indices(Some(Indices::U16(mesh_info.indices)));
    mesh
}