use wgpu::BindingResource;

#[derive(Clone, Copy)]
pub enum BufferType {
    StorageBuffer,
    UniformBuffer,
}

pub enum BufferUsage {
    CopyDestination,
    CopySource,
}

pub struct Buffer {
    buffer_type: BufferType,
    initial_data: Option<Vec<u8>>,
    output_data: Option<Vec<u8>>,
    size: u64,
    buffer: wgpu::Buffer,
    staging: Option<wgpu::Buffer>,
}

impl Buffer {
    pub fn new(
        buffer_type: BufferType,
        initial_data: Option<Vec<u8>>,
        output_data: Option<Vec<u8>>,
        size: u64,
        buffer: wgpu::Buffer,
        staging: Option<wgpu::Buffer>,
    ) -> Self {
        Self {
            buffer_type,
            initial_data,
            output_data,
            size,
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
    pub fn size(&self) -> u64 {
        self.size
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
    pub fn initial_data(&self) -> &Option<Vec<u8>> {
        &self.initial_data
    }
    pub fn staging(&self) -> &Option<wgpu::Buffer> {
        &self.staging
    }
}
