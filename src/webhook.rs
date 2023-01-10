use std::convert::Infallible;

use teloxide::{
    dispatching::update_listeners::{webhooks, UpdateListener},
    prelude::*,
};

use crate::env::get_env;

const WEBHOOK_HOST_ENV: &str = "WEBHOOK_HOST";
const WEBHOOK_PORT_ENV: &str = "PORT";

pub async fn start_webhook(bot: Bot) -> impl UpdateListener<Err = Infallible> {
    let host = get_env(WEBHOOK_HOST_ENV);
    let port: u16 = get_env(WEBHOOK_PORT_ENV)
        .parse()
        .expect("Port value should be integer");

    let addr = ([0, 0, 0, 0], port).into();
    let url = format!("https://{host}/webhook").parse().unwrap();

    webhooks::axum(bot.clone(), webhooks::Options::new(addr, url))
        .await
        .expect("Couldn't setup webhook")
}
