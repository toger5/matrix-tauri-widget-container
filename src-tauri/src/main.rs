// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use url::Url;
mod cmd;
mod custom_messages;
mod element_call_url;
mod init_script;
mod matrix;
mod widget_driver;

use async_channel::{unbounded, Sender};
use cmd::{get_args, Args};
use element_call_url::EC_URL;
use serde_json::json;
use tauri::Window;

use init_script::INIT_SCRIPT;
use matrix_sdk::{
    config::SyncSettings,
    ruma::RoomId,
    widget::{ClientProperties, WidgetSettings},
};
use widget_driver::widget_driver_setup;

use crate::custom_messages::add_response;

fn send_post_message(window: &tauri::Window, message: &str) {
    println!("\n## -> Outgoing msg: {}", message);
    let script = format!("postMessage({},'{}')", message, element_call_url::EC_URL);
    window.eval(&script).expect("could not eval js");
}

fn app_setup(
    app: &mut tauri::App,
    on_page_load: Box<dyn FnOnce(Window, &str)>,
    url: Url,
) -> std::result::Result<(), tauri::Error> {
    println!("use url:\n{}", url);
    let element_call_url = tauri::WindowUrl::External(url.clone());
    let window = tauri::WindowBuilder::new(app, "widget_window", element_call_url)
        .initialization_script(INIT_SCRIPT)
        .build()?;
    on_page_load(window, &url.to_string());
    Ok(())
}

#[tauri::command]
fn handle_post_message(
    sender: tauri::State<Sender<String>>,
    widget_settings: tauri::State<WidgetSettings>,
    window: tauri::Window,
    message: &str,
) {
    // ------ using the driver to process messages from the widget

    println!("\n## <- Incoming msg: {}", message);
    if message.contains("watch_turn_servers") {
        send_post_message(
            &window,
            &add_response(message, json!({}))
                .expect("could not add response to watch_turn_servers"),
        );
        send_post_message(&window, &custom_messages::join_action(&widget_settings.id));
        println!(
            "Did not pass message (watch_turn_servers) to widget driver but handled it locally"
        )
    } else {
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

    let widget_settings: WidgetSettings = WidgetSettings::new_virtual_element_call_widget(
        EC_URL.to_owned(),
        "w_id_1234".to_owned(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

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
    let ruma_room_id = <&RoomId>::try_from(room_id.as_str()).unwrap();
    let Some(room) = client.get_room(&ruma_room_id) else {panic!("could not get room")};
    let props = ClientProperties::new("tauri.widget.container", None, None);
    let ec_url = widget_settings
        .generate_url(&room, props)
        .await
        .expect("could not parse url");

    let room_id_ = room_id.clone();
    let widget_settings_ = widget_settings.clone();
    let on_page_load = move |window: tauri::Window, _url: &str| {
        widget_driver_setup(
            window,
            &client,
            in_rx,
            out_rx,
            out_tx.clone(),
            &(room_id_.clone()),
            widget_settings_,
        );
    };

    tauri::Builder::default()
        .manage(in_tx.clone())
        .manage(widget_settings.clone())
        .setup(move |app| Ok(app_setup(app, Box::new(on_page_load), ec_url)?))
        // .on_page_load(on_page_load)
        .invoke_handler(tauri::generate_handler![handle_post_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
