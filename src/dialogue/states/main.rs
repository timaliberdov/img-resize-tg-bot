use crate::dialogue::{states::receive_image_tg_sticker::ReceiveImageTgStickerState, Dialogue};
use teloxide::prelude::*;
use teloxide_macros::teloxide;

use crate::commands::Command;

#[derive(Default)]
pub struct MainState;

#[teloxide(subtransition)]
async fn main(state: MainState, cx: TransitionIn) -> TransitionOut<Dialogue> {
    match cx.update.text_owned() {
        None => {
            cx.answer_str("Command expected. Try /help to get info about available commands")
                .await?;
            next(state)
        }
        Some(input) => handle_command(state, cx, &input).await,
    }
}

async fn handle_command(
    state: MainState,
    cx: TransitionIn,
    input: &str,
) -> TransitionOut<Dialogue> {
    match Command::from(input) {
        None => {
            cx.answer_str("Unknown or absent command. Try /help")
                .await?;
            next(state)
        }
        Some(command) => {
            // todo
            if command == Command::ResizeTgSticker {
                cx.answer_str("OK. Send me a picture").await?;
                next(ReceiveImageTgStickerState)
            } else {
                cx.answer_str("Not implemented :c").await?;
                next(state)
            }
        }
    }
}
