// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cmd;
mod init_script;
mod matrix;
mod widget_driver;

use tauri::Window;
use async_channel::{unbounded, Sender};

use init_script::INIT_SCRIPT;
use matrix_sdk::config::SyncSettings;
use widget_driver::widget_driver_setup;

const WIDGET_ID: &str = "w_id_1234";

fn send_post_message(window: &tauri::Window, message: &str) {
    let script = format!("postMessage({},'*')", message);
    // println!("eval js: {}", script);

    window.eval(&script).expect("could not eval js");
}

fn app_setup(
    app: &mut tauri::App,
    on_page_load: Box<dyn FnOnce(Window, &str)>,
) -> std::result::Result<(), tauri::Error> {
    let url = "index.html".to_string();
    let window = tauri::WindowBuilder::new(
        app,
        "widget_window",
        tauri::WindowUrl::App(url.clone().into()),
    )
    .initialization_script(INIT_SCRIPT)
    .build()?;
    window.open_devtools();
    on_page_load(window, &url);

    Ok(())
}

#[tauri::command]
fn handle_post_message(
    sender: tauri::State<Sender<String>>,
    _window: tauri::Window,
    message: &str,
) {
    // ------ using the driver to process messages from the widget

    println!("\n## <- Incoming msg: {}", message);
    let _ = sender
        .send_blocking(message.to_owned())
        .map_err(|err| println!("Could not send message to driver: {}", err.to_string()));
}

#[tokio::main]
async fn main() {
    // parse the command line for homeserver, username and password
    let (homeserver_url, username, password) = (
        "http://localhost:8008".to_owned(),
        "timo".to_owned(),
        "1234".to_owned(),
    ); //cmd::get_args();
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

    let on_page_load = move |window: tauri::Window, _url: &str| {
        widget_driver_setup(window, &client, in_rx, out_rx, out_tx.clone());
    };

    tauri::Builder::default()
        .manage(in_tx.clone())
        .setup(move |app| Ok(app_setup(app, Box::new(on_page_load))?))
        // .on_page_load(on_page_load)
        .invoke_handler(tauri::generate_handler![handle_post_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
