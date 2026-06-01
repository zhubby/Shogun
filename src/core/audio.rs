use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;

use super::settings::{AudioSettings, normalize_output_device_name};
use super::state::Screen;

const MAIN_MENU_BGM_BYTES: &[u8] = include_bytes!("../../assets/audio/bgm.mp3");

#[derive(Default)]
pub(super) struct MainMenuAudio {
    stream: Option<OutputStream>,
    sink: Option<Sink>,
    requested_device_name: Option<String>,
}

impl MainMenuAudio {
    pub(super) fn sync(
        &mut self,
        screen: Screen,
        bgm_enabled: bool,
        audio_settings: &AudioSettings,
    ) -> Result<Option<String>, String> {
        let settings = audio_settings.clone().normalized();
        let should_play = screen == Screen::MainMenu && bgm_enabled;
        if !should_play {
            self.stop();
            return Ok(None);
        }

        let device_changed = self.requested_device_name != settings.output_device_name;
        if let Some(sink) = &self.sink
            && !device_changed
        {
            sink.set_volume(settings.master_volume);
            return Ok(None);
        }

        self.stop();
        self.start(&settings)
    }

    pub(super) fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
        self.stream = None;
        self.requested_device_name = None;
    }

    fn start(&mut self, settings: &AudioSettings) -> Result<Option<String>, String> {
        let (stream, handle, warning) = output_stream_for(settings.output_device_name.as_deref())?;
        let sink =
            Sink::try_new(&handle).map_err(|error| format!("创建音频播放器失败: {error}"))?;
        let source = Decoder::new_looped(Cursor::new(MAIN_MENU_BGM_BYTES.to_vec()))
            .map_err(|error| format!("解码主菜单背景音乐失败: {error}"))?;
        sink.set_volume(settings.master_volume);
        sink.append(source);
        sink.play();

        self.stream = Some(stream);
        self.sink = Some(sink);
        self.requested_device_name = settings.output_device_name.clone();
        Ok(warning)
    }
}

pub(super) fn available_output_device_names() -> Result<Vec<String>, String> {
    let host = rodio::cpal::default_host();
    let devices = host
        .output_devices()
        .map_err(|error| format!("读取输出设备失败: {error}"))?;
    let mut names = Vec::new();
    for device in devices {
        let Ok(name) = device.name() else {
            continue;
        };
        if !name.trim().is_empty() && !names.contains(&name) {
            names.push(name);
        }
    }
    Ok(names)
}

#[cfg(test)]
pub(super) fn selected_output_device_index(
    device_names: &[String],
    desired_name: Option<&str>,
) -> Option<usize> {
    let desired_name = normalize_output_device_name(desired_name.map(str::to_string))?;
    device_names.iter().position(|name| name == &desired_name)
}

fn output_stream_for(
    device_name: Option<&str>,
) -> Result<(OutputStream, rodio::OutputStreamHandle, Option<String>), String> {
    let mut warning = None;
    if let Some(device_name) = normalize_output_device_name(device_name.map(str::to_string)) {
        match output_device_by_name(&device_name) {
            Ok(Some(device)) => match OutputStream::try_from_device(&device) {
                Ok((stream, handle)) => return Ok((stream, handle, None)),
                Err(error) => {
                    warning = Some(format!(
                        "输出设备 {device_name} 无法播放，已切回系统默认: {error}"
                    ));
                }
            },
            Ok(None) => {
                warning = Some(format!("找不到输出设备 {device_name}，已切回系统默认"));
            }
            Err(error) => {
                warning = Some(format!("读取输出设备失败，已切回系统默认: {error}"));
            }
        }
    }

    let (stream, handle) = OutputStream::try_default()
        .map_err(|error| format!("打开系统默认输出设备失败: {error}"))?;
    Ok((stream, handle, warning))
}

fn output_device_by_name(device_name: &str) -> Result<Option<rodio::Device>, String> {
    let host = rodio::cpal::default_host();
    let devices = host
        .output_devices()
        .map_err(|error| format!("读取输出设备失败: {error}"))?;
    for device in devices {
        let Ok(name) = device.name() else {
            continue;
        };
        if name == device_name {
            return Ok(Some(device));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_output_device_index_matches_requested_name() {
        let names = vec![
            "System Speakers".to_string(),
            "External DAC".to_string(),
            "HDMI".to_string(),
        ];

        assert_eq!(
            selected_output_device_index(&names, Some("External DAC")),
            Some(1)
        );
        assert_eq!(selected_output_device_index(&names, Some("Missing")), None);
        assert_eq!(selected_output_device_index(&names, None), None);
        assert_eq!(selected_output_device_index(&names, Some("  ")), None);
    }
}
