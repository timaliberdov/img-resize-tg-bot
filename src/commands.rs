use teloxide::{requests::Requester, types::Message, utils::command::BotCommands, Bot};

const HELP_MESSAGE: &str = "Hello, I'm the Resize Image Bot! \
                            If you send me an image or an image file, \
                            I can resize it to fit in a 512x512 square, \
                            and send you back the file in PNG format. \
                            \n\nThe result can be sent to the @Stickers bot to \
                            add a new sticker to your sticker pack!";

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub(crate) enum Command {
    Start,
    #[command(description = "display this text")]
    Help,
}

pub(crate) async fn commands_handler(
    bot: Bot,
    msg: Message,
    cmd: Command,
) -> Result<(), teloxide::RequestError> {
    let text = match cmd {
        Command::Help | Command::Start => HELP_MESSAGE,
    };

    bot.send_message(msg.chat.id, text).await?;

    Ok(())
}
