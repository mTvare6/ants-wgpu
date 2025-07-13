
struct AntInput {
    @location(0) pos: vec2<f32>,
    @location(1) vel: vec2<f32>,
};


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@group(0) @binding(0) var t_pheromone: texture_2d<f32>;
@group(0) @binding(1) var s_pheromone: sampler;

const WIDTH: f32 = 1280.0;
const HEIGHT: f32 = 720.0;



@vertex
fn vs_pheromone(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    // Generate fullscreen quad with proper UV coordinates
    var pos: vec2<f32>;
    var uv: vec2<f32>;
    
    switch in_vertex_index {
        case 0u: { pos = vec2<f32>(-1.0, -1.0); uv = vec2<f32>(0.0, 1.0); }
        case 1u: { pos = vec2<f32>( 1.0, -1.0); uv = vec2<f32>(1.0, 1.0); }
        case 2u: { pos = vec2<f32>( 1.0,  1.0); uv = vec2<f32>(1.0, 0.0); }
        case 3u: { pos = vec2<f32>(-1.0, -1.0); uv = vec2<f32>(0.0, 1.0); }
        case 4u: { pos = vec2<f32>( 1.0,  1.0); uv = vec2<f32>(1.0, 0.0); }
        default: { pos = vec2<f32>(-1.0,  1.0); uv = vec2<f32>(0.0, 0.0); }
    }
    
    out.clip_position = vec4<f32>(pos, 0.0, 1.0);
    out.color = vec4<f32>(0.0, 0.0, 0.0, 0.0); // Transparent, will use UV to sample texture
    out.uv = uv;
    return out;
}


@vertex
fn vs_ant(
    @builtin(vertex_index) in_vertex_index: u32,
    ant: AntInput
) -> VertexOutput {
    var out: VertexOutput;
    
    
    let screen_pos = ant.pos / vec2<f32>(WIDTH, HEIGHT) * 2.0 - 1.0;
    
    
    var corner_offset: vec2<f32>;
    switch in_vertex_index {
        case 0u: { corner_offset = vec2<f32>(-1.0, -1.0); }
        case 1u: { corner_offset = vec2<f32>(1.0, -1.0); }
        case 2u: { corner_offset = vec2<f32>(1.0, 1.0); }
        case 3u: { corner_offset = vec2<f32>(-1.0, -1.0); }
        case 4u: { corner_offset = vec2<f32>(1.0, 1.0); }
        default: { corner_offset = vec2<f32>(-1.0, 1.0); } 
    }

    let size = 2.0 / vec2<f32>(WIDTH, HEIGHT); 
    out.clip_position = vec4<f32>(screen_pos + corner_offset * size, 0.0, 1.0);
    out.color = vec4<f32>(1.0, 1.0, 0.0, 1.0); // Yellow ants
    out.uv = vec2<f32>(0.0, 0.0); // UV not needed for ants
    
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // If color has alpha > 0, it's an ant - render the ant color
    if (in.color.a > 0.0) {
        return in.color;
    } else { 
        // Otherwise, it's the pheromone background - use the UV coordinate
        let color = textureSample(t_pheromone, s_pheromone, in.uv);
        return color;
    }
}
