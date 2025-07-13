struct Ant {
    pos: vec2<f32>,
    angle: f32,
};

@group(0) @binding(0) var<storage, read_write> ants: array<Ant>;
@group(0) @binding(1) var pheromone_map: texture_storage_2d<rgba8unorm, write>;

const PI: f32 = 3.14159265359;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    var ant = ants[index];

    ant.angle += (fract(sin(dot(ant.pos, vec2<f32>(12.9898, 78.233))) * 43758.5453) - 0.5) * 0.5;
    let dir = vec2<f32>(cos(ant.angle), sin(ant.angle));
    ant.pos += dir * 2.0;

    let dims = vec2<f32>(textureDimensions(pheromone_map));
    if (ant.pos.x < 0.0) { ant.pos.x += dims.x; }
    if (ant.pos.y < 0.0) { ant.pos.y += dims.y; }
    if (ant.pos.x >= dims.x) { ant.pos.x -= dims.x; }
    if (ant.pos.y >= dims.y) { ant.pos.y -= dims.y; }

    let tex_coords = vec2<i32>(floor(ant.pos));
    textureStore(pheromone_map, tex_coords, vec4<f32>(1.0, 0.0, 1.0, 1.0));

    ants[index] = ant;
}
