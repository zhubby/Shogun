use crate::ai::{
    AiApiError, BAILIAN_DEFAULT_IMAGE_MODEL, BailianClient, BailianConfig,
    BailianImageGenerationRequest, BailianImageGenerationResponse, BailianImageParameters,
    BailianRegion, BailianWaitOptions,
};
use crate::game::{OfficerGender, OfficerId};
use bevy::{image::Image as BevyImage, render::render_resource::TextureFormat};
use bevy_egui::egui;
use directories::ProjectDirs;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread,
    time::{Duration, SystemTime},
};

use super::state::OfficerEditDraft;

pub(super) const OFFICER_PORTRAIT_PROMPT_TEMPLATE: &str = "\
A portrait of [人名], [身份描述], in Three Kingdoms historical strategy game character illustration style,
semi-realistic anime aesthetic, dramatic cinematic lighting, rim light,
dark atmospheric background, highly detailed digital painting, masterpiece";
pub(super) const OFFICER_PORTRAIT_SIZE: &str = "576*768";

const OFFICER_PORTRAIT_PNG_MAGIC: &[u8] = b"\x89PNG\r\n\x1a\n";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum OfficerPortraitTaskState {
    Idle,
    Generating,
    Succeeded { path: PathBuf },
    Failed(String),
}

pub(super) struct OfficerPortraitStore {
    task_states: BTreeMap<OfficerId, OfficerPortraitTaskState>,
    textures: BTreeMap<OfficerId, OfficerPortraitTexture>,
    sender: Sender<OfficerPortraitTaskEvent>,
    receiver: Mutex<Receiver<OfficerPortraitTaskEvent>>,
}

impl Default for OfficerPortraitStore {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            task_states: BTreeMap::new(),
            textures: BTreeMap::new(),
            sender,
            receiver: Mutex::new(receiver),
        }
    }
}

impl OfficerPortraitStore {
    pub(super) fn poll_task_events(&mut self) {
        let events = self
            .receiver
            .lock()
            .map(|receiver| receiver.try_iter().collect::<Vec<_>>())
            .unwrap_or_default();

        for event in events {
            match event {
                OfficerPortraitTaskEvent::Succeeded { officer_id, path } => {
                    self.textures.remove(&officer_id);
                    self.task_states
                        .insert(officer_id, OfficerPortraitTaskState::Succeeded { path });
                }
                OfficerPortraitTaskEvent::Failed { officer_id, error } => {
                    self.task_states
                        .insert(officer_id, OfficerPortraitTaskState::Failed(error));
                }
            }
        }
    }

    pub(super) fn task_state(&self, officer_id: &str) -> OfficerPortraitTaskState {
        self.task_states
            .get(officer_id)
            .cloned()
            .unwrap_or(OfficerPortraitTaskState::Idle)
    }

    pub(super) fn start_generation(
        &mut self,
        draft: OfficerEditDraft,
        api_key: String,
        model_name: String,
        missing_api_key_message: String,
    ) {
        let officer_id = draft.id.clone();
        if matches!(
            self.task_states.get(&officer_id),
            Some(OfficerPortraitTaskState::Generating)
        ) {
            return;
        }

        if api_key.trim().is_empty() {
            self.task_states.insert(
                officer_id,
                OfficerPortraitTaskState::Failed(missing_api_key_message),
            );
            return;
        }

        let path = match officer_portrait_path(&officer_id) {
            Ok(path) => path,
            Err(error) => {
                self.task_states
                    .insert(officer_id, OfficerPortraitTaskState::Failed(error));
                return;
            }
        };

        self.task_states
            .insert(officer_id.clone(), OfficerPortraitTaskState::Generating);

        let sender = self.sender.clone();
        thread::spawn(move || {
            let result = run_officer_portrait_generation(api_key, model_name, draft, path.clone());
            let event = match result {
                Ok(()) => OfficerPortraitTaskEvent::Succeeded { officer_id, path },
                Err(error) => OfficerPortraitTaskEvent::Failed { officer_id, error },
            };
            let _ = sender.send(event);
        });
    }

    pub(super) fn texture_for(
        &mut self,
        ctx: &egui::Context,
        officer_id: &str,
        path: &Path,
    ) -> Result<Option<OfficerPortraitTextureView>, String> {
        if !path.is_file() {
            self.textures.remove(officer_id);
            return Ok(None);
        }

        let modified = fs::metadata(path)
            .ok()
            .and_then(|metadata| metadata.modified().ok());
        if let Some(texture) = self.textures.get(officer_id)
            && texture.modified == modified
        {
            return Ok(Some(texture.view()));
        }

        let texture = load_portrait_texture(ctx, officer_id, path, modified)?;
        let view = texture.view();
        self.textures.insert(officer_id.to_string(), texture);
        Ok(Some(view))
    }
}

