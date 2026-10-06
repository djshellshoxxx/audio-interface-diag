#[cfg(feature = "native-audio")]
use cpal::traits::{DeviceTrait, HostTrait};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeDeviceSummary {
    pub host_api: String,
    pub name: String,
    pub is_default_input: bool,
    pub is_default_output: bool,
}

#[cfg(feature = "native-audio")]
pub fn enumerate_native_devices() -> Result<Vec<NativeDeviceSummary>, String> {
    let host = cpal::default_host();
    let host_api = format!("{:?}", host.id());
    let default_input = host
        .default_input_device()
        .and_then(|device| device.name().ok());
    let default_output = host
        .default_output_device()
        .and_then(|device| device.name().ok());
    let devices = host.devices().map_err(|error| error.to_string())?;

    let mut summaries = Vec::new();

    for device in devices {
        let name = device
            .name()
            .unwrap_or_else(|_| "<unavailable>".to_string());

        summaries.push(NativeDeviceSummary {
            host_api: host_api.clone(),
            is_default_input: default_input.as_deref() == Some(name.as_str()),
            is_default_output: default_output.as_deref() == Some(name.as_str()),
            name,
        });
    }

    Ok(summaries)
}

#[cfg(not(feature = "native-audio"))]
pub fn enumerate_native_devices() -> Result<Vec<NativeDeviceSummary>, String> {
    Err("native-audio feature is not enabled".into())
}
