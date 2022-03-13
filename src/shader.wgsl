// Vertex Shader

struct VertexOutput {
  [[builtin(position)]] clip_position: vec4<f32>;
  [[location(9)]] uv: vec2<f32>;
};

struct Vertex {
 [[location(0)]] position: vec3<f32>;
 [[location(1)]] uv: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(vertex: Vertex)
  -> VertexOutput {
  var out: VertexOutput;

  out.clip_position = vec4<f32>(vec3<f32>(vertex.position.x, vertex.position.y - 0.1, vertex.position.z), 1.0);
  out.uv = vertex.uv;
  return out;
}

[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;

[[group(0),binding(1)]]
var s_diffuse: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
  return textureSample(t_diffuse, s_diffuse, in.uv);
}