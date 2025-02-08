use std::cell::RefCell;

use encase::{internal::WriteInto, ShaderType, StorageBuffer, UniformBuffer};
use regex::Regex;

use crate::{
    buffer::{Buffer, BufferContents, BufferInit, BufferType},
    types::ShaderModule,
    Limits,
};

pub struct Device {
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    limits: Limits,
    staging_buffer: RefCell<Option<wgpu::Buffer>>,
}

pub enum LimitType {
    Highest,
    Default,
    Downlevel,
}

mod private {
    pub trait Sealed {}
}

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
    pub async fn from_adapter(
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
        })
    }
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        limits: wgpu::Limits,
        adapter: wgpu::Adapter,
    ) -> Self {
        Self {
            adapter,
            device,
            queue,
            limits: Limits::from_wgpu_limits(limits),
            staging_buffer: None.into(),
        }
    }
    pub fn limits(&self) -> &Limits {
        &self.limits
    }
    pub fn info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }
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
    pub fn staging(&self) -> &RefCell<Option<wgpu::Buffer>> {
        &self.staging_buffer
    }
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
    pub fn override_staging(&self, size: u32) {
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("shute staging buffer"),
            size: size as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        self.staging_buffer.replace(Some(staging_buffer));
    }
    pub async fn execute_async<const N: usize>(
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
            self.override_staging(max_output_buffer_size);
        }
        self.queue.submit(Some(encoder.finish()));
    }
    pub fn stage_output(&self, buffer: &Buffer) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        if let Some(staging) = self.staging_buffer.borrow().as_ref() {
            encoder.copy_buffer_to_buffer(buffer.buffer(), 0, staging, 0, buffer.size() as u64);
        }
        self.queue.submit(Some(encoder.finish()));
    }
    pub fn execute_blocking<const N: usize>(
        &self,
        buffers: &Vec<Vec<&mut Buffer<'_>>>,
        shader_module: ShaderModule,
        dispatch_dimensions: [u32; N],
    ) where
        [u32; N]: Dimensions,
    {
        pollster::block_on(self.execute_async(buffers, shader_module, dispatch_dimensions));
        self.device().poll(wgpu::MaintainBase::Wait);
    }
    pub fn block_until_complete(&self) {
        self.device.poll(wgpu::Maintain::Wait);
    }
}
