// Vertex Shader

struct VertexOutput {
  [[builtin(position)]] clip_position: vec4<f32>;
  [[location(0)]] color: vec3<f32>;
};

struct Vertex {
 [[location(0)]] position: vec3<f32>;
 [[location(1)]] color: vec3<f32>;
};

[[stage(vertex)]]
fn vs_main(vertex: Vertex)
  -> VertexOutput {
  var out: VertexOutput;

  out.clip_position = vec4<f32>(vec3<f32>(vertex.position.x, vertex.position.y - 0.1, vertex.position.z), 1.0);
  out.color = pow(vertex.color, vec3<f32>(2.2, 2.2, 2.2));
  return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
  return vec4<f32>(in.color, 1.0);
}