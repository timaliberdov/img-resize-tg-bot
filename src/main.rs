#[macro_use]
extern crate smart_default;

use std::convert::Infallible;

use teloxide::prelude::*;

use crate::dialogue::Dialogue;

mod dialogue;
mod env;
mod webhook;

const BOT_USE_POLLING_ENV: &str = "BOT_USE_POLLING";

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    teloxide::enable_logging!();
    let use_polling: bool = env::get_env_opt(BOT_USE_POLLING_ENV)
        .and_then(|s| s.parse().ok())
        .unwrap_or(true);

    let bot = Bot::from_env();

    let dispatcher =
        Dispatcher::new(bot.clone()).messages_handler(DialogueDispatcher::new(|cx| async move {
            match handle_message(cx).await {
                Err(e) => {
                    log::error!("Error in handle_message: {}", e);
                    DialogueStage::Exit
                }
                Ok(res) => res,
            }
        }));

    if use_polling {
        dispatcher.dispatch().await;
    } else {
        dispatcher
            .dispatch_with_listener(
                webhook::start_webhook(bot.clone()).await,
                LoggingErrorHandler::with_custom_text("An error from the update listener"),
            )
            .await;
    }
}

async fn handle_message(
    cx: DialogueWithCx<Message, Dialogue, Infallible>,
) -> ResponseResult<DialogueStage<Dialogue>> {
    cx.dialogue.expect("Infallible").react(cx.cx, ()).await
}
