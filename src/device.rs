use std::path::{Path, PathBuf};

use crate::{
    buffer::{Buffer, BufferType},
    types::ShaderModule,
};

pub struct Device {
    device: wgpu::Device,
    queue: wgpu::Queue,
    limits: wgpu::Limits,
}

impl Device {
    pub async fn new(adapter: wgpu::Adapter) -> Result<Device, wgpu::RequestDeviceError> {
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await?;
        Ok(Self {
            device,
            queue,
            limits: adapter.limits(),
        })
    }
    pub fn get_limits(&self) -> &wgpu::Limits {
        &self.limits
    }
    pub fn create_shader_module(
        &self,
        path: impl AsRef<Path>,
        entry_point: String,
    ) -> ShaderModule {
        let mut path_buf = PathBuf::new();
        path_buf.push(path);
        ShaderModule {
            module: self
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some(&path_buf.display().to_string()),
                    source: wgpu::ShaderSource::Wgsl(path_buf.to_string_lossy()),
                }),
            entry_point,
        }
    }
    pub fn create_buffer(
        &self,
        label: Option<&str>,
        buffer_type: BufferType,
        buffer_size: u64,
        initial_data: Option<Vec<u8>>,
        output: bool,
    ) -> Buffer {
        let size = if let Some(data) = &initial_data {
            data.len() as u64
        } else {
            buffer_size
        };
        Buffer::new(
            buffer_type,
            initial_data,
            None,
            size,
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label,
                size,
                usage: {
                    let buffer_type = match buffer_type {
                        BufferType::StorageBuffer => wgpu::BufferUsages::STORAGE,
                        BufferType::UniformBuffer => wgpu::BufferUsages::UNIFORM,
                    };
                    buffer_type | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC
                },
                mapped_at_creation: false,
            }),
            if output {
                Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                    label,
                    size,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                }))
            } else {
                None
            },
        )
    }
    pub async fn execute(
        &self,
        buffers: &mut Vec<Vec<&mut Buffer>>,
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
                            ty: match buffer.binding_type {
                                BufferType::StorageBuffer => {
                                    wgpu::BufferBindingType::Storage { read_only: false }
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
                        resource: buffer.buffer.as_entire_binding(),
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
                module: &shader_module.module,
                entry_point: Some(&shader_module.entry_point),
                compilation_options: Default::default(),
                cache: None,
            });
        for buffer_group in buffers.iter() {
            for buffer in buffer_group {
                if let Some(initial_data) = &buffer.initial_data {
                    self.queue
                        .write_buffer(&buffer.buffer, 0, &initial_data[..]);
                }
            }
        }
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
                if let Some(staging) = &buffer.staging {
                    encoder.copy_buffer_to_buffer(&buffer.buffer, 0, &staging, 0, buffer.size);
                }
            }
        }
        self.queue.submit(Some(encoder.finish()));
        for buffer_group in buffers.iter_mut() {
            for buffer in buffer_group {
                let mut output_data: Vec<u8> = vec![0; buffer.size as usize];
                if let Some(staging) = &buffer.staging {
                    let slice = staging.slice(..);
                    let (tx, rx) = flume::bounded(1);
                    slice.map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
                    self.device.poll(wgpu::Maintain::wait()).panic_on_timeout();
                    rx.recv_async().await.unwrap().unwrap();
                    {
                        let view = slice.get_mapped_range();
                        output_data.copy_from_slice(bytemuck::cast_slice(&view));
                    }
                    staging.unmap();
                }
                if buffer.staging.is_some() {
                    buffer.write_output_data(output_data);
                }
            }
        }
    }
}
