pub use shute_macros::load_wgsl_shader;
use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, Queue,
        QueueCreateInfo, QueueFlags,
    },
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{
        AllocationCreateInfo, FreeListAllocator, GenericMemoryAllocator, MemoryTypeFilter,
        StandardMemoryAllocator,
    },
    pipeline::{
        compute::ComputePipelineCreateInfo, layout::PipelineDescriptorSetLayoutCreateInfo,
        ComputePipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
    },
    shader::{ShaderModule, ShaderModuleCreateInfo},
    sync::{self, GpuFuture},
    VulkanLibrary,
};

use std::sync::Arc;
use std::{collections::HashMap, ops::Deref};

pub struct ComputeDevice<T> {
    device: Arc<Device>,
    queue: Arc<Queue>,
    memory_allocator: Arc<GenericMemoryAllocator<FreeListAllocator>>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    data_buffers: HashMap<u32, Subbuffer<[T]>>,
    pipeline: Arc<ComputePipeline>,
}

impl<T> ComputeDevice<T> {
    pub fn autoselect(shader: &Vec<u32>, entry_point: &str) -> Self {
        let library = VulkanLibrary::new().unwrap();
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                ..Default::default()
            },
        )
        .unwrap();
        let device_ext = DeviceExtensions {
            khr_storage_buffer_storage_class: true,
            ..DeviceExtensions::empty()
        };
        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|device| device.supported_extensions().contains(&device_ext))
            .filter_map(|device| {
                device
                    .queue_family_properties()
                    .iter()
                    .position(|q| q.queue_flags.intersects(QueueFlags::COMPUTE))
                    .map(|q| (device, q as u32))
            })
            .min_by_key(|(device, _)| match device.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .unwrap();

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_ext,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .unwrap();

        let queue = queues.next().unwrap();

        // === Creating allocators ===

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        // Initializing shader module

        let shader_module = unsafe {
            ShaderModule::new(device.clone(), ShaderModuleCreateInfo::new(shader)).unwrap()
        };

        let pipeline = {
            let cs = shader_module.entry_point(entry_point).unwrap();
            let stage = PipelineShaderStageCreateInfo::new(cs);
            let layout = PipelineLayout::new(
                device.clone(),
                PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
                    .into_pipeline_layout_create_info(device.clone())
                    .unwrap(),
            )
            .unwrap();
            let params = ComputePipelineCreateInfo::stage_layout(stage, layout);
            ComputePipeline::new(device.clone(), None, params).unwrap()
        };

        Self {
            device,
            queue,
            memory_allocator,
            descriptor_set_allocator,
            command_buffer_allocator,
            data_buffers: HashMap::new(),
            pipeline,
        }
    }
    pub fn get_device(&self) -> Arc<Device> {
        self.device.clone()
    }
    pub fn get_queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }
    pub fn execute(&self) {
        let layout = &self.pipeline.layout().set_layouts()[0];
        let set = PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            layout.clone(),
            self.data_buffers
                .clone()
                .into_iter()
                .map(|(binding, subbuffer)| WriteDescriptorSet::buffer(binding, subbuffer.clone())),
            [],
        )
        .unwrap();

        let mut cb = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();
        cb.bind_pipeline_compute(self.pipeline.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                self.pipeline.layout().clone(),
                0,
                set,
            )
            .unwrap();
        cb.dispatch([500, 1, 1]).unwrap();

        let cb = cb.build().unwrap();

        let future = sync::now(self.device.clone())
            .then_execute(self.queue.clone(), cb)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();
        future.wait(None).unwrap();
    }
}

impl<T: BufferContents> ComputeDevice<T> {
    pub fn add_buffer<I>(&mut self, binding: u32, iter: I)
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        self.data_buffers.insert(
            binding,
            Buffer::from_iter(
                self.memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::STORAGE_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                    ..Default::default()
                },
                iter,
            )
            .unwrap(),
        );
    }
}
impl<T: BufferContents + Clone> ComputeDevice<T> {
    pub fn read_buffer(&self, buffer_binding: u32) -> Vec<T> {
        self.data_buffers
            .get(&buffer_binding)
            .unwrap()
            .read()
            .unwrap()
            .deref()
            .to_vec()
    }
}
