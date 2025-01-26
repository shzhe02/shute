use std::path::{Path, PathBuf};

use encase::{internal::WriteInto, ShaderType, StorageBuffer, UniformBuffer};
use wgpu::Maintain;

use crate::{
    buffer::{Buffer, BufferContents, BufferInit, BufferType},
    types::ShaderModule,
    Limits,
};

pub struct Device {
    device: wgpu::Device,
    queue: wgpu::Queue,
    limits: Limits,
}

pub enum LimitType {
    Highest,
    Default,
    Downlevel,
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
            device,
            queue,
            limits: Limits::from_wgpu_limits(limits),
        })
    }
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, limits: wgpu::Limits) -> Self {
        Self {
            device,
            queue,
            limits: Limits::from_wgpu_limits(limits),
        }
    }
    pub fn limits(&self) -> &Limits {
        &self.limits
    }
    pub fn create_shader_module(
        &self,
        path: impl AsRef<Path>,
        entry_point: String,
    ) -> ShaderModule {
        let mut path_buf = PathBuf::new();
        path_buf.push(path);
        ShaderModule::new(
            self.device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some(&path_buf.display().to_string()),
                    source: wgpu::ShaderSource::Wgsl(path_buf.to_string_lossy()),
                }),
            entry_point,
        )
    }
    pub fn create_buffer<T: ShaderType + WriteInto>(
        &self,
        label: Option<&str>,
        buffer_type: BufferType,
        init_with: BufferInit<T>,
    ) -> Buffer {
        let buffer_contents = match init_with {
            BufferInit::WithSize(size) => BufferContents::Size(size),
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
        Buffer::new(label, self, buffer_type, buffer_contents)
    }
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
    pub fn send_all_data_to_device(&self, buffers: &Vec<Vec<&mut Buffer>>) {
        for buffer_group in buffers.iter() {
            for buffer in buffer_group {
                if let Some(data) = &buffer.data() {
                    self.queue.write_buffer(buffer.buffer(), 0, &data[..]);
                }
            }
        }
        self.queue.submit([]);
    }
    pub async fn execute_async(
        &self,
        buffers: &Vec<Vec<&mut Buffer>>,
        shader_module: ShaderModule,
        workgroup_dimensions: (u32, u32, u32),
    ) {
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
                workgroup_dimensions.0,
                workgroup_dimensions.1,
                workgroup_dimensions.2,
            );
        }
        for buffer_group in buffers.iter() {
            for buffer in buffer_group {
                if let Some(staging) = buffer.staging() {
                    encoder.copy_buffer_to_buffer(
                        buffer.buffer(),
                        0,
                        staging,
                        0,
                        buffer.size() as u64,
                    );
                }
            }
        }
        self.queue.submit(Some(encoder.finish()));
    }
    pub async fn execute_blocking(
        &self,
        buffers: &Vec<Vec<&mut Buffer>>,
        shader_module: ShaderModule,
        workgroup_dimensions: (u32, u32, u32),
    ) {
        self.execute_async(buffers, shader_module, workgroup_dimensions)
            .await;
        self.device().poll(wgpu::MaintainBase::Wait);
    }
    pub fn block_until_complete(&self) {
        self.device.poll(Maintain::Wait);
    }
}
