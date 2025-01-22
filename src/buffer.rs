use wgpu::BindingResource;

use crate::ShaderType;

#[derive(Clone, Copy)]
pub enum BufferType {
    StorageBuffer { output: bool, read_only: bool },
    UniformBuffer,
}

pub struct Buffer<T>
where
    T: ShaderType,
{
    buffer_type: BufferType,
    init_with: BufferInit<T>,
    output_data: Option<Vec<u8>>,
    buffer: wgpu::Buffer,
    staging: Option<wgpu::Buffer>,
}

pub enum BufferInit<T>
where
    T: ShaderType,
{
    WithSize(u32),
    WithData(T),
}

impl<T> BufferInit<T>
where
    T: ShaderType,
{
    pub fn size(&self, buffer_type: BufferType) -> u32 {
        match self {
            BufferInit::WithSize(size) => *size,
            BufferInit::WithData(_) => self.to_bytes(buffer_type).unwrap().len() as u32,
        }
    }
    pub fn to_bytes(&self, buffer_type: BufferType) -> Option<Vec<u8>> {
        let data = match &self {
            BufferInit::WithSize(_) => {
                return None;
            }
            BufferInit::WithData(data) => data,
        };
        match buffer_type {
            BufferType::StorageBuffer { .. } => {
                let mut buffer: encase::StorageBuffer<_> = encase::StorageBuffer::new(vec![]);
                buffer.write(&data).unwrap();
                Some(buffer.into_inner())
            }
            BufferType::UniformBuffer => {
                let mut buffer: encase::UniformBuffer<_> = encase::UniformBuffer::new(vec![]);
                buffer.write(&data).unwrap();
                Some(buffer.into_inner())
            }
        }
    }
}

impl<T> Buffer<T>
where
    T: ShaderType,
{
    pub fn new(
        buffer_type: BufferType,
        init_with: BufferInit<T>,
        output_data: Option<Vec<u8>>,
        buffer: wgpu::Buffer,
        staging: Option<wgpu::Buffer>,
    ) -> Self {
        Self {
            buffer_type,
            init_with,
            output_data,
            buffer,
            staging,
        }
    }
    pub fn write_output_data(&mut self, data: Vec<u8>) {
        self.output_data = Some(data);
    }
    pub fn read_output_data(&self) -> &Option<Vec<u8>> {
        &self.output_data
    }
    pub fn size(&self) -> u32 {
        self.init_with.size(self.buffer_type)
    }
    pub fn init_data(&self) -> Option<Vec<u8>> {
        self.init_with.to_bytes(self.buffer_type)
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
    pub fn init_with(&self) -> &BufferInit<T> {
        &self.init_with
    }
    pub fn staging(&self) -> &Option<wgpu::Buffer> {
        &self.staging
    }
}
