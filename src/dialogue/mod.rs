use derive_more::From;
use teloxide_macros::Transition;

use crate::dialogue::states::ResizeImageTgStickerState;

mod states;

#[derive(From, SmartDefault, Transition)]
pub enum Dialogue {
    #[default]
    ResizeImageTgSticker(ResizeImageTgStickerState),
}
