@group(0) @binding(0) var t_source: texture_2d<f32>;
@group(0) @binding(1) var t_destination: texture_storage_2d<rgba8unorm, write>;

const DECAY_RATE = 0.8;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let tex_coords = vec2<i32>(global_id.xy);
    let dims = vec2<f32>(textureDimensions(t_source));

    var blurred_color = vec4<f32>(0.0);
    for (var y: i32 = -1; y <= 1; y = y + 1) {
        for (var x: i32 = -1; x <= 1; x = x + 1) {
            let offset = vec2<i32>(x, y);
            blurred_color += textureLoad(t_source, tex_coords, 0);
        }
    }
    blurred_color /= 9.0;

    let final_color = blurred_color * DECAY_RATE;

    textureStore(t_destination, tex_coords, final_color);
}
