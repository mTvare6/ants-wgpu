struct Ant {
    pos: vec2<f32>,
    vel: vec2<f32>,
};

@group(0) @binding(0) var<storage, read_write> ants: array<Ant>;
@group(0) @binding(1) var pheromone_map: texture_storage_2d<rgba8unorm, read_write>;

const WIDTH: f32 = 1280.0; 
const HEIGHT: f32 = 720.0; 
const SPEED: f32 = 1.0;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= arrayLength(&ants)) {
        return;
    }

    var ant = ants[index];

    
    ant.pos += ant.vel * SPEED;

    
    if (ant.pos.x < 0.0 || ant.pos.x > WIDTH) {
        ant.vel.x *= -1.0;
        ant.pos.x = clamp(ant.pos.x, 0.0, WIDTH);
    }
    if (ant.pos.y < 0.0 || ant.pos.y > HEIGHT) {
        ant.vel.y *= -1.0;
        ant.pos.y = clamp(ant.pos.y, 0.0, HEIGHT);
    }
    
    
    ants[index] = ant;

    
    let tex_coords = vec2<i32>(floor(ant.pos));
    
    
    let existing_color = textureLoad(pheromone_map, tex_coords);
    let new_color = existing_color + vec4<f32>(0.1, 0.0, 0.1, 0.0); // Magenta trail 
    
    textureStore(pheromone_map, tex_coords, clamp(new_color, vec4(0.0), vec4(1.0)));
}
