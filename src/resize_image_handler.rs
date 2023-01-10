use std::path::Path;

use futures::{future, TryFutureExt};
use image::{imageops::FilterType, DynamicImage, ImageFormat};
use teloxide::{
    net::Download,
    prelude::*,
    types::{Document, FileMeta, InputFile, PhotoSize},
};
use tempfile::NamedTempFile;
use tokio::fs::OpenOptions;

const MAX_IMAGE_SIZE: u32 = 512;
const PNG_EXTENSION: &str = ".png";

pub async fn handle_photo(
    bot: Bot,
    msg: Message,
    photos: Vec<PhotoSize>,
) -> Result<(), teloxide::RequestError> {
    let mut sorted_photos = photos;
    sorted_photos.sort_by_key(|ps| std::cmp::Reverse(ps.width));

    let result = future::ready(
        sorted_photos
            .first()
            .ok_or(Error::PhotosAbsent)
            .map(|ps| &ps.file),
    )
    .and_then(|file| resize_and_answer(&bot, &msg, file))
    .await;

    handle_result(&bot, &msg, result).await;

    Ok(())
}

pub async fn handle_document(
    bot: Bot,
    msg: Message,
    document: Document,
) -> Result<(), teloxide::RequestError> {
    let result = resize_and_answer(&bot, &msg, &document.file).await;

    handle_result(&bot, &msg, result).await;

    Ok(())
}

#[derive(Debug)]
enum Error {
    Image(image::ImageError),
    Io(std::io::Error),
    TeloxideRequest(teloxide::RequestError),
    TeloxideDownload(teloxide::DownloadError),
    PhotosAbsent,
}

async fn handle_result(bot: &Bot, msg: &Message, result: Result<(), Error>) {
    if let Err(e) = result {
        log::error!("Error in handle_message: {:?}", e);
        bot.send_message(msg.chat.id, "Couldn't process image.")
            .await
            .log_on_error()
            .await;
    }
}

async fn resize_and_answer(bot: &Bot, msg: &Message, file_meta: &FileMeta) -> Result<(), Error> {
    let tg_file_path = get_tg_file_path(bot, file_meta).await?;
    let tmp_file = create_tmp_file(PNG_EXTENSION)?;
    let tmp_file_path = tmp_file.path();

    download_file(bot, tmp_file_path, &tg_file_path).await?;

    load_image(tmp_file_path)?
        .resize(MAX_IMAGE_SIZE, MAX_IMAGE_SIZE, FilterType::Triangle)
        .save_with_format(tmp_file_path, ImageFormat::Png)
        .map_err(Error::Image)?;

    bot.send_document(msg.chat.id, InputFile::file(tmp_file_path))
        .await
        .map_err(Error::TeloxideRequest)?;

    Ok(())
}

async fn get_tg_file_path(bot: &Bot, file_meta: &FileMeta) -> Result<String, Error> {
    bot.get_file(&file_meta.id)
        .await
        .map(|file| file.path)
        .map_err(Error::TeloxideRequest)
}

fn create_tmp_file(extension: &str) -> Result<NamedTempFile, Error> {
    tempfile::Builder::new()
        .suffix(extension)
        .tempfile()
        .map_err(Error::Io)
}

async fn download_file<P>(bot: &Bot, tmp_file_path: P, tg_file_path: &str) -> Result<(), Error>
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

    bot.download_file(tg_file_path, &mut tokio_file)
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
