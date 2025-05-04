use crate::{
    device::{Device, DeviceError, LimitType},
    types::PowerPreference,
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
            instance: wgpu::Instance::new(&wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                flags: if cfg!(debug_assertions) {
                    wgpu::InstanceFlags::DEBUG
                        | wgpu::InstanceFlags::VALIDATION
                        | wgpu::InstanceFlags::GPU_BASED_VALIDATION
                } else {
                    wgpu::InstanceFlags::DISCARD_HAL_LABELS
                },
                backend_options: wgpu::BackendOptions::default(),
            }),
        }
    }
    /// Get all available devices on the system.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn devices(&self) -> Vec<Result<Device, DeviceError>> {
        self.instance
            .enumerate_adapters(wgpu::Backends::all())
            .into_iter()
            .filter(|adapter| adapter.get_info().device_type != wgpu::DeviceType::Other)
            .map(|adapter| pollster::block_on(Device::new(adapter, LimitType::Highest)))
            .collect()
    }
    /// Automatically select a device (like a GPU) based on a power preference.
    /// The `limit_type` parameter will denote how the limits of the device are set.
    /// See [LimitType] for more information about that.
    pub async fn autoselect(
        &self,
        power_preference: PowerPreference,
        limit_type: LimitType,
    ) -> Result<Device, DeviceError> {
        let adapter = self
            .instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .or_else(|_| Err(DeviceError::DeviceNotFound))?;
        Device::new(adapter, limit_type).await
    }
}

impl Default for Instance {
    fn default() -> Self {
        Self::new()
    }
}
