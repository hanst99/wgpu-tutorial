use anyhow::Context;
use log::*;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[derive(Deserialize)]
pub struct ModelData {
    positions: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u16>,
}

impl ModelData {
    pub fn vertices(&self) -> Vec<Vertex> {
        self.positions
            .iter()
            .zip(self.uvs.iter())
            .map(|(&position, &uv)| Vertex { position, uv })
            .collect()
    }

    pub fn indices(&self) -> &[u16] {
        &self.indices
    }

    pub fn load(path: &Path) -> anyhow::Result<ModelData> {
        serde_json::from_reader(BufReader::new(
            File::open(path).with_context(|| format!("ModelData::load({:?})", path))?,
        ))
        .map_err(|err| anyhow::Error::from(err))
    }
}

pub struct Model {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_vertices: u32,
}

impl Model {
    pub fn new(device: &wgpu::Device, model_data: &ModelData) -> anyhow::Result<Model> {
        let vertices = model_data.vertices();
        log!(Level::Info, "vertices = #{:?}", vertices);
        let indices = model_data.indices();
        log!(Level::Info, "indices = #{:?}", indices);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_vertices = indices.len() as u32;
        Ok(Self {
            vertex_buffer,
            index_buffer,
            num_vertices,
        })
    }

    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_buffer
    }

    pub fn num_vertices(&self) -> u32 {
        self.num_vertices
    }
}
