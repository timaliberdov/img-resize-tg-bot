#[macro_use]
extern crate smart_default;

use teloxide::prelude::*;

#[derive(SmartDefault)]
pub enum Dialogue {
    #[default]
    Start,
}

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    teloxide::enable_logging!();
    let bot = Bot::from_env();

    teloxide::dialogues_repl(bot, |message, dialogue| async move {
        match handle_message(message, dialogue).await {
            Err(e) => {
                log::error!("Error in handle_message: {}", e);
                DialogueStage::Exit
            }
            Ok(res) => res,
        }
    })
    .await;
}

async fn handle_message(cx: UpdateWithCx<Message>, dialogue: Dialogue) -> TransitionOut<Dialogue> {
    match dialogue {
        Dialogue::Start => {
            // todo
            log::info!("Chat id: {}", cx.chat_id());
            cx.answer_dice().send().await?;
            exit()
        }
    }
}
