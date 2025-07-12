@group(0) @binding(0) var pheromone_in: texture_storage_2d<rgba8unorm, read>;
@group(0) @binding(1) var pheromone_out: texture_storage_2d<rgba8unorm, write>;

const DECAY_FACTOR = 0.995;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let tex_coords = vec2<i32>(global_id.xy);
    var color = textureLoad(pheromone_in, tex_coords);
    
    color.r *= DECAY_FACTOR;
    color.g *= DECAY_FACTOR;
    color.b *= DECAY_FACTOR;

    textureStore(pheromone_out, tex_coords, color);
}
