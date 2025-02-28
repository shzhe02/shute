/// Alias of [`wgpu::PowerPreference`](https://docs.rs/wgpu/latest/wgpu/enum.PowerPreference.html).
///
/// Power preference when autoselecting a device with `Instance::autoselect`.
pub type PowerPreference = wgpu::PowerPreference;
// TODO: would be great to get rid of this entirely.
/// Alias of [`wgpu::Adapter`](https://docs.rs/wgpu/latest/wgpu/struct.Adapter.html).
///
/// Use with `Device::from_adapter` to get a device.
pub type Adapter = wgpu::Adapter;
/// A compute shader module. Used in `Device::execute`.
pub struct ShaderModule {
    module: wgpu::ShaderModule,
    entry_point: String,
}
impl ShaderModule {
    /// Create a new shader module.
    ///
    /// Preferably, create shader modules using `Device::create_shader_module` and
    /// `Device::create_shader_module_with_workgroup_size` instead. This method is used there.
    pub(crate) fn new(module: wgpu::ShaderModule, entry_point: &str) -> Self {
        Self {
            module,
            entry_point: entry_point.to_string(),
        }
    }
    // TODO: think about if this is really needs to be accessible by
    // downstream crates.
    /// Get just the shader module (without entry point).
    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.module
    }
    /// Get the entry point of the compute shader.
    pub fn entry_point(&self) -> &String {
        &self.entry_point
    }
}

/// Limits for a device.
///
/// This is a trimmed-down version of
/// [`wgpu::Limits`](https://docs.rs/wgpu/latest/wgpu/struct.Limits.html)
/// with only compute-related parameters.
///
/// This can be used to, for example, dynamically set workgroup or dispatch dimensions
/// for optimization purposes.
#[derive(Debug)]
pub struct Limits {
    /// Amount of bind groups that can be attached to a pipeline at the same time.
    /// Defaults to 4. Higher is “better”.
    pub max_bind_groups: u32,
    /// Maximum binding index allowed in `create_bind_group_layout`.
    /// Defaults to 1000. Higher is “better”.
    pub max_bindings_per_bind_group: u32,
    /// Amount of uniform buffer bindings that can be dynamic in a single pipeline.
    /// Defaults to 8. Higher is “better”.
    pub max_dynamic_uniform_buffers_per_pipeline_layout: u32,
    /// Amount of storage buffer bindings that can be dynamic in a single pipeline.
    /// Defaults to 4. Higher is “better”.
    pub max_dynamic_storage_buffers_per_pipeline_layout: u32,
    /// Maximum size in bytes of a binding to a uniform buffer.
    /// Defaults to 64 KiB. Higher is “better”.
    pub max_uniform_buffer_binding_size: u32,
    /// Maximum size in bytes of a binding to a storage buffer.
    /// Defaults to 128 MiB. Higher is “better”.
    pub max_storage_buffer_binding_size: u32,
    /// A limit above which buffer allocations are guaranteed to fail.
    /// Defaults to 256 MiB. Higher is “better”.
    ///
    /// Buffer allocations below the maximum buffer size may not succeed
    /// depending on available memory, fragmentation, and other factors.
    pub max_buffer_size: u64,
    /// Maximum number of bytes used for workgroup memory in a compute entry point.
    /// Defaults to 16384. Higher is “better”.
    pub max_compute_workgroup_storage_size: u32,
    /// Maximum value of the product of the `workgroup_size` dimensions for a compute entry-point.
    /// Defaults to 256. Higher is “better”.
    pub max_compute_invocations_per_workgroup: u32,
    /// The maximum value of the workgroup_size X dimension for a
    /// compute stage `ShaderModule` entry-point.
    /// Defaults to 256. Higher is “better”.
    pub max_compute_workgroup_size_x: u32,
    /// The maximum value of the workgroup_size Y dimension for a
    /// compute stage `ShaderModule` entry-point.
    /// Defaults to 256. Higher is “better”.
    pub max_compute_workgroup_size_y: u32,
    /// The maximum value of the workgroup_size Z dimension for a
    /// compute stage `ShaderModule` entry-point.
    /// Defaults to 64. Higher is “better”.
    pub max_compute_workgroup_size_z: u32,
    /// The maximum value for each dimension of a `Device::execute(..., [x, y, z])` operation.
    /// Defaults to 65535. Higher is “better”.
    pub max_compute_workgroups_per_dimension: u32,
    /// Minimal number of invocations in a subgroup. Higher is “better”.
    pub min_subgroup_size: u32,
    /// Maximal number of invocations in a subgroup. Lower is “better”.
    pub max_subgroup_size: u32,
}

impl Limits {
    // TODO: convert this into a From<> trait implementation.
    pub(crate) fn from_wgpu_limits(limits: wgpu::Limits) -> Self {
        Self {
            max_bind_groups: limits.max_bind_groups,
            max_bindings_per_bind_group: limits.max_bindings_per_bind_group,
            max_dynamic_uniform_buffers_per_pipeline_layout: limits
                .max_dynamic_uniform_buffers_per_pipeline_layout,
            max_dynamic_storage_buffers_per_pipeline_layout: limits
                .max_dynamic_storage_buffers_per_pipeline_layout,
            max_uniform_buffer_binding_size: limits.max_uniform_buffer_binding_size,
            max_storage_buffer_binding_size: limits.max_storage_buffer_binding_size,
            max_buffer_size: limits.max_buffer_size,
            max_compute_workgroup_storage_size: limits.max_compute_workgroup_storage_size,
            max_compute_invocations_per_workgroup: limits.max_compute_invocations_per_workgroup,
            max_compute_workgroup_size_x: limits.max_compute_workgroup_size_x,
            max_compute_workgroup_size_y: limits.max_compute_workgroup_size_y,
            max_compute_workgroup_size_z: limits.max_compute_workgroup_size_z,
            max_compute_workgroups_per_dimension: limits.max_compute_workgroups_per_dimension,
            min_subgroup_size: limits.min_subgroup_size,
            max_subgroup_size: limits.max_subgroup_size,
        }
    }
}