struct OfficerPortraitTexture {
    texture: egui::TextureHandle,
    image_size: egui::Vec2,
    modified: Option<SystemTime>,
}

impl OfficerPortraitTexture {
    fn view(&self) -> OfficerPortraitTextureView {
        OfficerPortraitTextureView {
            texture_id: self.texture.id(),
            image_size: self.image_size,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct OfficerPortraitTextureView {
    pub(super) texture_id: egui::TextureId,
    pub(super) image_size: egui::Vec2,
}

enum OfficerPortraitTaskEvent {
    Succeeded {
        officer_id: OfficerId,
        path: PathBuf,
    },
    Failed {
        officer_id: OfficerId,
        error: String,
    },
}

pub(super) fn officer_portrait_path(officer_id: &str) -> Result<PathBuf, String> {
    officer_portrait_path_in_data_dir(default_game_data_dir(), officer_id)
}

pub(super) fn officer_portrait_path_in_data_dir(
    data_dir: impl AsRef<Path>,
    officer_id: &str,
) -> Result<PathBuf, String> {
    validate_officer_portrait_id(officer_id)?;
    Ok(data_dir
        .as_ref()
        .join("portraits")
        .join("officers")
        .join(format!("{officer_id}.png")))
}

pub(super) fn build_officer_portrait_prompt(draft: &OfficerEditDraft) -> String {
    let name = draft.name.trim();
    let name = if name.is_empty() {
        draft.id.as_str()
    } else {
        name
    };
    let identity = officer_identity_description(draft);
    let base = OFFICER_PORTRAIT_PROMPT_TEMPLATE
        .replace("[人名]", name)
        .replace("[身份描述]", &identity);
    let mut details = vec![
        format!("Name: {name}"),
        format!("Gender: {}", officer_gender_prompt_label(&draft.gender)),
        format!(
            "Abilities: leadership {}, strength {}, intelligence {}, politics {}, charm {}",
            draft.leadership, draft.strength, draft.intelligence, draft.politics, draft.charm
        ),
    ];
    push_prompt_detail(&mut details, "Courtesy name", &draft.courtesy_name);
    push_prompt_detail(&mut details, "Native place", &draft.native_place);
    push_prompt_detail(&mut details, "Birth year", &draft.birth_year);
    push_prompt_detail(&mut details, "Death year", &draft.death_year);
    push_prompt_detail(&mut details, "Tags", &draft.tags);
    let biography = truncate_prompt_text(draft.biography.trim(), 260);
    push_prompt_detail(&mut details, "Biography", &biography);

    format!("{base}\n\nCharacter reference: {}.", details.join("; "))
}

fn run_officer_portrait_generation(
    api_key: String,
    model_name: String,
    draft: OfficerEditDraft,
    path: PathBuf,
) -> Result<(), String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("初始化图像生成运行时失败: {error}"))?;
    runtime.block_on(generate_officer_portrait(api_key, model_name, draft, path))
}

async fn generate_officer_portrait(
    api_key: String,
    model_name: String,
    draft: OfficerEditDraft,
    path: PathBuf,
) -> Result<(), String> {
    let model_name = normalized_model_name(&model_name);
    let client = BailianClient::new(
        BailianConfig::new(api_key, BailianRegion::Beijing).with_timeout(Duration::from_secs(180)),
    )
    .map_err(localize_ai_error)?;
    let mut request = BailianImageGenerationRequest::text_to_image(
        model_name,
        build_officer_portrait_prompt(&draft),
    );
    request.parameters = Some(BailianImageParameters {
        size: Some(OFFICER_PORTRAIT_SIZE.to_string()),
        n: Some(1),
        watermark: Some(false),
        ..Default::default()
    });

    let created = client
        .create_image_task(&request)
        .await
        .map_err(localize_ai_error)?;
    let completed = if created.is_finished() {
        created
    } else {
        let task_id = created
            .output
            .as_ref()
            .and_then(|output| output.task_id.as_deref())
            .ok_or_else(|| image_task_error(&created, "百炼未返回图像任务 ID"))?;
        client
            .wait_image_task(task_id, BailianWaitOptions::default())
            .await
            .map_err(localize_ai_error)?
    };
    let image_url = first_output_image_url(&completed)
        .ok_or_else(|| image_task_error(&completed, "百炼任务完成但未返回图片"))?;
    download_png_to_path(&image_url, &path).await
}

async fn download_png_to_path(image_url: &str, path: &Path) -> Result<(), String> {
    let response = reqwest::Client::builder()
        .timeout(Duration::from_secs(180))
        .build()
        .map_err(|error| format!("初始化图片下载客户端失败: {error}"))?
        .get(image_url)
        .send()
        .await
        .map_err(|error| format!("下载肖像图片失败: {error}"))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("下载肖像图片失败: HTTP {status}"));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|error| format!("读取肖像图片失败: {error}"))?;
    if !bytes.starts_with(OFFICER_PORTRAIT_PNG_MAGIC) {
        return Err("百炼返回的肖像不是 PNG，已拒绝覆盖本地 PNG 文件。".to_string());
    }

    let Some(parent) = path.parent() else {
        return Err("肖像保存路径无效。".to_string());
    };
    fs::create_dir_all(parent).map_err(|error| format!("创建肖像目录失败: {error}"))?;
    let tmp_path = path.with_extension("png.tmp");
    fs::write(&tmp_path, &bytes).map_err(|error| format!("写入临时肖像文件失败: {error}"))?;
    fs::rename(&tmp_path, path).map_err(|error| format!("保存肖像文件失败: {error}"))?;
    Ok(())
}

