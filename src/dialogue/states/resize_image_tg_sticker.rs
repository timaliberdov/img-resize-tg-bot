use std::path::Path;

use futures::{future, TryFutureExt};
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat};
use teloxide::prelude::*;
use teloxide::requests::RequestWithFile;
use teloxide::types::{
    Document, InputFile, MediaDocument, MediaKind, MediaPhoto, MessageCommon, MessageKind,
};
use teloxide_macros::teloxide;
use tempfile::NamedTempFile;
use tokio::fs::OpenOptions;

use crate::dialogue::Dialogue;

#[derive(Default)]
pub struct ResizeImageTgStickerState;

#[teloxide(subtransition)]
async fn resize_image_tg_sticker(
    state: ResizeImageTgStickerState,
    cx: TransitionIn,
) -> TransitionOut<Dialogue> {
    match &cx.update.kind {
        MessageKind::Common(MessageCommon {
            media_kind: MediaKind::Photo(MediaPhoto { photo: photos, .. }),
            ..
        }) => {
            let result = future::ready(
                photos
                    .first()
                    .ok_or(Error::PhotosAbsent)
                    .map(|ps| &ps.file_id),
            )
            .and_then(|file_id| resize_and_answer(&cx, file_id))
            .await;

            handle_result(state, &cx, result).await
        }
        MessageKind::Common(MessageCommon {
            media_kind:
                MediaKind::Document(MediaDocument {
                    document: Document { file_id, .. },
                    ..
                }),
            ..
        }) => {
            let result = resize_and_answer(&cx, file_id).await;
            handle_result(state, &cx, result).await
        }
        _ => {
            cx.answer_str("Expected image or file containing image.")
                .await?;
            next(state)
        }
    }
}

#[derive(Debug)]
enum Error {
    Image(image::ImageError),
    Io(std::io::Error),
    TeloxideRequest(teloxide::RequestError),
    TeloxideDownload(teloxide::DownloadError),
    PhotosAbsent,
}

const MAX_IMAGE_SIZE: u32 = 512;
const PNG_EXTENSION: &str = ".png";

async fn handle_result(
    state: ResizeImageTgStickerState,
    cx: &TransitionIn,
    result: Result<(), Error>,
) -> TransitionOut<Dialogue> {
    match result {
        Err(Error::TeloxideRequest(e)) => Err(e),
        Err(e) => {
            log::error!("{:?}", e);
            cx.answer_str("Couldn't process image.").await?;
            next(state)
        }
        Ok(_) => next(state),
    }
}

async fn resize_and_answer(cx: &TransitionIn, file_id: &str) -> Result<(), Error> {
    let tg_file_path = get_tg_file_path(&cx, file_id).await?;
    let tmp_file = create_tmp_file(PNG_EXTENSION)?;
    let tmp_file_path = tmp_file.path();

    download_file(&cx, tmp_file_path, &tg_file_path).await?;

    load_image(tmp_file_path)?
        .resize(MAX_IMAGE_SIZE, MAX_IMAGE_SIZE, FilterType::Lanczos3)
        .save_with_format(tmp_file_path, ImageFormat::Png)
        .map_err(Error::Image)?;

    cx.answer_document(InputFile::file(tmp_file_path))
        .send()
        .await
        .map_err(Error::Io)?
        .map_err(Error::TeloxideRequest)?;

    Ok(())
}

async fn get_tg_file_path(cx: &TransitionIn, file_id: &str) -> Result<String, Error> {
    cx.bot
        .get_file(file_id)
        .send()
        .await
        .map(|file| file.file_path)
        .map_err(Error::TeloxideRequest)
}

fn create_tmp_file(extension: &str) -> Result<NamedTempFile, Error> {
    tempfile::Builder::new()
        .suffix(extension)
        .tempfile()
        .map_err(Error::Io)
}

async fn download_file<P>(
    cx: &TransitionIn,
    tmp_file_path: P,
    tg_file_path: &str,
) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    let mut tokio_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(tmp_file_path)
        .await
        .map_err(Error::Io)?;

    cx.bot
        .download_file(&tg_file_path, &mut tokio_file)
        .await
        .map_err(Error::TeloxideDownload)
}

fn load_image<P>(path: P) -> Result<DynamicImage, Error>
where
    P: AsRef<Path>,
{
    let bytes = std::fs::read(path.as_ref()).map_err(Error::Io)?;
    image::load_from_memory(bytes.as_slice()).map_err(Error::Image)
}
