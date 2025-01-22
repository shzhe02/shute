use crate::{
    device::Device,
    types::{Adapter, PowerPreference},
};

pub struct Instance {
    instance: wgpu::Instance,
}

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
    pub fn devices(&self) -> Vec<Adapter> {
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
        Device::from_adapter(adapter).await
    }
}