fn first_output_image_url(response: &BailianImageGenerationResponse) -> Option<String> {
    response
        .output
        .as_ref()?
        .choices
        .iter()
        .flat_map(|choice| choice.message.content.iter())
        .find_map(|content| content.image.clone())
}

fn image_task_error(response: &BailianImageGenerationResponse, fallback: &str) -> String {
    response
        .message
        .clone()
        .or_else(|| response.code.clone())
        .or_else(|| {
            response
                .output
                .as_ref()
                .and_then(|output| output.task_status.clone())
        })
        .unwrap_or_else(|| fallback.to_string())
}

fn localize_ai_error(error: AiApiError) -> String {
    error.to_string()
}

fn load_portrait_texture(
    ctx: &egui::Context,
    officer_id: &str,
    path: &Path,
    modified: Option<SystemTime>,
) -> Result<OfficerPortraitTexture, String> {
    let bytes = fs::read(path).map_err(|error| format!("读取肖像文件失败: {error}"))?;
    let image = BevyImage::from_buffer(
        &bytes,
        bevy::image::ImageType::Format(bevy::image::ImageFormat::Png),
        bevy::image::CompressedImageFormats::NONE,
        true,
        bevy::image::ImageSampler::Default,
        bevy::asset::RenderAssetUsages::MAIN_WORLD,
    )
    .map_err(|error| format!("解析肖像 PNG 失败: {error}"))?;
    let color_image = color_image_from_bevy_image(&image)?;
    let image_size = egui::vec2(color_image.size[0] as f32, color_image.size[1] as f32);
    let texture = ctx.load_texture(
        format!("officer_portrait_{officer_id}"),
        color_image,
        egui::TextureOptions::LINEAR,
    );

    Ok(OfficerPortraitTexture {
        texture,
        image_size,
        modified,
    })
}

fn color_image_from_bevy_image(image: &BevyImage) -> Result<egui::ColorImage, String> {
    let width = image.width() as usize;
    let height = image.height() as usize;
    let format = image.texture_descriptor.format;
    let data = image
        .data
        .as_ref()
        .ok_or_else(|| "解析后的肖像没有 CPU 像素数据".to_string())?;
    let rgba = match format {
        TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Uint => {
            data.clone()
        }
        TextureFormat::Rgba16Unorm | TextureFormat::Rgba16Uint => rgba16_to_rgba8(data)?,
        _ => return Err(format!("不支持的肖像纹理格式: {format:?}")),
    };

    Ok(egui::ColorImage::from_rgba_unmultiplied(
        [width, height],
        &rgba,
    ))
}

