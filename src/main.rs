use commands::{commands_handler, Command};
use resize_image_handler::{handle_document, handle_photo};
use teloxide::{prelude::*, types::Update};

mod commands;
mod env;
mod resize_image_handler;
mod webhook;

const BOT_USE_POLLING_ENV: &str = "BOT_USE_POLLING";

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    pretty_env_logger::init();
    let use_polling: bool = env::get_env_opt(BOT_USE_POLLING_ENV)
        .and_then(|s| s.parse().ok())
        .unwrap_or(true);

    let bot = Bot::from_env();

    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(commands_handler),
        )
        .branch(Message::filter_photo().endpoint(handle_photo))
        .branch(Message::filter_document().endpoint(handle_document));

    let mut dispatcher = Dispatcher::builder(bot.clone(), handler)
        .enable_ctrlc_handler()
        .build();

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
