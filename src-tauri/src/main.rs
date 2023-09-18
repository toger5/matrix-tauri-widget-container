// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cmd;
mod custom_messages;
mod element_call_url;
mod init_script;
mod matrix;
mod widget_driver;

use async_channel::{unbounded, Sender};
use cmd::{get_args, Args};
use serde_json::json;
use tauri::Window;

use init_script::INIT_SCRIPT;
use matrix_sdk::config::SyncSettings;
use widget_driver::widget_driver_setup;

use crate::custom_messages::add_response;

const WIDGET_ID: &str = "w_id_1234";

fn send_post_message(window: &tauri::Window, message: &str) {
    println!("\n## -> Outgoing msg: {}", message);
    let script = format!("postMessage({},'{}')", message, element_call_url::EC_URL);
    window.eval(&script).expect("could not eval js");
}

fn app_setup(
    app: &mut tauri::App,
    on_page_load: Box<dyn FnOnce(Window, &str)>,
    room_id: &str,
    user_id: &str,
) -> std::result::Result<(), tauri::Error> {
    let url = "index.html".to_string();
    let _app_url = tauri::WindowUrl::App(url.clone().into());
    let ec_url = element_call_url::url(room_id, user_id, WIDGET_ID);
    println!("use url:\n{}", ec_url);
    let element_call_url = tauri::WindowUrl::External(ec_url.parse().unwrap());
    let window = tauri::WindowBuilder::new(app, "widget_window", element_call_url)
        .initialization_script(INIT_SCRIPT)
        .build()?;
    on_page_load(window, &ec_url);

    Ok(())
}

#[tauri::command]
fn handle_post_message(sender: tauri::State<Sender<String>>, window: tauri::Window, message: &str) {
    // ------ using the driver to process messages from the widget

    println!("\n## <- Incoming msg: {}", message);
    if message.contains("watch_turn_servers") {
        send_post_message(
            &window,
            &add_response(message, json!({}))
                .expect("could not add response to watch_turn_servers"),
        );
        send_post_message(&window, &custom_messages::join_action(WIDGET_ID));
        println!(
            "Did not pass message (watch_turn_servers) to widget driver but handled it locally"
        )
    }
    // else if message.contains("set_always_on_screen") {
    //     send_post_message(
    //         &window,
    //         &add_response(message, json!({}))
    //             .expect("could not add response to set_always_on_screen"),
    //     );
    // } else if message.contains("im.vector.hangup") {
    //     send_post_message(
    //         &window,
    //         &add_response(message, json!({})).expect("could not add response to im.vector.hangup"),
    //     );
    // } else if message.contains("io.element.tile_layout") {
    //     send_post_message(
    //         &window,
    //         &add_response(message, json!({}))
    //             .expect("could not add response to io.element.tile_layout"),
    //     );
    // }
    else {
        let _ = sender
            .send_blocking(message.to_owned())
            .map_err(|err| println!("Could not send message to driver: {}", err.to_string()));
    }
}

#[tokio::main]
async fn main() {
    // start logger
    tracing_subscriber::fmt::init();

    // parse the command line for homeserver, username and password
    let Args {
        homeserver_url,
        username,
        password,
        room_id,
    } = get_args();
    // (
    //     "http://localhost:8008".to_owned(),
    //     "timo".to_owned(),
    //     "1234".to_owned(),
    // );

    // create comm channels between widget and driver
    let (out_tx, out_rx) = unbounded::<String>();
    let (in_tx, in_rx) = unbounded::<String>();

    // create the logged in sdk client.
    let client = match matrix::login(homeserver_url.clone(), username.clone(), password.clone())
        .await
    {
        Ok(client) => client,
        Err(e) => {
            println!("Could not create client with provided args.\n homeserver: {homeserver_url}, username: {username}, password: {password}");
            println!("{e}");
            return;
        }
    };

    // sync once so that the client is ready to listen to room events.
    let _sync_token = client
        .sync_once(SyncSettings::default())
        .await
        .unwrap()
        .next_batch;
    let room_id_ = room_id.clone();
    let on_page_load = move |window: tauri::Window, _url: &str| {
        widget_driver_setup(
            window,
            &client,
            in_rx,
            out_rx,
            out_tx.clone(),
            &(room_id_.clone()),
        );
    };

    tauri::Builder::default()
        .manage(in_tx.clone())
        .setup(move |app| Ok(app_setup(app, Box::new(on_page_load), &room_id, &username)?))
        // .on_page_load(on_page_load)
        .invoke_handler(tauri::generate_handler![handle_post_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