fn rgba16_to_rgba8(bytes: &[u8]) -> Result<Vec<u8>, String> {
    if !bytes.len().is_multiple_of(8) {
        return Err(format!(
            "16-bit RGBA 肖像缓冲区长度必须能被 8 整除，当前为 {}",
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

fn default_game_data_dir() -> PathBuf {
    ProjectDirs::from("", "", "Shogun")
        .map(|dirs| dirs.data_local_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from(".shogun_data"))
}

fn validate_officer_portrait_id(officer_id: &str) -> Result<(), String> {
    let valid = !officer_id.is_empty()
        && officer_id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-');
    if valid {
        Ok(())
    } else {
        Err(format!("非法武将 ID，无法作为肖像文件名: {officer_id}"))
    }
}

fn normalized_model_name(model_name: &str) -> String {
    let model_name = model_name.trim();
    if model_name.is_empty() {
        BAILIAN_DEFAULT_IMAGE_MODEL.to_string()
    } else {
        model_name.to_string()
    }
}

fn officer_identity_description(draft: &OfficerEditDraft) -> String {
    let mut parts = vec![format!(
        "{} officer of the late Han and Three Kingdoms era",
        officer_gender_prompt_label(&draft.gender)
    )];
    push_prompt_detail(&mut parts, "courtesy name", &draft.courtesy_name);
    push_prompt_detail(&mut parts, "native place", &draft.native_place);
    if !draft.tags.trim().is_empty() {
        parts.push(format!("historical tags {}", draft.tags.trim()));
    }
    parts.join(", ")
}

fn officer_gender_prompt_label(gender: &OfficerGender) -> &'static str {
    match gender {
        OfficerGender::Male => "male",
        OfficerGender::Female => "female",
    }
}

fn push_prompt_detail(parts: &mut Vec<String>, label: &str, value: &str) {
    let value = value.trim();
    if !value.is_empty() {
        parts.push(format!("{label}: {value}"));
    }
}

fn truncate_prompt_text(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::SourceConfidence;

    fn test_draft() -> OfficerEditDraft {
        OfficerEditDraft {
            id: "liu_bei".to_string(),
            name: "刘备".to_string(),
            courtesy_name: "玄德".to_string(),
            native_place: "涿郡涿县".to_string(),
            birth_year: "161".to_string(),
            death_year: "223".to_string(),
            gender: OfficerGender::Male,
            leadership: 76,
            strength: 72,
            intelligence: 78,
            politics: 80,
            charm: 99,
            tags: "ruler,shu_han".to_string(),
            confidence: SourceConfidence::High,
            biography: "汉昭烈帝，蜀汉开国君主。".to_string(),
            notes: String::new(),
        }
    }

    #[test]
    fn portrait_path_uses_stable_officer_id_under_data_dir() {
        let path =
            officer_portrait_path_in_data_dir(Path::new("/tmp/shogun-data"), "liu_bei").unwrap();

        assert_eq!(
            path,
            PathBuf::from("/tmp/shogun-data/portraits/officers/liu_bei.png")
        );
    }

    #[test]
    fn portrait_path_rejects_path_separators() {
        assert!(
            officer_portrait_path_in_data_dir(Path::new("/tmp/shogun-data"), "../liu").is_err()
        );
        assert!(
            officer_portrait_path_in_data_dir(Path::new("/tmp/shogun-data"), "liu/bei").is_err()
        );
    }

    #[test]
    fn portrait_prompt_replaces_name_and_identity_description() {
        let prompt = build_officer_portrait_prompt(&test_draft());

        assert!(prompt.contains("A portrait of 刘备"));
        assert!(prompt.contains("male officer of the late Han and Three Kingdoms era"));
        assert!(prompt.contains("courtesy name: 玄德"));
        assert!(
            prompt.contains("Three Kingdoms historical strategy game character illustration style")
        );
        assert!(prompt.contains("Abilities: leadership 76"));
        assert!(!prompt.contains("[人名]"));
        assert!(!prompt.contains("[身份描述]"));
    }

    #[test]
    fn portrait_request_uses_fixed_size_single_png_generation_parameters() {
        let mut request = BailianImageGenerationRequest::text_to_image(
            normalized_model_name(""),
            build_officer_portrait_prompt(&test_draft()),
        );
        request.parameters = Some(BailianImageParameters {
            size: Some(OFFICER_PORTRAIT_SIZE.to_string()),
            n: Some(1),
            watermark: Some(false),
            ..Default::default()
        });

        let value = serde_json::to_value(&request).unwrap();

        assert_eq!(value["model"], BAILIAN_DEFAULT_IMAGE_MODEL);
        assert_eq!(value["parameters"]["size"], "576*768");
        assert_eq!(value["parameters"]["n"], 1);
        assert_eq!(value["parameters"]["watermark"], false);
    }
}
