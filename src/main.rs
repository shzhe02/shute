use std::{fs::read_to_string, ops::Deref, sync::Arc};

use naga::{
    back::spv::{self, Options},
    front::wgsl,
    valid::{Capabilities, ValidationFlags, Validator},
};
use shute::load_wgsl_shader;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo,
        QueueFlags,
    },
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        compute::ComputePipelineCreateInfo, layout::PipelineDescriptorSetLayoutCreateInfo,
        ComputePipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
    },
    shader::{ShaderModule, ShaderModuleCreateInfo},
    sync::{self, GpuFuture},
    VulkanLibrary,
};

fn main() {
    // === Turn wgsl shader into spirv with Naga ===
    let spv_out = load_wgsl_shader!("shaders/doubler.wgsl");
    // Starting vulkano implementation
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

    // let intermediate = instance
    //     .enumerate_physical_devices()
    //     .unwrap()
    //     .filter(|device| device.supported_extensions().contains(&device_ext))
    //     .filter_map(|device| {
    //         device
    //             .queue_family_properties()
    //             .iter()
    //             .position(|q| q.queue_flags.intersects(QueueFlags::COMPUTE))
    //             .map(|q| (device, q as u32))
    //     });
    // // dbg!(test);
    // let (physical_device, queue_family_index) = intermediate
    //     .min_by_key(|(device, _)| match device.properties().device_type {
    //         PhysicalDeviceType::DiscreteGpu => 0,
    //         PhysicalDeviceType::IntegratedGpu => 1,
    //         PhysicalDeviceType::VirtualGpu => 2,
    //         PhysicalDeviceType::Cpu => 3,
    //         PhysicalDeviceType::Other => 4,
    //         _ => 5,
    //     })
    //     .unwrap();
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
    let module = unsafe {
        ShaderModule::new(device.clone(), ShaderModuleCreateInfo::new(&spv_out))
            .expect("Failed to create shader module")
    };
    let pipeline = {
        let cs = module.entry_point("doubler").unwrap();
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

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
    let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
        device.clone(),
        Default::default(),
    ));
    let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
        device.clone(),
        Default::default(),
    ));

    let data_buffer_a = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_RANDOM_ACCESS,
            ..Default::default()
        },
        0..500u32,
    )
    .unwrap();
    let data_buffer_b = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_RANDOM_ACCESS,
            ..Default::default()
        },
        5..505u32,
    )
    .unwrap();
    let output_buffer = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::STORAGE_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_RANDOM_ACCESS,
            ..Default::default()
        },
        0..500u32,
    )
    .unwrap();

    let layout = &pipeline.layout().set_layouts()[0];
    let set = PersistentDescriptorSet::new(
        &descriptor_set_allocator,
        layout.clone(),
        [
            WriteDescriptorSet::buffer(0, data_buffer_a),
            WriteDescriptorSet::buffer(1, data_buffer_b),
            WriteDescriptorSet::buffer(2, output_buffer.clone()),
        ],
        [],
    )
    .unwrap();

    let mut cb = AutoCommandBufferBuilder::primary(
        &command_buffer_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();
    cb.bind_pipeline_compute(pipeline.clone())
        .unwrap()
        .bind_descriptor_sets(
            PipelineBindPoint::Compute,
            pipeline.layout().clone(),
            0,
            set,
        )
        .unwrap();
    cb.dispatch([500, 1, 1]).unwrap();

    let cb = cb.build().unwrap();

    let future = sync::now(device)
        .then_execute(queue, cb)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();
    future.wait(None).unwrap();
    let binding = output_buffer.read().unwrap();
    let output = binding.deref().to_vec();
    dbg!(output);
}
