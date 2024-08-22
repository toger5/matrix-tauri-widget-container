// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cmd;
mod custom_messages;
mod init_script;
mod matrix;
mod widget_driver;
use cmd::{get_args, Args};
use matrix_sdk::widget::EncryptionSystem;
use matrix_sdk::Client;
use matrix_sdk::Room;

use init_script::INIT_SCRIPT;
use matrix_sdk::{
    config::SyncSettings,
    ruma::RoomId,
    widget::{ClientProperties, VirtualElementCallWidgetOptions, WidgetDriver, WidgetSettings},
};
use tauri::Manager;
use url::Url;
use widget_driver::{handle_post_message, widget_driver_setup, WidgetData};

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
    let Some(room) = client.get_room(ruma_room_id) else {
        panic!("could not get room")
    };

    // very ugly way to switch between widget driver test / and room_info test
    room_info_experiment(client, room).await;
    // element_call(client, room).await;
}

async fn room_info_experiment(client: Client, room: Room) {
    println!("Start room info: \n\n {:#?}", room.clone_info());
    tokio::spawn(async move {
        while room.subscribe_info().next().await.is_some() {
            // println!("Updated room info: \n\n {:?}", info);
            println!(
                "active_call: {}\n\
                participants: {:?}\n\
                count: {}",
                room.has_active_room_call(),
                room.active_room_call_participants(),
                room.active_room_call_participants().len(),
            )
        }
    });
    let _ = client.sync(SyncSettings::default()).await;
}

async fn element_call(client: Client, room: Room) {
    let props = ClientProperties::new("tauri.widget.container", None, Some("dark".to_owned()));

    let options = VirtualElementCallWidgetOptions {
        element_call_url: "https://call.element.io".to_owned(),
        // element_call_url: "https://localhost:3000".to_owned(),
        widget_id: "w_id_1234".to_owned(),
        parent_url: None,
        hide_header: None,
        preload: None,
        font_scale: None,
        app_prompt: None,
        skip_lobby: None,
        confine_to_room: None,
        analytics_id: None,
        font: None,
        encryption: EncryptionSystem::Unencrypted,
    };

    let widget_settings = WidgetSettings::new_virtual_element_call_widget(options)
        .map_err(|e| println!("could not create widget because: {}", e.to_string()))
        .unwrap();

    let generated_url = widget_settings
        .generate_webview_url(&room, props)
        .await
        .expect("could not parse url");

    let (driver, handle) = WidgetDriver::new(widget_settings.clone());

    let widget_data = WidgetData {
        driver,
        handle: handle.clone(),
        room,
        widget_settings,
        generated_url,
    };
    tauri::Builder::default()
        .manage(handle)
        .manage(client.clone())
        .setup(move |app|
            // Ok(app_setup(app, &client, widget_data)?)
            {
            println!("use url:\n{}\n", widget_data.generated_url);
            let element_call_url =
                tauri::WindowUrl::External(Url::parse(&widget_data.generated_url.to_string()).unwrap());
            let window = tauri::WindowBuilder::new(app, "widget_window", element_call_url)
                .initialization_script(INIT_SCRIPT)
                .build()?;

            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_window("widget_window").unwrap();
                window.open_devtools();
            }
            widget_driver_setup(window, &client, widget_data);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![handle_post_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
