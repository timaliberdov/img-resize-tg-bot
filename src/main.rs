#[macro_use]
extern crate smart_default;

use crate::commands::Command;
use crate::dialogue::Dialogue;
use std::convert::Infallible;
use teloxide::prelude::*;
use teloxide::types::BotCommand;

mod commands;
mod dialogue;

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    teloxide::enable_logging!();
    let bot = Bot::from_env();

    let commands: Vec<BotCommand> = Command::values();
    if let Some(e) = bot.set_my_commands(commands).send().await.err() {
        log::error!("Error in set_my_commands: {}", e);
    }

    Dispatcher::new(bot)
        .messages_handler(DialogueDispatcher::new(|cx| async move {
            match handle_message(cx).await {
                Err(e) => {
                    log::error!("Error in handle_message: {}", e);
                    DialogueStage::Exit
                }
                Ok(res) => res,
            }
        }))
        .dispatch()
        .await;
}

async fn handle_message(
    cx: DialogueWithCx<Message, Dialogue, Infallible>,
) -> ResponseResult<DialogueStage<Dialogue>> {
    cx.dialogue.expect("Infallible").react(cx.cx, ()).await
}
