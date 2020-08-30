use teloxide::types::BotCommand as BotCommandStruct;
use teloxide::utils::command::BotCommand;

#[derive(BotCommand, Copy, Clone, Eq, PartialEq, Debug)]
#[command(rename = "lowercase", description = "These commands are supported:")]
pub enum Command {
    Start,
    #[command(description = "display help text.")]
    Help,
    #[command(description = "resize image to use in @Stickers bot.")]
    ResizeTgSticker,
}

impl Command {
    pub fn from(input: &str) -> Option<Self> {
        use Command::*;
        match input {
            "/start" => Some(Start),
            "/help" => Some(Help),
            "/resize_tg_sticker" => Some(ResizeTgSticker),
            _ => None,
        }
    }

    // fixme: see https://github.com/teloxide/teloxide/issues/262
    pub fn values() -> Vec<BotCommandStruct> {
        vec![
            BotCommandStruct::new("/help", "display help text."),
            BotCommandStruct::new(
                "/resize_tg_sticker",
                "resize image to use in @Stickers bot.",
            ),
        ]
    }
}
