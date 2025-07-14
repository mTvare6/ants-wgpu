struct Ant {
  pos: vec2<f32>,
  angle: f32,
  state: u32
};

@group(0) @binding(0) var<storage, read_write> ants: array<Ant>;
@group(0) @binding(1) var world_output: texture_storage_2d<rgba8unorm, write>; 
@group(0) @binding(2) var world_input: texture_2d<f32>;

const PI: f32 = 3.14159265359;
const AWAY: u32 = 0;
const HOME: u32 = 1;
const TURN_SPEED: f32 = 0.5;
const PHEROMONE_STRENGTH: f32 = 1.0; 

fn sense_pheromone(pos: vec2<f32>, current_angle: f32, angle_offset: f32, state: u32) -> f32 {
  let dims = vec2<f32>(textureDimensions(world_input));
  let angle = current_angle + angle_offset;
  let dir = vec2<f32>(cos(angle), sin(angle));
  let sense_pos = pos + dir * 10.0;

  var total_pheromone: f32 = 0.0;
  var count: f32 = 0.0;

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
      count += 1.0;
    }
  }

  if count > 0.0 {
    return total_pheromone / count;
  }
  return 0.0;
}

fn check_for_food(pos: vec2<f32>) -> bool {
  let dims = vec2<f32>(textureDimensions(world_input));
  let sample_x = i32(floor(pos.x));
  let sample_y = i32(floor(pos.y));

  let wrapped_x = (sample_x + i32(dims.x)) % i32(dims.x);
  let wrapped_y = (sample_y + i32(dims.y)) % i32(dims.y);

  let sample = textureLoad(world_input, vec2<i32>(wrapped_x, wrapped_y), 0);
  return sample.g > 0.5;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
  let index = global_id.x;
  var ant = ants[index];

  let dims = vec2<f32>(textureDimensions(world_input));

  if check_for_food(ant.pos) && ant.state == AWAY {
    ant.state = HOME;
    ant.angle += PI; 
  }

  let center = dims / 2.0;
  let dist_to_home = distance(ant.pos, center);
  if dist_to_home < 20.0 && ant.state == HOME {
    ant.state = AWAY;
    ant.angle += PI; 
  }

  let left = sense_pheromone(ant.pos, ant.angle, -PI/4.0, ant.state);
  let forward = sense_pheromone(ant.pos, ant.angle, 0.0, ant.state);
  let right = sense_pheromone(ant.pos, ant.angle, PI/4.0, ant.state);

  if forward > left && forward > right {
  } else if right > left {
    ant.angle += TURN_SPEED * 0.2;
  } else if left > right {
    ant.angle -= TURN_SPEED * 0.2;
  }

  ant.angle += (fract(sin(dot(ant.pos, vec2<f32>(12.9898, 78.233))) * 43758.5453) - 0.5) * 0.3;
  let dir = vec2<f32>(cos(ant.angle), sin(ant.angle));
  ant.pos += dir * 2.0;

  if ant.pos.x < 0.0 { ant.pos.x += dims.x; }
  if ant.pos.y < 0.0 { ant.pos.y += dims.y; }
  if ant.pos.x >= dims.x { ant.pos.x -= dims.x; }
  if ant.pos.y >= dims.y { ant.pos.y -= dims.y; }

  let coords = vec2<i32>(i32(floor(ant.pos.x)), i32(floor(ant.pos.y)));
  var current = textureLoad(world_input, coords, 0);

  var pheromone = vec4<f32>(0.0, 0.0, 0.0, 1.0);
  if ant.state == AWAY {
    pheromone.r = PHEROMONE_STRENGTH; 
  } else {
    pheromone.b = PHEROMONE_STRENGTH; 
  }

  let result = vec4(
    max(current.r, pheromone.r), 
    current.g,                   
    max(current.b, pheromone.b), 
    1.0
  );

  if coords.x >= 0 && coords.y >= 0 && coords.x < i32(dims.x) && coords.y < i32(dims.y) {
    textureStore(world_output, coords, result);
  }

  ants[index] = ant;
}
