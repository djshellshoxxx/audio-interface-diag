#[cfg(feature = "native-audio")]
use cpal::traits::{DeviceTrait, HostTrait};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeDeviceSummary {
    pub host_api: String,
    pub name: String,
    pub default_input: Option<String>,
    pub default_output: Option<String>,
}

#[cfg(feature = "native-audio")]
pub fn enumerate_native_devices() -> Result<Vec<NativeDeviceSummary>, String> {
    let host = cpal::default_host();
    let host_api = format!("{:?}", host.id());
    let default_input = host.default_input_device().and_then(|d| d.name().ok());
    let default_output = host.default_output_device().and_then(|d| d.name().ok());
    let devices = host.devices().map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for d in devices {
        let name = d.name().unwrap_or_else(|_| "<unavailable>".into());
        out.push(NativeDeviceSummary {
            host_api: host_api.clone(),
            name,
            default_input: default_input.clone(),
            default_output: default_output.clone(),
        });
    }
    Ok(out)
}

#[cfg(not(feature = "native-audio"))]
pub fn enumerate_native_devices() -> Result<Vec<NativeDeviceSummary>, String> {
    Err("native-audio feature is not enabled".into())
}
