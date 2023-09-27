// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use url::Url;
mod cmd;
mod custom_messages;
mod element_call_url;
mod init_script;
mod matrix;
mod widget_driver;

use async_channel::unbounded;
use cmd::{get_args, Args};
use element_call_url::EC_URL;

use init_script::INIT_SCRIPT;
use matrix_sdk::{
    config::SyncSettings,
    ruma::RoomId,
    widget::{ClientProperties, WidgetSettings},
    Client,
};
use widget_driver::{widget_driver_setup, WidgetData};

use crate::widget_driver::handle_post_message;

fn app_setup(
    app: &mut tauri::App,
    client: &Client,
    widget_data: WidgetData,
) -> std::result::Result<(), tauri::Error> {
    println!("use url:\n{}", widget_data.url);
    let element_call_url = tauri::WindowUrl::External(widget_data.url.clone());
    let window = tauri::WindowBuilder::new(app, "widget_window", element_call_url)
        .initialization_script(INIT_SCRIPT)
        .build()?;

    widget_driver_setup(window, &client, widget_data);

    Ok(())
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

    // create comm channels between widget and driver
    let (tx_client_widget, client_widget_rx) = unbounded::<String>();
    let (tx_widget_client, widget_client_rx) = unbounded::<String>();

    let widget_data = WidgetData {
        client_widget_rx,
        widget_client_rx,
        tx_client_widget,
        room_id,
        widget_settings: widget_settings.clone(),
        url: ec_url,
    };

    tauri::Builder::default()
        .manage(tx_widget_client)
        .setup(move |app| Ok(app_setup(app, &client, widget_data)?))
        .invoke_handler(tauri::generate_handler![handle_post_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
