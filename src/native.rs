#[cfg(feature = "native-audio")]
use cpal::traits::{DeviceTrait, HostTrait};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeDeviceSummary {
    pub host_api: String,
    pub name: String,
    pub is_default_input: bool,
    pub is_default_output: bool,
    /// Advertised ranges only. Successful stream opens are a separate test.
    pub input_formats: Result<Vec<NativeFormatRange>, String>,
    pub output_formats: Result<Vec<NativeFormatRange>, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeFormatRange {
    pub channels: u16,
    pub min_rate: u32,
    pub max_rate: u32,
    pub sample_format: String,
    /// None means the driver did not advertise a buffer range.
    pub buffer_frames: Option<(u32, u32)>,
}

#[cfg(feature = "native-audio")]
fn format_range(range: cpal::SupportedStreamConfigRange) -> NativeFormatRange {
    NativeFormatRange {
        channels: range.channels(),
        min_rate: range.min_sample_rate().0,
        max_rate: range.max_sample_rate().0,
        sample_format: format!("{:?}", range.sample_format()),
        buffer_frames: match range.buffer_size() {
            cpal::SupportedBufferSize::Range { min, max } => Some((*min, *max)),
            cpal::SupportedBufferSize::Unknown => None,
        },
    }
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
            input_formats: device
                .supported_input_configs()
                .map(|configs| configs.map(format_range).collect())
                .map_err(|error| error.to_string()),
            output_formats: device
                .supported_output_configs()
                .map(|configs| configs.map(format_range).collect())
                .map_err(|error| error.to_string()),
            name,
        });
    }

    Ok(summaries)
}

#[cfg(not(feature = "native-audio"))]
pub fn enumerate_native_devices() -> Result<Vec<NativeDeviceSummary>, String> {
    Err("native-audio feature is not enabled".into())
}

#[cfg(all(test, feature = "native-audio"))]
mod tests {
    use super::*;

    #[test]
    fn advertised_ranges_preserve_format_and_unknown_buffer() {
        for buffer in [
            cpal::SupportedBufferSize::Unknown,
            cpal::SupportedBufferSize::Range { min: 32, max: 4096 },
        ] {
            let range = cpal::SupportedStreamConfigRange::new(
                8,
                cpal::SampleRate(44_100),
                cpal::SampleRate(192_000),
                buffer,
                cpal::SampleFormat::F32,
            );
            let mapped = format_range(range);
            assert_eq!(mapped.channels, 8);
            assert_eq!((mapped.min_rate, mapped.max_rate), (44_100, 192_000));
            assert_eq!(mapped.sample_format, "F32");
            assert_eq!(
                mapped.buffer_frames,
                match buffer {
                    cpal::SupportedBufferSize::Unknown => None,
                    cpal::SupportedBufferSize::Range { min, max } => Some((min, max)),
                }
            );
        }
    }
}
