struct Ant {
  pos: vec2<f32>,
  angle: f32,
  state: u32,
  frame_hit: u32
};

struct FrameUniform {
  home: vec2<f32>,
  frame_count: u32,
};

@group(0) @binding(0) var<storage, read_write> ants: array<Ant>;
@group(0) @binding(1) var world_output: texture_storage_2d<rgba8unorm, write>; 
@group(0) @binding(2) var world_input: texture_2d<f32>;
@group(0) @binding(3) var<uniform> uniforms: FrameUniform;

const PI: f32 = 3.14159265359;
const AWAY: u32 = 0;
const HOME: u32 = 1;
const TURN_SPEED: f32 = 0.3;
const PHEROMONE_STRENGTH: f32 = 0.3; 
const ANGLE_INFLUENCE: f32 = PI / 4.;
const THETA: f32 = PI / 2.;

fn sense_pheromone(pos: vec2<f32>, current_angle: f32, angle_offset: f32, state: u32) -> f32 {
  let dims = vec2<f32>(textureDimensions(world_input));
  let angle = current_angle + angle_offset;
  let dir = vec2<f32>(cos(angle), sin(angle));
  let sense_pos = pos + dir * 10.0;

  var total_pheromone: f32 = 0.0;
  for (var y: i32 = -1; y <= 1; y = y + 1) {
    for (var x: i32 = -1; x <= 1; x = x + 1) {
      let sample_x = i32(floor(sense_pos.x + f32(x)));
      let sample_y = i32(floor(sense_pos.y + f32(y)));

      let wrapped_x = (sample_x + i32(dims.x)) % i32(dims.x);
      let wrapped_y = (sample_y + i32(dims.y)) % i32(dims.y);

      let sample = textureLoad(world_input, vec2<i32>(wrapped_x, wrapped_y), 0);

      if state == AWAY {
        total_pheromone += sample.b;
      } else {
        total_pheromone += sample.r;
      }
    }
  }

  return total_pheromone;
}


fn get_wrapped_coords(pos: vec2<f32>) -> vec2<i32> {
  let dims = vec2<f32>(textureDimensions(world_input));
  let x = i32(floor(pos.x));
  let y = i32(floor(pos.y));

  let wrapped_x = (x + i32(dims.x)) % i32(dims.x);
  let wrapped_y = (y + i32(dims.y)) % i32(dims.y);

  return vec2<i32>(wrapped_x, wrapped_y);
}

fn check_for_food(color: vec4<f32>) -> bool {
  return color.g > 0.5 && color.r == 0. && color.b == 0.;
}


fn check_for_wall(pos: vec2<f32>) -> bool {
  let coords = get_wrapped_coords(pos);
  let sample = textureLoad(world_input, coords, 0);
  return sample.r >= 1. && sample.g >= 1. && sample.b >= 1.;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
  let index = global_id.x;
  var ant = ants[index];
  let dims = vec2<f32>(textureDimensions(world_input));
  let coords = get_wrapped_coords(ant.pos);
  let current = textureLoad(world_input, coords, 0);

  let random = fract(sin(dot(ant.pos, vec2<f32>(12.9898, 78.233)) + f32(uniforms.frame_count)) * 43758.5453);
  let theta = THETA * random + PI;

  let has_found_food = ant.state == AWAY && check_for_food(current);
  if has_found_food {
    ant.state = HOME;
    ant.angle += PI;
  }

  let dist_to_home = distance(ant.pos, uniforms.home);
  if dist_to_home < 100.0 && ant.state == HOME {
    ant.state = AWAY;
    ant.angle += PI;
  }

  let left = sense_pheromone(ant.pos, ant.angle, -ANGLE_INFLUENCE, ant.state);
  let forward = sense_pheromone(ant.pos, ant.angle, 0.0, ant.state);
  let right = sense_pheromone(ant.pos, ant.angle, ANGLE_INFLUENCE, ant.state);

  ant.angle += (fract(sin(dot(ant.pos, vec2<f32>(12.9898, 78.233))) * 43758.5453) - 0.5) * 0.3;
  if forward > left && forward > right {
  } else if right > left {
    ant.angle += TURN_SPEED;
  } else if left > right {
    ant.angle -= TURN_SPEED;
  }

  let dir = vec2<f32>(cos(ant.angle), sin(ant.angle));
  let update_pos = ant.pos + dir;

  if check_for_wall(update_pos) {
    ant.angle += theta;
    ant.frame_hit = uniforms.frame_count;
  } else {
    ant.pos = update_pos;
  }


  if ant.pos.x < 0.0 { ant.pos.x += dims.x; }
  if ant.pos.y < 0.0 { ant.pos.y += dims.y; }
  if ant.pos.x >= dims.x { ant.pos.x -= dims.x; }
  if ant.pos.y >= dims.y { ant.pos.y -= dims.y; }


  var pheromone = vec4<f32>(current.r, 0.0, current.b, 1.0);
  if ant.state == AWAY {
    pheromone.r += PHEROMONE_STRENGTH;
  } else {
    pheromone.b += PHEROMONE_STRENGTH;
  }

  let result = vec4(
    pheromone.r,
    select(current.g, 0.0, has_found_food),
    pheromone.b,
    1.0
  );

  textureStore(world_output, coords, result);

  ants[index] = ant;
}
