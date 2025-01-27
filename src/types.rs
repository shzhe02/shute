pub type PowerPreference = wgpu::PowerPreference;
pub type Adapter = wgpu::Adapter;
pub struct ShaderModule {
    module: wgpu::ShaderModule,
    entry_point: String,
}
impl ShaderModule {
    pub fn new(module: wgpu::ShaderModule, entry_point: &str) -> Self {
        Self {
            module,
            entry_point: entry_point.to_string(),
        }
    }
    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.module
    }
    pub fn entry_point(&self) -> &String {
        &self.entry_point
    }
}

#[derive(Debug)]
pub struct Limits {
    pub max_bind_groups: u32,
    pub max_bindings_per_bind_group: u32,
    pub max_dynamic_uniform_buffers_per_pipeline_layout: u32,
    pub max_dynamic_storage_buffers_per_pipeline_layout: u32,
    pub max_uniform_buffer_binding_size: u32,
    pub max_storage_buffer_binding_size: u32,
    pub max_buffer_size: u64,
    pub max_compute_workgroup_storage_size: u32,
    pub max_compute_invocations_per_workgroup: u32,
    pub max_compute_workgroup_size_x: u32,
    pub max_compute_workgroup_size_y: u32,
    pub max_compute_workgroup_size_z: u32,
    pub max_compute_workgroups_per_dimension: u32,
    pub min_subgroup_size: u32,
    pub max_subgroup_size: u32,
}

impl Limits {
    pub fn from_wgpu_limits(limits: wgpu::Limits) -> Self {
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
