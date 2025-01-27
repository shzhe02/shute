use encase::{
    internal::{ReadFrom, WriteInto},
    ShaderType, StorageBuffer, UniformBuffer,
};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindingResource, BufferDescriptor,
};

use crate::Device;

#[derive(Clone, Copy)]
pub enum BufferType {
    StorageBuffer { output: bool, read_only: bool },
    UniformBuffer,
}

pub struct Buffer<'a> {
    device: &'a Device,
    buffer_type: BufferType,
    contents: BufferContents,
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

impl<'a> Buffer<'a> {
    pub fn new(
        label: Option<&str>,
        device: &'a Device,
        buffer_type: BufferType,
        contents: BufferContents,
    ) -> Self {
        let size: u64 = match &contents {
            BufferContents::Size(size) => *size as u64,
            BufferContents::Data(data) => data.len() as u64,
        };
        let usage = {
            let buffer_type = match buffer_type {
                BufferType::StorageBuffer { .. } => wgpu::BufferUsages::STORAGE,
                BufferType::UniformBuffer => wgpu::BufferUsages::UNIFORM,
            };
            buffer_type | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST
        };
        let buffer = match &contents {
            BufferContents::Size(size) => device.device().create_buffer(&BufferDescriptor {
                label,
                size: *size as u64,
                usage,
                mapped_at_creation: false,
            }),
            BufferContents::Data(data) => {
                device.device().create_buffer_init(&BufferInitDescriptor {
                    label,
                    contents: &data[..],
                    usage,
                })
            }
        };
        device.queue().submit([]);

        Self {
            device,
            buffer_type,
            contents,
            buffer,
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
    pub fn send_data_to_device<T>(&self, data: &T)
    where
        T: ShaderType + WriteInto,
    {
        let data: Vec<u8> = match self.buffer_type {
            BufferType::StorageBuffer { .. } => {
                let mut buffer = StorageBuffer::new(vec![]);
                buffer.write(&data).unwrap();
                buffer.into_inner()
            }
            BufferType::UniformBuffer => {
                let mut buffer = UniformBuffer::new(vec![]);
                buffer.write(&data).unwrap();
                buffer.into_inner()
            }
        };
        // TODO: Improve to use write_buffer_with
        self.device.queue().write_buffer(&self.buffer, 0, &data);
        self.device.queue().submit([]);
    }
    pub async fn fetch_data_from_device<T>(&self, output: &mut T)
    where
        T: ShaderType + ReadFrom,
    {
        // TODO: Return an error if the output is not large enough to hold the buffer's data.
        if let Some(staging) = self.staging() {
            let slice = staging.slice(..);
            let (tx, rx) = flume::bounded(1);
            slice.map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
            self.device
                .device()
                .poll(wgpu::Maintain::wait())
                .panic_on_timeout();
            rx.recv_async().await.unwrap().unwrap();
            {
                let view = slice.get_mapped_range();
                let buffer = StorageBuffer::new(&*view);
                buffer.read(output).unwrap();
            }
            staging.unmap();
        }
    }
}
