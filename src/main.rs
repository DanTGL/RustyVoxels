use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::render::RenderPlugin;
use bevy::render::mesh::{VertexAttributeValues, Indices};
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::settings::{WgpuSettings, WgpuFeatures};
use bevy::{prelude::*, transform};
use block_mesh::ilattice::prelude::Vec3A;
use block_mesh::ndshape::{ConstShape, ConstShape3u32};
use block_mesh::{Voxel, VoxelVisibility, MergeVoxel, RIGHT_HANDED_Y_UP_CONFIG, GreedyQuadsBuffer, greedy_quads};
use noise::{Fbm, Perlin, NoiseFn};
use noise::utils::{PlaneMapBuilder, NoiseMapBuilder};

use crate::camera::MyCameraPlugin;

mod camera;

const ChunkSize: u32 = 16;
const PaddedChunkSize: u32 = ChunkSize + 2;

#[derive(Clone, Copy, Eq, PartialEq)]
struct BoolVoxel(bool);

const EMPTY: BoolVoxel = BoolVoxel(false);
const FULL: BoolVoxel = BoolVoxel(true);


impl Voxel for BoolVoxel {
    fn get_visibility(&self) -> VoxelVisibility {
        if *self == EMPTY {
            VoxelVisibility::Empty
        } else {
            VoxelVisibility::Opaque
        }
    }
}

impl MergeVoxel for BoolVoxel {
    type MergeValue = Self;

    fn merge_value(&self) -> Self::MergeValue {
        *self
    }
}

fn into_domain(array_dim: u32, [x, y, z]: [u32; 3]) -> Vec3A {
    (2.0 / array_dim as f32) * Vec3A::new(x as f32, y as f32, z as f32) - 1.0
}

fn sphere(radius: f32, p: Vec3A) -> BoolVoxel {
    //BoolVoxel(p.length() < radius)
    //BoolVoxel(p.x == p.y);
    
    let fbm = Fbm::<Perlin>::new(2);

    BoolVoxel(fbm.get([p.x as f64, p.z as f64]) * 2.0 > p.y as f64)

    //BoolVoxel(p.y <= (PlaneMapBuilder::<_, 2>::new(&fbm)
    //    .build().get_value(p.x as usize, p.z as usize)) as f32)
}

type ChunkShape = ConstShape3u32<PaddedChunkSize, PaddedChunkSize, PaddedChunkSize>;

fn spawn_pbr(
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    mesh: Handle<Mesh>,
    transform: Transform,
) {
    let mut material = StandardMaterial::from(Color::rgb(0.0, 0.0, 0.0));
    material.perceptual_roughness = 0.9;

    commands.spawn(PbrBundle {
        mesh,
        material: materials.add(material),
        transform,
        ..Default::default()
    });
}

fn generate_greedy_mesh(
    meshes: &mut Assets<Mesh>,
    sdf: impl Fn(Vec3A) -> BoolVoxel,
) -> Handle<Mesh> {
    let mut samples = [EMPTY; ChunkShape::SIZE as usize];

    for i in 0..(ChunkShape::SIZE) {
        let p = into_domain(ChunkSize, ChunkShape::delinearize(i));
        samples[i as usize] = sdf(p);
    }

    let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;

    let mut buffer = GreedyQuadsBuffer::new(samples.len());

    greedy_quads(
        &samples, 
        &ChunkShape {}, 
        [0; 3], 
        [PaddedChunkSize-1; 3], 
        &faces, 
        &mut buffer,
    );

    let num_indices = buffer.quads.num_quads() * 6;
    let num_vertices = buffer.quads.num_quads() * 4;
    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);

    for (group, face) in buffer.quads.groups.into_iter().zip(faces.into_iter()) {
        for quad in group.into_iter() {
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            positions.extend_from_slice(&face.quad_mesh_positions(&quad, 1.0));
            normals.extend_from_slice(&face.quad_mesh_normals());
        }
    }

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, VertexAttributeValues::Float32x3(positions));
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, VertexAttributeValues::Float32x3(normals));
    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        VertexAttributeValues::Float32x2(vec![[0.0; 2]; num_vertices]),
    );

    render_mesh.set_indices(Some(Indices::U32(indices.clone())));


    meshes.add(render_mesh)
}

fn setup(
    mut commands: Commands,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    wireframe_config.global = true;

    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(25.0, 25.0, 25.0)),
        point_light: PointLight {
            range: 200.0,
            intensity: 8000.0,
            ..Default::default()
        },
        ..Default::default()
    });

    /*commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(Vec3::new(50.0, 15.0, 50.0))
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..Default::default()
    });*/

    let greedy_sphere_mesh = generate_greedy_mesh(&mut meshes, |p| sphere(0.9, p));

    spawn_pbr(
        &mut commands,
        &mut materials,
        greedy_sphere_mesh,
        Transform::from_translation(Vec3::new(16.0, -16.0, 8.0)),
    );
}

fn main() {
    println!("Hello, world!");



    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            wgpu_settings: WgpuSettings {
features: WgpuFeatures::POLYGON_MODE_LINE,
..Default::default()
            },
            ..default()
        }))
        .add_plugin(WireframePlugin)
        .add_plugin(MyCameraPlugin)
        .add_startup_system(setup)
        .run();
}
