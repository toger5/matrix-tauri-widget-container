// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cmd;
mod custom_messages;
mod init_script;
mod matrix;
mod widget_driver;

use cmd::{get_args, Args};

use init_script::INIT_SCRIPT;
use matrix_sdk::{
    config::SyncSettings,
    ruma::RoomId,
    widget::{ClientProperties, WidgetDriver, WidgetSettings}
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

    let widget_settings = WidgetSettings::new_virtual_element_call_widget(
        // "http://localhost:3000".to_owned(),
        // "https://pr1675--element-call.netlify.app".to_string(),
        "https://call.element.dev".to_owned(),
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
    )
    .map_err(|e| println!("could not create widget because: {}", e.to_string()))
    .unwrap();

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
    let Some(room) = client.get_room(&ruma_room_id) else {
        panic!("could not get room")
    };

    let props = ClientProperties::new("tauri.widget.container", None, None);

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
