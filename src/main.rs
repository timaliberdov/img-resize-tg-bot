use teloxide::prelude::*;

mod env;
mod resize_image_handler;
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
        Dispatcher::new(bot.clone()).messages_handler(|rx: DispatcherHandlerRx<Message>| {
            rx.for_each_concurrent(None, resize_image_handler::handle_message)
        });

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
