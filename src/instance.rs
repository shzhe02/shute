use crate::{
    device::{Device, LimitType},
    types::{Adapter, PowerPreference},
};

/// Context for all other Shute objects.
///
/// This is the first thing you create when using Shute. Use it to get a `Device`,
/// which is used for basically everything else, through the `autoselect` or `devices` methods.
pub struct Instance {
    instance: wgpu::Instance,
}

impl Instance {
    /// Create a new instance of Shute.
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
    /// Get all available adapters (physical devices) on the system.
    /// After selecting an adapter, use the `Device::from_adapter` method to get a `Device`.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn devices(&self) -> Vec<Adapter> {
        self.instance.enumerate_adapters(wgpu::Backends::all())
    }
    /// Automatically select a device (GPU) based on a power preference.
    /// The `limit_type` parameter will denote how the limits of the device are set.
    /// See [LimitType] for more information about that.
    pub async fn autoselect(
        &self,
        power_preference: PowerPreference,
        limit_type: LimitType,
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
        Device::from_adapter(adapter, limit_type).await
    }
}

impl Default for Instance {
    fn default() -> Self {
        Self::new()
    }
}
