use std::path::Path;

use futures::{future, TryFutureExt};
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat};
use teloxide::prelude::*;
use teloxide::requests::RequestWithFile;
use teloxide::types::{
    Document, InputFile, MediaDocument, MediaKind, MediaPhoto, MessageCommon, MessageKind,
};
use tempfile::NamedTempFile;
use tokio::fs::OpenOptions;

const START_COMMAND: &str = "/start";
const HELP_COMMAND: &str = "/help";

const HELP_MESSAGE: &str = "Hello, I'm the Resize Image Bot! \
                            If you send me an image or an image file, \
                            I can resize it to fit in a 512x512 square, \
                            and send you back the file in PNG format. \
                            \n\nThe result can be sent to the @Stickers bot to \
                            add a new sticker to your sticker pack!";

const MAX_IMAGE_SIZE: u32 = 512;
const PNG_EXTENSION: &str = ".png";

pub async fn handle_message(cx: UpdateWithCx<Message>) {
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

            handle_result(&cx, result).await
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
            handle_result(&cx, result).await
        }
        _ => match cx.update.text() {
            Some(START_COMMAND) | Some(HELP_COMMAND) => {
                cx.answer_str(HELP_MESSAGE).await.log_on_error().await;
            }
            _ => {
                cx.answer_str("Expected image or file containing image.")
                    .await
                    .log_on_error()
                    .await;
            }
        },
    };
}

#[derive(Debug)]
enum Error {
    Image(image::ImageError),
    Io(std::io::Error),
    TeloxideRequest(teloxide::RequestError),
    TeloxideDownload(teloxide::DownloadError),
    PhotosAbsent,
}

async fn handle_result(cx: &UpdateWithCx<Message>, result: Result<(), Error>) {
    if let Err(e) = result {
        log::error!("Error in handle_message: {:?}", e);
        cx.answer_str("Couldn't process image.")
            .await
            .log_on_error()
            .await;
    }
}

async fn resize_and_answer(cx: &UpdateWithCx<Message>, file_id: &str) -> Result<(), Error> {
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

async fn get_tg_file_path(cx: &UpdateWithCx<Message>, file_id: &str) -> Result<String, Error> {
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
    cx: &UpdateWithCx<Message>,
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
