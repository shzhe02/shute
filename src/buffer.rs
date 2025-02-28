use encase::{
    ShaderType, StorageBuffer, UniformBuffer,
    internal::{ReadFrom, WriteInto},
};
use wgpu::{
    BindingResource, BufferDescriptor,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::Device;

/// Specifies buffer type.
#[derive(Clone, Copy)]
pub enum BufferType {
    /// A storage buffer. Usually used for holding large data types like vectors.
    /// Storage buffers are also the only buffer type that can be mutable.
    StorageBuffer {
        /// Denotes if the buffer should be accessible by the CPU after being used by the GPU.
        output: bool,
        /// Denotes if the buffer is read-only or mutable on the GPU side.
        read_only: bool,
    },
    /// A uniform buffer. Usually used for holding read-only data like constant parameters.
    UniformBuffer,
}

/// A buffer for sharing data between the CPU and GPU.
///
/// Create a buffer using the `Device::create_buffer` method.
pub struct Buffer<'a> {
    device: &'a Device,
    buffer_type: BufferType,
    contents: BufferContents,
    buffer: wgpu::Buffer,
    // staging: Option<wgpu::Buffer>,
}

/// Specifies how a buffer is initialized.
pub enum BufferInit<T>
where
    T: ShaderType + WriteInto,
{
    /// Initialize a buffer with a fixed size.
    WithSize(usize),
    /// Initialize a buffer with initial data.
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
    /// Used to create a new buffer. However, this method is sealed.
    /// Use `Device::create_buffer` instead.
    pub(crate) fn new(
        label: Option<&str>,
        device: &'a Device,
        buffer_type: BufferType,
        contents: BufferContents,
    ) -> Self {
        // let size: u64 = match &contents {
        //     BufferContents::Size(size) => *size as u64,
        //     BufferContents::Data(data) => data.len() as u64,
        // };
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
            // staging: if let BufferType::StorageBuffer { output: true, .. } = buffer_type {
            //     Some(device.device().create_buffer(&wgpu::BufferDescriptor {
            //         label: label.map(|s| s.to_string() + "-output").as_deref(),
            //         size,
            //         usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            //         mapped_at_creation: false,
            //     }))
            // } else {
            //     None
            // },
        }
    }
    /// Get the size of the buffer (in bytes).
    pub fn size(&self) -> u32 {
        self.contents.size()
    }
    /// Check if the buffer is an output buffer (i.e., readable from the CPU
    /// after being used by the GPU) or not.
    pub fn output(&self) -> bool {
        matches!(
            self.buffer_type,
            BufferType::StorageBuffer { output: true, .. }
        )
    }
    /// Get a reference to the data stored in the buffer (as bytes).
    pub fn data(&self) -> Option<&Vec<u8>> {
        match &self.contents {
            BufferContents::Size(_) => None,
            BufferContents::Data(data) => Some(data),
        }
    }
    /// Get the type of the buffer.
    pub fn buffer_type(&self) -> BufferType {
        self.buffer_type
    }
    /// Note: This method is meant to be used only within the crate.
    ///
    /// Return the binding view of the entire buffer.
    pub(crate) fn as_entire_binding(&self) -> BindingResource<'_> {
        self.buffer.as_entire_binding()
    }
    pub(crate) fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
    // pub fn staging(&self) -> &Option<wgpu::Buffer> {
    //     &self.staging
    // }
    // TODO: This should be "write_buffer" or similar.
    /// Write data to the buffer.
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
    // TODO: Make this "read_buffer" or similar.
    /// Get the data from the buffer. This makes the buffer temporarily accessible
    /// to the CPU to write the buffer contents to the output mutable reference.
    pub async fn fetch_data_from_device<T>(&self, output: &mut T)
    where
        T: ShaderType + ReadFrom,
    {
        if !self.output() {
            return;
        }
        self.device.stage_output(self);

        // TODO: Return an error if the output is not large enough to hold the buffer's data.
        let staging = self.device.staging().borrow();
        if let Some(staging) = staging.as_ref() {
            let output_size = self.size() as u64;
            let slice = staging.slice(..output_size);
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
