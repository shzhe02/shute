pub type PowerPreference = wgpu::PowerPreference;
pub type Adapter = wgpu::Adapter;
pub struct ShaderModule {
    module: wgpu::ShaderModule,
    entry_point: String,
}
impl ShaderModule {
    pub fn new(module: wgpu::ShaderModule, entry_point: String) -> Self {
        Self {
            module,
            entry_point,
        }
    }
    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.module
    }
    pub fn entry_point(&self) -> &String {
        &self.entry_point
    }
}

pub trait ShaderType: encase::ShaderType + encase::internal::WriteInto {}
