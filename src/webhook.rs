use std::{convert::Infallible, net::SocketAddr};

use reqwest::StatusCode;
use teloxide::Bot;
use teloxide::{dispatching::update_listeners::UpdateListener, prelude::*};
use tokio::sync::mpsc;
use warp::Filter;

use crate::env::get_env;

const WEBHOOK_HOST_ENV: &str = "WEBHOOK_HOST";
const WEBHOOK_PORT_ENV: &str = "PORT";

async fn handle_rejection(error: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
    log::error!("Could not process the request due to: {:?}", error);
    Ok(StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn start_webhook(bot: Bot) -> impl UpdateListener<Infallible> {
    let host = get_env(WEBHOOK_HOST_ENV);
    let port: u16 = get_env(WEBHOOK_PORT_ENV)
        .parse()
        .expect("Port value should be integer");
    let path = format!("bot{}", bot.token());
    let url = format!("https://{}/{}", host, path);

    bot.set_webhook(url)
        .send()
        .await
        .expect("Could not setup a webhook");

    let (sender, receiver) = mpsc::unbounded_channel();

    let server = warp::post()
        .and(warp::path(path))
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

            if let Ok(update) = try_parse {
                sender
                    .send(Ok(update))
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
