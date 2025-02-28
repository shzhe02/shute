use std::cell::RefCell;

use encase::{ShaderType, StorageBuffer, UniformBuffer, internal::WriteInto};
use regex::Regex;

use crate::{
    DeviceInfo, Limits,
    buffer::{Buffer, BufferContents, BufferInit, BufferType},
    types::ShaderModule,
};

/// Effectively a reference to a GPU. Obtain a device by using `Instance::autoselect`
/// or `Instance::devices`.
pub struct Device {
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    limits: Limits,
    staging_buffer: RefCell<Option<wgpu::Buffer>>,
    staging_size: RefCell<Option<u32>>,
}

/// Describes the limits imposed on the device.
///
/// These limits are mostly derived from those
/// in [wgpu](https://docs.rs/wgpu/latest/wgpu/struct.Limits.html#).
/// However, the [`downlevel_webgl2_defaults`](https://docs.rs/wgpu/latest/wgpu/struct.Limits.html#method.downlevel_webgl2_defaults) default is unsupported in this library as that set
/// of limits does not allow for any compute shaders to be used
/// (note how all compute workgroup dimensions there are limited to 0).
///
/// Picking `Default` or `Downlevel` limits may be a good idea if you want to ensure
/// compatibility with most devices, especially if you are using constant values for workgroup or
/// dispatch dimensions.
pub enum LimitType {
    /// Sets the limits such that they are as high as the device allows.
    Highest,
    /// Sets the limits to the [wgpu default limits](https://docs.rs/wgpu/latest/wgpu/struct.Limits.html#method.default).
    ///
    /// According to the documentation there:
    /// > This is the set of limits that is guaranteed to work on all modern backends
    /// > and is guaranteed to be supported by WebGPU. Applications needing more
    /// > modern features can use this as a reasonable set of limits if they are
    /// > targeting only desktop and modern mobile devices.
    Default,
    /// Sets the limits to the [`downlevel_defaults`](https://docs.rs/wgpu/latest/wgpu/struct.Limits.html#method.downlevel_defaults) default in wgpu.
    Downlevel,
}

mod private {
    pub trait Sealed {}
}

/// A sealed trait for denoting 3D dimensions.
///
/// This allows for dimension specifications similar to that available in WGSL and CUDA,
/// forcing other dimensions to be set to 1 if not given. If there is only one element in the slice,
/// it is assumed to be the x dimension. If two elements are in the slice, it is assumed that the
/// first and second elements are the x and y dimensions respectively. If all three are given,
/// then the first, second, and third elements are x, y, and z respectively.
///
/// Only slices of length 1, 2, and 3 are allowed.
pub trait Dimensions: private::Sealed {
    fn x(&self) -> u32 {
        1
    }
    fn y(&self) -> u32 {
        1
    }
    fn z(&self) -> u32 {
        1
    }
}

impl private::Sealed for [u32; 1] {}
impl private::Sealed for [u32; 2] {}
impl private::Sealed for [u32; 3] {}

impl Dimensions for [u32; 1] {
    fn x(&self) -> u32 {
        self[0]
    }
}
impl Dimensions for [u32; 2] {
    fn x(&self) -> u32 {
        self[0]
    }
    fn y(&self) -> u32 {
        self[1]
    }
}
impl Dimensions for [u32; 3] {
    fn x(&self) -> u32 {
        self[0]
    }
    fn y(&self) -> u32 {
        self[1]
    }
    fn z(&self) -> u32 {
        self[2]
    }
}

