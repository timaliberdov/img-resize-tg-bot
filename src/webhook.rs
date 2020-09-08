use std::{convert::Infallible, net::SocketAddr};
use teloxide::Bot;
use tokio::sync::mpsc;
use warp::Filter;

use crate::env::get_env;
use teloxide::{dispatching::update_listeners, prelude::*};
use update_listeners::UpdateListener;
use warp::http::StatusCode;

const TELOXIDE_TOKEN_ENV: &str = "TELOXIDE_TOKEN";
const WEBHOOK_HOST_ENV: &str = "WEBHOOK_HOST_ENV";
const WEBHOOK_PORT_ENV: &str = "WEBHOOK_PORT_ENV";

async fn handle_rejection(error: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    log::error!("Could not process the request due to: {:?}", error);
    Ok(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn start_webhook(bot: Bot) -> impl UpdateListener<serde_json::Error> {
    let host = get_env(WEBHOOK_HOST_ENV);
    let port: u16 = get_env(WEBHOOK_PORT_ENV)
        .parse()
        .expect("Port value should be integer");
    let token = get_env(TELOXIDE_TOKEN_ENV);

    let url = format!("https://{}/{}", host, token);

    bot.set_webhook(url)
        .send()
        .await
        .expect("Could not setup a webhook");

    let (sender, receiver) = mpsc::unbounded_channel();

    let server = warp::post()
        .and(warp::path(token))
        .and(warp::body::json())
        .map(move |json: serde_json::Value| {
            let try_parse = match serde_json::from_str(&json.to_string()) {
                Ok(update) => Ok(update),
                Err(error) => {
                    log::error!(
                        "Could not parse an update.\nError: {:?}\nValue: {}\n\
                       It's probably a is a teloxide bug: https://github.com/teloxide/teloxide",
                        error,
                        json
                    );
                    Err(error)
                }
            };

            if try_parse.is_ok() {
                sender
                    .send(try_parse)
                    .expect("Could not send an incoming update from the webhook")
            }

            StatusCode::OK
        })
        .recover(handle_rejection);

    let serve = warp::serve(server);

    let address = format!("0.0.0.0:{}", port);
    tokio::spawn(serve.run(address.parse::<SocketAddr>().unwrap()));
    receiver
}
