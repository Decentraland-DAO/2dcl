use std::path::PathBuf;

use bevy::asset::{Assets, Handle};
use bevy::ecs::component::Component;
use bevy::ecs::prelude::Events;
use bevy::ecs::system::{Commands, Res, ResMut};
use bevy::render::texture::Image;
use bevy::render::texture::TextureFormatPixelInfo;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use image::ImageFormat;

use crate::data::{ActiveRecorders, Alignment, CaptureFrame, HasTaskStatus};
use crate::image_utils::frame_data_to_rgba_image;
use crate::plugin::CaptureState;
#[cfg(target_arch = "wasm32")]
use crate::web_utils;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub enum SavePng {
    #[default]
    Basic,
    Watermarked {
        watermark: Handle<Image>,
        alignment: Alignment,
    },
}

pub type SavePngFile = CaptureFrame<SavePng>;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Component)]
pub struct SaveFrameTask(pub Task<()>);

#[cfg(not(target_arch = "wasm32"))]
impl HasTaskStatus for SaveFrameTask {
    fn is_done(&mut self) -> bool {
        let result = future::block_on(future::poll_once(&mut self.0));
        result.is_some()
    }
}

pub fn is_ready_to_export(recorders: Res<ActiveRecorders>, total_frames: usize) -> bool {
    for (_, recorder) in recorders.iter() {
        if recorder.frames.len() >= total_frames {
            let texture = &recorder.frames[recorder.frames.len() - total_frames].texture;

            for i in 0..texture.len() {
                if (i + 1) % 4 == 0
          //Not empty
          && !alpha_is_zero(texture, i)

          //Not Cyan (chroma color)
          && !is_cyan(texture, i)

          //Not plain blank
          //R
          && !is_blank(texture, i)
                {
                    return true;
                }
            }
        }
    }

    return false;

    fn alpha_is_zero(texture: &Vec<u8>, index: usize) -> bool {
        if texture.len() > index && (index + 1) % 4 == 0 {
            return texture[index] == 0;
        }
        false
    }

    fn is_cyan(texture: &Vec<u8>, index: usize) -> bool {
        if texture.len() > index && (index + 1) % 4 == 0 {
            return texture[index - 3] == 0
                && texture[index - 2] == 255
                && texture[index - 1] == 255;
        }
        false
    }

    fn is_blank(texture: &Vec<u8>, index: usize) -> bool {
        if texture.len() > index && (index + 1) % 4 == 0 {
            return texture[index - 3] == 255
                && texture[index - 2] == 255
                && texture[index - 1] == 255;
        }
        false
    }
}

pub fn save_single_frame(
    mut commands: Commands,
    mut events: ResMut<Events<SavePngFile>>,
    recorders: ResMut<ActiveRecorders>,
    images: Res<Assets<Image>>,
    mut state: ResMut<CaptureState>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    'event_drain: for event in events.drain() {
        if let Some(recorder) = recorders.get(&event.tracking_id) {
            let mut data: Vec<u8> = Vec::default();
            for i in recorder.frames.len() - event.total_frames..recorder.frames.len() {
                if i < recorder.frames.len() {
                    for e in 0..recorder.frames[i].texture.len() {
                        if (e + 1) % 4 == 0
                          //R
                          && recorder.frames[i].texture[e - 3] == 0
                          //G
                          && recorder.frames[i].texture[e - 2] == 255
                          //B
                          && recorder.frames[i].texture[e - 1] == 255
                        {
                            data.push(0);
                            continue;
                        }
                        data.push(recorder.frames[i].texture[e]);
                    }
                } else {
                    todo!(); // blank
                }
            }

            let (width, height, target_format) = match images.get(&recorder.target_handle) {
                Some(image) => (
                    image.size().x as u32,
                    image.size().y as u32 * event.total_frames as u32,
                    image.texture_descriptor.format,
                ),
                None => continue 'event_drain,
            };

            *state = CaptureState::TakingScreenshot;
            let task = thread_pool.spawn(async move {
                let data = data;
                let format = target_format;

                let expected_size = width * height * format.pixel_size() as u32;
                if expected_size != data.len() as u32 {
                    log::error!("Failed to assert that the data frame is correctly formatted");
                    return;
                }

                let image = frame_data_to_rgba_image(width, height, data, format);

                // if let SavePng::Watermarked { watermark } = event {
                // 	let watermark_image = watermark
                // }

                #[cfg(not(target_arch = "wasm32"))]
                {
                    let file_name = event.path.unwrap_or_else(|| {
                        PathBuf::from(format!(
                            "{}.png",
                            std::time::UNIX_EPOCH.elapsed().unwrap().as_secs()
                        ))
                    });

                    println!("saving: {:?}", file_name);
                    if let Err(e) = image.save_with_format(file_name, ImageFormat::Png) {
                        log::error!("Failed to write screenshot: {}", e);
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    let file_name = event
                        .path
                        .and_then(|path| {
                            path.file_name()
                                .and_then(|name| name.to_str())
                                .map(|name| PathBuf::from(name))
                        })
                        .unwrap_or_else(|| {
                            PathBuf::from(format!("{}.png", crate::web_utils::get_now()))
                        });

                    log::info!("Image size: {}", image.len());

                    let mut file_bytes = Cursor::new(vec![0; image.len()]);
                    image.write_to(&mut file_bytes, image::ImageFormat::Png);

                    crate::web_utils::download_bytes(file_name, file_bytes.into_inner())
                }
            });

            #[cfg(target_arch = "wasm32")]
            task.detach();
            #[cfg(not(target_arch = "wasm32"))]
            commands.spawn(SaveFrameTask(task));
        }
    }
}
