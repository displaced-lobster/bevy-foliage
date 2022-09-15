#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

struct FoliageMaterial {
    base_color: vec4<f32>,
    perceptual_roughness: f32,
    effect_blend: f32,
    billboard_size: f32,
    inflate: f32,
    wind_strength: f32,
    time: f32,
}

@group(1) @binding(0)
var<uniform> material: FoliageMaterial;
@group(1) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(2)
var base_color_sampler: sampler;
@group(1) @binding(3)
var alpha_mask_texture: texture_2d<f32>;
@group(1) @binding(4)
var alpha_mask_sampler: sampler;

#import bevy_pbr::mesh_functions
#import bevy_pbr::utils
#import bevy_pbr::lighting
#import bevy_pbr::shadows
#import bevy_pbr::clustered_forward
#import bevy_pbr::pbr_types
#import bevy_pbr::pbr_functions

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

fn rotate(v: vec2<f32>, rotation: f32, mid: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
        cos(rotation) * (v.x - mid.x) + sin(rotation) * (v.y - mid.y) + mid.x,
        cos(rotation) * (v.y - mid.y) - sin(rotation) * (v.x - mid.x) + mid.y,
    );
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var position = vertex.position.xyz;

    // Remap UV from 0 to 1 to -1 to 1
    var offset = vertex.uv * 2.0 - 1.0;

    // Invert direction
    offset *= vec2<f32>(-1.0, 1.0);

    // Add wind effect
    offset = rotate(
        offset,
        sin(material.time) * material.wind_strength,
        vec2<f32>(0.0, -1.0)
    );

    var view_offset = normalize(view.view * vec4<f32>(offset, 0.0, 0.0));
    var world_position = mesh_position_local_to_world(
        mesh.model,
        vec4<f32>(position, 1.0)
    );

    let blend = saturate(material.effect_blend);
    let zero = vec4<f32>(0.0);

    view_offset = mix(zero, view_offset, blend);
    world_position += material.billboard_size * view_offset;

    let inflated_normals = material.inflate * vertex.normal;

    world_position += vec4<f32>(inflated_normals, 0.0);

    var out: VertexOutput;

    out.world_position = world_position;
    out.clip_position = mesh_position_world_to_clip(out.world_position);
    out.world_normal = mesh_normal_local_to_world(vertex.normal);
    out.uv = vertex.uv;

    return out;
}

struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

@fragment
fn fragment(
    in: FragmentInput,
) -> @location(0) vec4<f32> {
    let alpha = textureSample(alpha_mask_texture, alpha_mask_sampler, in.uv).r;
    var base_color = textureSample(base_color_texture, base_color_sampler, in.uv);

    base_color *= material.base_color;

    var pbr_input = pbr_input_new();

    pbr_input.material.base_color = vec4<f32>(base_color.rgb, alpha);
    pbr_input.material.flags = STANDARD_MATERIAL_FLAGS_ALPHA_MODE_MASK;
    pbr_input.material.perceptual_roughness = saturate(material.perceptual_roughness);
    pbr_input.frag_coord = in.clip_position;
    pbr_input.N = prepare_normal(
        pbr_input.material.flags,
        in.world_normal,
        in.uv,
        in.is_front
    );
    pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic);
    pbr_input.world_normal = in.world_normal;
    pbr_input.world_position = in.world_position;

    return tone_mapping(pbr(pbr_input));
}