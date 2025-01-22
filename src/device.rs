use std::path::{Path, PathBuf};

use crate::{
    buffer::{Buffer, BufferInit, BufferType},
    types::ShaderModule,
    ShaderType,
};

pub struct Device {
    device: wgpu::Device,
    queue: wgpu::Queue,
    limits: wgpu::Limits,
}

impl Device {
    pub async fn from_adapter(adapter: wgpu::Adapter) -> Result<Device, wgpu::RequestDeviceError> {
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
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, limits: wgpu::Limits) -> Self {
        Self {
            device,
            queue,
            limits,
        }
    }
    pub fn limits(&self) -> &wgpu::Limits {
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
    pub fn create_buffer<T: ShaderType>(
        &self,
        label: Option<&str>,
        buffer_type: BufferType,
        init_with: BufferInit<T>,
    ) -> Buffer<T> {
        let size = init_with.size(buffer_type) as u64;
        let output = if let BufferType::StorageBuffer { output, .. } = buffer_type {
            output
        } else {
            false
        };
        Buffer::new(
            buffer_type,
            init_with,
            None,
            self.device.create_buffer(&wgpu::BufferDescriptor {
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
    pub async fn execute<T>(
        &self,
        buffers: &mut Vec<Vec<&mut Buffer<T>>>,
        shader_module: ShaderModule,
        workgroup_dimensions: (u32, u32, u32),
    ) where
        T: ShaderType,
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
                module: &shader_module.module(),
                entry_point: Some(&shader_module.entry_point()),
                compilation_options: Default::default(),
                cache: None,
            });
        for buffer_group in buffers.iter() {
            for buffer in buffer_group {
                if let BufferInit::WithData(_) = &buffer.init_with() {
                    self.queue
                        .write_buffer(&buffer.buffer(), 0, &buffer.init_data().unwrap()[..]);
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
                if let Some(staging) = buffer.staging() {
                    encoder.copy_buffer_to_buffer(
                        buffer.buffer(),
                        0,
                        &staging,
                        0,
                        buffer.size() as u64,
                    );
                }
            }
        }
        self.queue.submit(Some(encoder.finish()));
        for buffer_group in buffers.iter_mut() {
            for buffer in buffer_group {
                let mut output_data: Vec<u8> = vec![0; buffer.size() as usize];
                if let Some(staging) = buffer.staging() {
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
                if buffer.staging().is_some() {
                    buffer.write_output_data(output_data);
                }
            }
        }
    }
}
