@group(0) @binding(0) var world_input: texture_2d<f32>;
@group(0) @binding(1) var world_output: texture_storage_2d<rgba8unorm, write>;

const DECAY_RATE: f32 = 0.8;

fn cleanup(color: vec4<f32>) -> vec4<f32> {
  if color.r > 0.9 && color.b > 0.9 {
    return color;
  } else {
    return vec4<f32>(0.0, 1.0, 0.0, 1.0);
  }
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
  let coords = vec2<i32>(global_id.xy);
  let dims = vec2<f32>(textureDimensions(world_input));
  var blurred = vec4<f32>(0.0, 0.0, 0.0, 0.0);
  var count = 0.0;

  for (var y: i32 = -1; y <= 1; y = y + 1) {
    for (var x: i32 = -1; x <= 1; x = x + 1) {
      let sample_coords = coords + vec2<i32>(x, y);

      let wrapped_x = (sample_coords.x + i32(dims.x)) % i32(dims.x);
      let wrapped_y = (sample_coords.y + i32(dims.y)) % i32(dims.y);

      let color = textureLoad(world_input, vec2<i32>(wrapped_x, wrapped_y), 0);
      if color.g > 0.0 {
        continue;
      }
      blurred += color;
      count += 1.;
    }
  }

  blurred /= count;

  let current = textureLoad(world_input, coords, 0);
  // Uncomment to disable blur
  blurred = current;
  let no_decay = current.g > 0.0;

  let decay = select(DECAY_RATE, 1.0, no_decay);
  var result = vec4(blurred.r * decay, current.g, blurred.b * decay, 1.0);
  let final_c = select(result, cleanup(current), no_decay);

  textureStore(world_output, coords, final_c);
}
