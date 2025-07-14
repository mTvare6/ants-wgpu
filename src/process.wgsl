@group(0) @binding(0) var world_input: texture_2d<f32>;
@group(0) @binding(1) var world_output: texture_storage_2d<rgba8unorm, write>;

const DECAY_RATE: f32 = 0.9; 

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
  let coords = vec2<i32>(global_id.xy);
  let dims = vec2<f32>(textureDimensions(world_input));
  var blurred = vec4<f32>(0.0, 0.0, 0.0, 0.0);

  for (var y: i32 = -1; y <= 1; y = y + 1) {
    for (var x: i32 = -1; x <= 1; x = x + 1) {
      let sample_coords = coords + vec2<i32>(x, y);

      let wrapped_x = (sample_coords.x + i32(dims.x)) % i32(dims.x);
      let wrapped_y = (sample_coords.y + i32(dims.y)) % i32(dims.y);

      blurred += textureLoad(world_input, vec2<i32>(wrapped_x, wrapped_y), 0);
    }
  }

  blurred /= 9.0;

  let current = textureLoad(world_input, coords, 0);

  let result = vec4(
    blurred.r * DECAY_RATE,
    current.g,
    blurred.b * DECAY_RATE,
    1.0
  );

  textureStore(world_output, coords, result);
}
