use encase::{internal::WriteInto, ShaderType};
use wgpu::BindingResource;

use crate::Device;

#[derive(Clone, Copy)]
pub enum BufferType {
    StorageBuffer { output: bool, read_only: bool },
    UniformBuffer,
}

pub struct Buffer {
    buffer_type: BufferType,
    contents: BufferContents,
    output_data: Option<Vec<u8>>,
    buffer: wgpu::Buffer,
    staging: Option<wgpu::Buffer>,
}

pub enum BufferInit<T>
where
    T: ShaderType + WriteInto,
{
    WithSize(u32),
    WithData(T),
}

pub enum BufferContents {
    Size(u32),
    Data(Vec<u8>),
}

impl BufferContents {
    pub fn size(&self) -> u32 {
        match self {
            BufferContents::Size(size) => *size,
            BufferContents::Data(data) => data.len() as u32,
        }
    }
}

impl Buffer {
    pub fn new(
        label: Option<&str>,
        device: &Device,
        buffer_type: BufferType,
        contents: BufferContents,
    ) -> Self {
        //     if output {
        //         Some(self.device.create_buffer(&wgpu::BufferDescriptor {
        //             label,
        //             size,
        //             usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        //             mapped_at_creation: false,
        //         }))
        //     } else {
        //         None
        //     },
        let size: u64 = match &contents {
            BufferContents::Size(size) => *size as u64,
            BufferContents::Data(data) => data.len() as u64,
        };
        Self {
            buffer_type,
            contents,
            output_data: None,
            buffer: device.device().create_buffer(&wgpu::BufferDescriptor {
                label,
                size,
                usage: {
                    let buffer_type = match buffer_type {
                        BufferType::StorageBuffer { .. } => wgpu::BufferUsages::STORAGE,
                        BufferType::UniformBuffer => wgpu::BufferUsages::UNIFORM,
                    };
                    buffer_type | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST
                },
                mapped_at_creation: false,
            }),
            staging: if let BufferType::StorageBuffer { output: true, .. } = buffer_type {
                Some(device.device().create_buffer(&wgpu::BufferDescriptor {
                    label: label.map(|s| s.to_string() + "-output").as_deref(),
                    size,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                }))
            } else {
                None
            },
        }
    }
    pub fn write_output_data(&mut self, data: Vec<u8>) {
        self.output_data = Some(data);
    }
    pub fn read_output_data(&self) -> &Option<Vec<u8>> {
        &self.output_data
    }
    pub fn size(&self) -> u32 {
        self.contents.size()
    }
    pub fn data(&self) -> Option<&Vec<u8>> {
        match &self.contents {
            BufferContents::Size(_) => None,
            BufferContents::Data(data) => Some(data),
        }
    }
    pub fn buffer_type(&self) -> BufferType {
        self.buffer_type
    }
    pub fn as_entire_binding(&self) -> BindingResource<'_> {
        self.buffer.as_entire_binding()
    }
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
    pub fn staging(&self) -> &Option<wgpu::Buffer> {
        &self.staging
    }
}
