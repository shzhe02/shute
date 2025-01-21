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
    binding_type: BufferType,
    initial_data: Option<Vec<u8>>,
    output_data: Option<Vec<u8>>,
    size: u64,
    buffer: wgpu::Buffer,
    staging: Option<wgpu::Buffer>,
}

impl Buffer {
    pub fn new(
        binding_type: BufferType,
        initial_data: Option<Vec<u8>>,
        output_data: Option<Vec<u8>>,
        size: u64,
        buffer: wgpu::Buffer,
        staging: Option<wgpu::Buffer>,
    ) -> Self {
        Self {
            binding_type,
            initial_data,
            output_data,
            size,
            buffer,
            staging,
        }
    }
    fn write_output_data(&mut self, data: Vec<u8>) {
        self.output_data = Some(data);
    }
    pub fn read_output_data(&self) -> &Option<Vec<u8>> {
        &self.output_data
    }
    pub fn get_size(&self) -> u64 {
        self.size
    }
}
