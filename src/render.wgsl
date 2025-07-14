@group(0) @binding(0) var world_sampler: sampler;
@group(0) @binding(1) var world_map: texture_2d<f32>;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
  let x = f32(in_vertex_index % 2u) * 4.0 - 1.0;
  let y = f32(in_vertex_index / 2u) * -4.0 + 1.0;
  return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
  let dims = vec2<f32>(textureDimensions(world_map));
  let uv = frag_coord.xy / dims;
  
  let color = textureSample(world_map, world_sampler, uv);
  
  let result = vec4(
    color.r * 1.5,  // Brighten red pheromone
    color.g,        // Keep food as is
    color.b * 1.5,  // Brighten blue pheromone
    1.0
  );
  
  return result;
}
