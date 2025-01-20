use std::path::{Path, PathBuf};

use encase::ShaderType;

pub struct Instance {
    instance: wgpu::Instance,
}

pub type PowerPreference = wgpu::PowerPreference;

impl Instance {
    pub fn new() -> Instance {
        Instance {
            instance: wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                flags: if cfg!(debug_assertions) {
                    wgpu::InstanceFlags::DEBUG
                        | wgpu::InstanceFlags::VALIDATION
                        | wgpu::InstanceFlags::GPU_BASED_VALIDATION
                } else {
                    wgpu::InstanceFlags::DISCARD_HAL_LABELS
                },
                dx12_shader_compiler: wgpu::Dx12Compiler::Fxc, // TODO: Somehow make this modifiable to Dxc
                gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
            }),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_devices(&self) -> Vec<Adapter> {
        self.instance.enumerate_adapters(wgpu::Backends::all())
    }
    pub async fn autoselect(
        &self,
        power_preference: PowerPreference,
    ) -> Result<Device, wgpu::RequestDeviceError> {
        let adapter = self
            .instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: adapter.limits(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await?;
        Ok(Device {
            device,
            queue,
            limits: adapter.limits(),
        })
    }
}

pub type Adapter = wgpu::Adapter;

pub struct Device {
    device: wgpu::Device,
    queue: wgpu::Queue,
    limits: wgpu::Limits,
}

#[derive(Debug)]
pub struct Limits {
    max_bind_groups: u32,
    max_bindings_per_bind_group: u32,
    max_dynamic_uniform_buffers_per_pipeline_layout: u32,
    max_dynamic_storage_buffers_per_pipeline_layout: u32,
    max_uniform_buffer_binding_size: u32,
    max_storage_buffer_binding_size: u32,
    max_buffer_size: u64,
    min_uniform_buffer_offset_alignment: u32,
    min_storage_buffer_offset_alignment: u32,
    max_compute_workgroup_storage_size: u32,
    max_compute_invocations_per_workgroup: u32,
    max_compute_workgroup_size_x: u32,
    max_compute_workgroup_size_y: u32,
    max_compute_workgroup_size_z: u32,
    max_compute_workgroups_per_dimension: u32,
    min_subgroup_size: u32,
    max_subgroup_size: u32,
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
    pub fn get_limits(&self) -> Limits {
        Limits {
            max_bind_groups: self.limits.max_bind_groups,
            max_bindings_per_bind_group: self.limits.max_bindings_per_bind_group,
            max_dynamic_uniform_buffers_per_pipeline_layout: self
                .limits
                .max_dynamic_uniform_buffers_per_pipeline_layout,
            max_dynamic_storage_buffers_per_pipeline_layout: self
                .limits
                .max_dynamic_storage_buffers_per_pipeline_layout,
            max_uniform_buffer_binding_size: self.limits.max_uniform_buffer_binding_size,
            max_storage_buffer_binding_size: self.limits.max_storage_buffer_binding_size,
            max_buffer_size: self.limits.max_buffer_size,
            min_uniform_buffer_offset_alignment: self.limits.min_uniform_buffer_offset_alignment,
            min_storage_buffer_offset_alignment: self.limits.min_storage_buffer_offset_alignment,
            max_compute_workgroup_storage_size: self.limits.max_compute_workgroup_storage_size,
            max_compute_invocations_per_workgroup: self
                .limits
                .max_compute_invocations_per_workgroup,
            max_compute_workgroup_size_x: self.limits.max_compute_workgroup_size_x,
            max_compute_workgroup_size_y: self.limits.max_compute_workgroup_size_y,
            max_compute_workgroup_size_z: self.limits.max_compute_workgroup_size_z,
            max_compute_workgroups_per_dimension: self.limits.max_compute_workgroups_per_dimension,
            min_subgroup_size: self.limits.min_subgroup_size,
            max_subgroup_size: self.limits.max_subgroup_size,
        }
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
    pub fn create_buffer<T: ShaderType>(
        &self,
        label: Option<&str>,
        buffer_init: BufferInit<T>,
        output: bool,
    ) -> Buffer {
        let buffer_type = match buffer_init {
            BufferInit::StorageBufferSize(_) | BufferInit::StorageBufferContents(_) => {
                BufferType::StorageBuffer
            }
            BufferInit::UniformBufferSize(_) | BufferInit::UniformBufferContents(_) => {
                BufferType::UniformBuffer
            }
        };
        let initial_data = match buffer_init {
            BufferInit::StorageBufferContents(contents) => {
                let mut buffer = encase::StorageBuffer::new(vec![]);
                buffer.write(&contents).unwrap();
                Some(buffer.into_inner())
            }
            BufferInit::UniformBufferContents(contents) => {
                let mut buffer = encase::UniformBuffer::new(vec![]);
                buffer.write(&contents).unwrap();
                Some(buffer.into_inner())
            }
            _ => None,
        };
        let size = match buffer_init {
            BufferInit::StorageBufferSize(s) => s,
            BufferInit::UniformBufferSize(s) => s,
            _ => 0,
        };
        Buffer {
            initial_data,
            output_data: None,
            size,
            binding_type: buffer_type,
            buffer: self.device.create_buffer(&wgpu::BufferDescriptor {
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
            staging: if output {
                Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                    label,
                    size,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                }))
            } else {
                None
            },
        }
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

#[derive(Clone, Copy)]
pub enum BufferInit<T>
where
    T: ShaderType,
{
    StorageBufferSize(u32),
    UniformBufferSize(u32),
    StorageBufferContents(T),
    UniformBufferContents(T),
}

enum BufferType {
    StorageBuffer,
    UniformBuffer,
}

pub struct ShaderModule {
    module: wgpu::ShaderModule,
    entry_point: String,
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
