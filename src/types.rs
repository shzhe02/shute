pub type PowerPreference = wgpu::PowerPreference;
pub type Adapter = wgpu::Adapter;
pub struct ShaderModule {
    module: wgpu::ShaderModule,
    entry_point: String,
}
