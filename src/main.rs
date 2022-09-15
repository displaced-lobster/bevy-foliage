use bevy::{
    asset::AssetServerSettings,
    gltf::{Gltf, GltfMesh, GltfNode},
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::*,
};
use bevy_atmosphere::prelude::*;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};
use bevy_foliage::{PanOrbitCameraBundle, PanOrbitCameraPlugin};

fn main() {
    App::new()
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..default()
        })
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .add_plugin(AtmospherePlugin)
        .add_plugin(PanOrbitCameraPlugin)
        .add_plugin(MaterialPlugin::<FoliageMaterial>::default())
        .add_plugin(InspectorPlugin::<Data>::new())
        // .init_resource::<Data>()
        .add_state(AppState::Loading)
        .add_startup_system(setup)
        .add_system_set(SystemSet::on_update(AppState::Loading).with_system(create_tree))
        .add_system_set(SystemSet::on_update(AppState::Ready).with_system(update_foliage))
        .run();
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Loading,
    Ready,
}

#[derive(Debug, Inspectable)]
struct Data {
    bark_material: Handle<StandardMaterial>,
    foliage_material: Handle<FoliageMaterial>,
    grass_material: Handle<StandardMaterial>,
}

impl FromWorld for Data {
    fn from_world(world: &mut World) -> Self {
        let server = world.get_resource::<AssetServer>().unwrap();
        let base_color_texture = server.load("textures/greens.png");
        let alpha_mask = server.load("textures/dots.png");
        let mut materials = world.get_resource_mut::<Assets<FoliageMaterial>>().unwrap();
        let foliage_material = materials.add(FoliageMaterial {
            base_color: Color::rgb(0.37, 0.58, 0.1),
            base_color_texture: Some(base_color_texture),
            alpha_mask: Some(alpha_mask),
            ..default()
        });

        let mut materials = world.get_resource_mut::<Assets<StandardMaterial>>().unwrap();
        let bark_material = materials.add(StandardMaterial {
            base_color: Color::rgb(0.44, 0.27, 0.11),
            perceptual_roughness: 1.0,
            reflectance: 0.0,
            ..default()
        });
        let grass_material = materials.add(StandardMaterial {
            base_color: Color::rgb(0.37, 0.58, 0.1),
            ..default()
        });

        Self { bark_material, foliage_material, grass_material }
    }
}

#[derive(AsBindGroup, Debug, Clone, Inspectable, Reflect, TypeUuid)]
#[uuid = "4ee9c363-1124-4113-890e-199d81b00281"]
struct FoliageMaterial {
    #[uniform(0)]
    base_color: Color,
    #[inspectable(min = 0.0, max = 1.0)]
    #[uniform(0)]
    perceptual_roughness: f32,
    #[inspectable(min = 0.0, max = 1.0)]
    #[uniform(0)]
    effect_blend: f32,
    #[inspectable(min = 0.0)]
    #[uniform(0)]
    billboard_size: f32,
    #[inspectable(min = 0.0)]
    #[uniform(0)]
    inflate: f32,
    #[inspectable(min = 0.0)]
    #[uniform(0)]
    wind_strength: f32,
    #[uniform(0)]
    time: f32,
    #[texture(1)]
    #[sampler(2)]
    base_color_texture: Option<Handle<Image>>,
    #[texture(3)]
    #[sampler(4)]
    alpha_mask: Option<Handle<Image>>,
}

impl Default for FoliageMaterial {
    fn default() -> Self {
        Self {
            base_color: Color::GREEN,
            perceptual_roughness: 0.5,
            effect_blend: 1.0,
            billboard_size: 1.0,
            inflate: 0.0,
            wind_strength: 0.1,
            time: 0.0,
            base_color_texture: None,
            alpha_mask: None,
        }
    }
}

impl From<Color> for FoliageMaterial {
    fn from(base_color: Color) -> Self {
        Self {
            base_color,
            ..default()
        }
    }
}

impl Material for FoliageMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/foliage.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/foliage.wgsl".into()
    }
}

struct GLTFAsset(Handle<Gltf>);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    data: Res<Data>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let gltf = asset_server.load("models/tree.glb");

    commands.insert_resource(GLTFAsset(gltf));

    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(10.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands
        .spawn_bundle(PanOrbitCameraBundle::new(Vec3::new(0.0, 2.0, 5.0), Vec3::ZERO))
        .insert(AtmosphereCamera(None));

    commands.spawn_bundle(PbrBundle {
        material: data.grass_material.clone(),
        mesh: meshes.add(Mesh::from(shape::Plane { size: 1000.0 })),
        ..default()
    });
}

fn create_tree(
    mut commands: Commands,
    data: Res<Data>,
    mut app_state: ResMut<State<AppState>>,
    gltf: ResMut<GLTFAsset>,
    assets_gltf: Res<Assets<Gltf>>,
    assets_gltfnode: Res<Assets<GltfNode>>,
    assets_gltfmesh: Res<Assets<GltfMesh>>,
) {
    if let Some(gltf) = assets_gltf.get(&gltf.0) {
        if let Some(named_node) = gltf.named_nodes.get("tree-trunk") {
            let node = assets_gltfnode.get(&named_node).unwrap();
            let mesh = assets_gltfmesh
                .get(node.mesh.as_ref().unwrap())
                .unwrap();

            commands.spawn_bundle(PbrBundle {
                material: data.bark_material.clone(),
                mesh: mesh.primitives[0].mesh.clone(),
                transform: node.transform,
                ..default()
            });
        }

        if let Some(named_node) = gltf.named_nodes.get("foliage") {
            let node = assets_gltfnode.get(&named_node).unwrap();
            let mesh = assets_gltfmesh
                .get(node.mesh.as_ref().unwrap())
                .unwrap();

                commands.spawn_bundle(MaterialMeshBundle {
                    material: data.foliage_material.clone(),
                    mesh: mesh.primitives[0].mesh.clone(),
                    transform: node.transform,
                    ..default()
                });
        }

        app_state.set(AppState::Ready).unwrap();
    }
}

fn update_foliage(
    time: Res<Time>,
    mut foliage_materials: ResMut<Assets<FoliageMaterial>>,
    foliage_materials_query: Query<&Handle<FoliageMaterial>>,
) {
    let elapsed_time = time.seconds_since_startup() as f32;

    for material in foliage_materials_query.iter() {
        if let Some(mut material) = foliage_materials.get_mut(material) {
            material.time = elapsed_time;
        }
    }
}