impl Device {
    // TODO: Convert into From<> implementation
    /// Creates a device from a wgpu::Adapter.
    pub(crate) async fn from_adapter(
        adapter: wgpu::Adapter,
        limit_type: LimitType,
    ) -> Result<Device, wgpu::RequestDeviceError> {
        let limits = match limit_type {
            LimitType::Highest => adapter.limits(),
            LimitType::Default => wgpu::Limits::default(),
            LimitType::Downlevel => wgpu::Limits::downlevel_defaults(),
        };
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: limits.clone(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await?;
        Ok(Self {
            adapter,
            device,
            queue,
            limits: Limits::from_wgpu_limits(limits),
            staging_buffer: None.into(),
            staging_size: None.into(),
        })
    }
    /// Gets the limits of the device.
    pub fn limits(&self) -> &Limits {
        &self.limits
    }
    /// Gets the device's information.
    pub fn info(&self) -> DeviceInfo {
        self.adapter.get_info()
    }
    // TODO: Allow for other shader sources too, such as SPIR-V and GLSL.
    /// Creates a compute shader module. Will panic if there are errors in the compute shader.
    pub fn create_shader_module(&self, shader: &str, entry_point: &str) -> ShaderModule {
        ShaderModule::new(
            self.device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: wgpu::ShaderSource::Wgsl(shader.into()),
                }),
            entry_point,
        )
    }
    /// Creates a compute shader module, but override the workgroup size of the entry point function
    /// in the compute shader at runtime.
    pub fn create_shader_module_with_workgroup_size<const N: usize>(
        &self,
        shader: &str,
        entry_point: &str,
        workgroup_dimensions: [u32; N],
    ) -> Option<ShaderModule>
    where
        [u32; N]: Dimensions,
    {
        // FIXME: change option to result later
        //
        // FIXME: this is also some terrible string manipulation. Find a better pure-regex alternative.
        let mut modified_shader = shader.to_string();
        let mut modified = false;
        if let Some(entry_pos) = shader.find(&("fn ".to_string() + entry_point)) {
            let workgroup_size_pattern = Regex::new(r"@workgroup_size\(.*?\)").unwrap();
            let matches = workgroup_size_pattern.find_iter(shader);
            if let Some(found) = matches.filter(|hit| hit.end() < entry_pos).last() {
                let found = found.start();
                let new_workgroup_size = match N {
                    1 => format!("@workgroup_size({})", workgroup_dimensions.x()),
                    2 => format!(
                        "@workgroup_size({}, {})",
                        workgroup_dimensions.x(),
                        workgroup_dimensions.y()
                    ),
                    3 => format!(
                        "@workgroup_size({}, {}, {})",
                        workgroup_dimensions.x(),
                        workgroup_dimensions.y(),
                        workgroup_dimensions.z()
                    ),
                    _ => unreachable!(),
                };
                modified_shader.replace_range(found..entry_pos, &new_workgroup_size);
                modified = true;
            }
        }
        if !modified {
            return None;
        }
        Some(self.create_shader_module(&modified_shader, entry_point))
    }
    /// Creates a buffer.
    pub fn create_buffer<T: ShaderType + WriteInto>(
        &self,
        label: Option<&str>,
        buffer_type: BufferType,
        init_with: BufferInit<T>,
    ) -> Buffer {
        let buffer_contents = match init_with {
            BufferInit::WithSize(size) => BufferContents::Size(size as u32 * size_of::<T>() as u32),
            BufferInit::WithData(data) => match buffer_type {
                BufferType::StorageBuffer { .. } => {
                    let mut buffer = StorageBuffer::new(vec![]);
                    buffer.write(&data).unwrap();
                    BufferContents::Data(buffer.into_inner())
                }
                BufferType::UniformBuffer => {
                    let mut buffer = UniformBuffer::new(vec![]);
                    buffer.write(&data).unwrap();
                    BufferContents::Data(buffer.into_inner())
                }
            },
        };
        let buffer = Buffer::new(label, self, buffer_type, buffer_contents);
        if let BufferType::StorageBuffer { output: true, .. } = buffer_type {}
        buffer
    }
    /// Gets the staging buffer of the device, which is necessary for getting data back
    /// from the GPU.
    pub(crate) fn staging(&self) -> &RefCell<Option<wgpu::Buffer>> {
        &self.staging_buffer
    }
    /// Gets the device as a wgpu device.
    pub(crate) fn device(&self) -> &wgpu::Device {
        &self.device
    }
    /// Gets the queue of the device.
    pub(crate) fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
    /// Overrides the size of the staging buffer.
    pub fn override_staging_size(&self, size: u32) {
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("shute staging buffer"),
            size: size as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        self.staging_buffer.replace(Some(staging_buffer));
        self.staging_size.replace(Some(size));
    }
    /// Executes a compute shader with the given buffers and dispatch dimensions.
    pub fn execute<const N: usize>(
        &self,
        buffers: &Vec<Vec<&mut Buffer<'_>>>,
        shader_module: ShaderModule,
        dispatch_dimensions: [u32; N],
    ) where
        [u32; N]: Dimensions,
    {
        let (bind_group_layouts, bind_groups): (Vec<_>, Vec<_>) = buffers
            .iter()
            .map(|group| {
                let layout_entries: Vec<_> = group
                    .iter()
                    .enumerate()
                    .map(|(binding, buffer)| wgpu::BindGroupLayoutEntry {
                        binding: binding as u32,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: match buffer.buffer_type() {
                                BufferType::StorageBuffer { read_only, .. } => {
                                    wgpu::BufferBindingType::Storage { read_only }
                                }
                                BufferType::UniformBuffer => wgpu::BufferBindingType::Uniform,
                            },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    })
                    .collect();
                let entries: Vec<_> = group
                    .iter()
                    .enumerate()
                    .map(|(binding, buffer)| wgpu::BindGroupEntry {
                        binding: binding as u32,
                        resource: buffer.as_entire_binding(),
                    })
                    .collect();
                let layout =
                    self.device
                        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                            label: None,
                            entries: &layout_entries[..],
                        });
                let group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &layout,
                    entries: &entries[..],
                });
                (layout, group)
            })
            .unzip();
        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &bind_group_layouts
                    .iter()
                    .collect::<Vec<&wgpu::BindGroupLayout>>(),
                push_constant_ranges: &[],
            });
        let pipeline = self
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                module: shader_module.module(),
                entry_point: Some(shader_module.entry_point()),
                compilation_options: Default::default(),
                cache: None,
            });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&pipeline);
            for (idx, bind_group) in bind_groups.iter().enumerate() {
                compute_pass.set_bind_group(idx as u32, bind_group, &[]);
            }
            compute_pass.dispatch_workgroups(
                dispatch_dimensions.x(),
                dispatch_dimensions.y(),
                dispatch_dimensions.z(),
            );
        }
        if let Some(max_output_buffer_size) = buffers
            .iter()
            .flatten()
            .filter(|buffer| buffer.output())
            .map(|buffer| buffer.size())
            .max()
        {
            if let Some(staging_size) = self.staging_size.borrow().as_ref() {
                if *staging_size < max_output_buffer_size {
                    self.override_staging_size(max_output_buffer_size);
                    self.staging_size.replace(Some(max_output_buffer_size));
                }
            } else {
                self.override_staging_size(max_output_buffer_size);
                self.staging_size.replace(Some(max_output_buffer_size));
            }
        }
        self.queue.submit(Some(encoder.finish()));
    }
    /// Copies the data from a GPU-mapped buffer to the staging buffer.
    pub(crate) fn copy_to_staging(&self, buffer: &Buffer) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        if let Some(staging) = self.staging_buffer.borrow().as_ref() {
            encoder.copy_buffer_to_buffer(buffer.buffer(), 0, staging, 0, buffer.size() as u64);
        }
        self.queue.submit(Some(encoder.finish()));
    }
    /// Waits until the GPU queue is empty. That is, this method blocks further execution on the
    /// CPU side until the GPU is done doing all work given to it.
    pub fn synchronize(&self) {
        self.device.poll(wgpu::Maintain::Wait);
    }
}
