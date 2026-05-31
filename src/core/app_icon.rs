use bevy::{app::Plugin, prelude::App};

#[cfg(any(test, not(target_os = "macos")))]
use bevy::{
    asset::RenderAssetUsages,
    image::{CompressedImageFormats, Image, ImageFormat, ImageSampler, ImageType},
    render::render_resource::TextureFormat,
};
#[cfg(not(target_os = "macos"))]
use bevy::{log::warn, prelude::MessageReader, window::WindowCreated, winit::WINIT_WINDOWS};
#[cfg(not(target_os = "macos"))]
use std::sync::OnceLock;
#[cfg(not(target_os = "macos"))]
use winit::window::Icon;

#[cfg(not(target_os = "macos"))]
const RUNTIME_ICON_BYTES: &[u8] = include_bytes!("../../assets/icons/icon.png");

#[cfg(test)]
const MACOS_ICON_BYTES: &[u8] = include_bytes!("../../assets/icons/macos.png");

pub(super) struct AppIconPlugin;

impl Plugin for AppIconPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_os = "macos"))]
        app.add_systems(bevy::prelude::Update, set_window_icon_on_created);
        #[cfg(target_os = "macos")]
        let _ = app;
    }
}

#[cfg(any(test, not(target_os = "macos")))]
#[derive(Clone, Debug, PartialEq, Eq)]
struct DecodedIcon {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
}

#[cfg(any(test, not(target_os = "macos")))]
fn decode_icon_bytes(bytes: &[u8]) -> Result<DecodedIcon, String> {
    let image = Image::from_buffer(
        bytes,
        ImageType::Format(ImageFormat::Png),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::Default,
        RenderAssetUsages::MAIN_WORLD,
    )
    .map_err(|error| format!("failed to decode PNG icon: {error}"))?;

    let width = image.width();
    let height = image.height();
    let format = image.texture_descriptor.format;
    let data = image
        .data
        .ok_or_else(|| "decoded icon has no CPU pixel data".to_string())?;

    let rgba = match format {
        TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Uint => {
            data
        }
        TextureFormat::Rgba16Unorm | TextureFormat::Rgba16Uint => rgba16_to_rgba8(&data)?,
        _ => {
            return Err(format!(
                "unsupported decoded icon texture format: {format:?}"
            ));
        }
    };

    let expected_len = width as usize * height as usize * 4;
    if rgba.len() != expected_len {
        return Err(format!(
            "decoded icon size mismatch: expected {expected_len} RGBA bytes, got {}",
            rgba.len()
        ));
    }

    Ok(DecodedIcon {
        rgba,
        width,
        height,
    })
}

#[cfg(any(test, not(target_os = "macos")))]
fn rgba16_to_rgba8(bytes: &[u8]) -> Result<Vec<u8>, String> {
    if !bytes.len().is_multiple_of(8) {
        return Err(format!(
            "16-bit RGBA icon buffer length must be divisible by 8, got {}",
            bytes.len()
        ));
    }

    let mut rgba = Vec::with_capacity(bytes.len() / 2);
    for channel in bytes.chunks_exact(2) {
        let value = u16::from_le_bytes([channel[0], channel[1]]);
        rgba.push((value / 257) as u8);
    }
    Ok(rgba)
}

#[cfg(not(target_os = "macos"))]
fn set_window_icon_on_created(mut window_created: MessageReader<WindowCreated>) {
    for event in window_created.read() {
        let icon = match runtime_icon() {
            Ok(icon) => icon,
            Err(error) => {
                warn!("Unable to load application icon: {error}");
                return;
            }
        };

        WINIT_WINDOWS.with_borrow(|windows| {
            let Some(window) = windows.get_window(event.window) else {
                warn!(
                    "Unable to set application icon: window {:?} was not found",
                    event.window
                );
                return;
            };
            window.set_window_icon(Some(icon));
        });
    }
}

#[cfg(not(target_os = "macos"))]
fn runtime_icon() -> Result<Icon, String> {
    static DECODED_ICON: OnceLock<Result<DecodedIcon, String>> = OnceLock::new();

    let decoded = DECODED_ICON
        .get_or_init(|| decode_icon_bytes(RUNTIME_ICON_BYTES))
        .as_ref()
        .map_err(Clone::clone)?;

    Icon::from_rgba(decoded.rgba.clone(), decoded.width, decoded.height)
        .map_err(|error| format!("failed to build window icon: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_icon_source_decodes_to_rgba8() {
        assert_icon_decodes(include_bytes!("../../assets/icons/icon.png"));
    }

    #[test]
    fn macos_icon_source_decodes_to_rgba8() {
        assert_icon_decodes(MACOS_ICON_BYTES);
    }

    fn assert_icon_decodes(bytes: &[u8]) {
        let decoded = decode_icon_bytes(bytes).unwrap();

        assert!(decoded.width > 0);
        assert!(decoded.height > 0);
        assert_eq!(
            decoded.rgba.len(),
            decoded.width as usize * decoded.height as usize * 4
        );
    }
}
