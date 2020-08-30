use std::path::Path;

use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat};
use teloxide::prelude::*;
use teloxide::requests::RequestWithFile;
use teloxide::types::{InputFile, MediaKind, MediaPhoto, MessageCommon, MessageKind, PhotoSize};
use teloxide_macros::teloxide;
use tokio::fs::OpenOptions;

use crate::dialogue::{states::main::MainState, Dialogue};
use tempfile::NamedTempFile;

pub struct ReceiveImageTgStickerState;

#[teloxide(subtransition)]
async fn receive_image_tg_sticker(
    state: ReceiveImageTgStickerState,
    cx: TransitionIn,
) -> TransitionOut<Dialogue> {
    if let MessageKind::Common(MessageCommon {
        // todo: handle attached files
        media_kind: MediaKind::Photo(MediaPhoto { photo: photos, .. }),
        ..
    }) = &cx.update.kind
    {
        // todo: handle media groups for batch resizing (seems like every file in group is in separate message)
        if let Some(PhotoSize { file_id, .. }) = photos.first() {
            match resize_and_answer(&cx, file_id).await {
                Err(Error::TeloxideRequest(e)) => return Err(e),
                Err(e) => log::error!("{:?}", e),
                Ok(_) => {}
            }
            return next(MainState);
        }
    }
    next(state)
}

#[derive(Debug)]
enum Error {
    Image(image::ImageError),
    Io(std::io::Error),
    TeloxideRequest(teloxide::RequestError),
    TeloxideDownload(teloxide::DownloadError),
}

const MAX_IMAGE_SIZE: u32 = 512;
const PNG_EXTENSION: &'static str = ".png";

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
