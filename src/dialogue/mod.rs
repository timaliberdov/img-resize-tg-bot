mod states;

use derive_more::From;
use teloxide_macros::Transition;

use crate::dialogue::states::{MainState, ReceiveImageTgStickerState};

#[derive(From, SmartDefault, Transition)]
pub enum Dialogue {
    #[default]
    Main(MainState),
    ReceivingImageTgSticker(ReceiveImageTgStickerState),
}
