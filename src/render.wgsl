@group(0) @binding(0) var s_pheromone: sampler;
@group(0) @binding(1) var t_pheromone: texture_2d<f32>;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
  let x = f32(in_vertex_index % 2u) * 4.0 - 1.0;
  let y = f32(in_vertex_index / 2u) * -4.0 + 1.0;
  return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
  let dims = vec2<f32>(textureDimensions(t_pheromone));
  let uv = frag_coord.xy / dims;

  return textureSample(t_pheromone, s_pheromone, uv);
}
