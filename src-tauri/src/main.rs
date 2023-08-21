// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cmd;
mod init_script;
mod matrix;

// mod placeholder_matrix_sdk_widget_driver;

// use placeholder_matrix_sdk_widget_driver::{Options, PermissionProvider, Info, Result, Widget, WidgetApi};
use tauri::{window, Manager, PageLoadPayload, Window};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
    time::Duration,
};

use matrix_sdk::{
    async_trait,
    config::SyncSettings,
    room::Room,
    ruma::{RoomId, _macros::room_id},
    widget_api::{
        api::widget::{self, Comm, Info},
        messages::capabilities::Options,
        run_client_widget_api, PermissionProvider, Result, Widget,
    },
    Client,
};

use init_script::INIT_SCRIPT;

fn send_post_message(window: &tauri::Window, message: &str) {
    let script = format!("postMessage({},'*')", message);
    // println!("eval js: {}", script);

    window.eval(&script).expect("could not eval js");
}

const WIDGET_ID: &str = "w_id_1234";

// ------ Struct for callbacks!!

// #[async_trait]
// impl Widget for ActualWidget {
//     fn send(&self, json: &str) -> Result<()> {
//         send_post_message(&self.window, json);
//         println!("\n## -> Outgoing msg: {}", json);
//         Ok(())
//     }

//     fn id(&self) -> &str {
//         WIDGET_ID
//     }
// }
struct PermProv {}
#[async_trait]
impl PermissionProvider for PermProv {
    async fn acquire_permissions(&self, cap: Options) -> Result<Options> {
        Ok(cap)
    }
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn handle_post_message(
    sender: tauri::State<UnboundedSender<String>>,
    window: tauri::Window,
    message: &str,
) {
    // ------ using the driver to process messages from the widget

    println!("\n## <- Incoming msg: {}", message);
    let _ = sender
        .send(message.to_owned())
        .map_err(|err| println!("Could not send message to driver: {}", err.to_string()));
}

fn app_setup(
    app: &mut tauri::App,
    on_page_load: Box<dyn FnOnce(Window, &str)>,
) -> std::result::Result<(), tauri::Error> {
    println!("in app_setup");
    // return Ok(());

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
    // let room_id = <&RoomId>::try_from("!IOSsIxnwmlSWYotZEl:my.matrix.host").unwrap();
    // println!("{}", room_id);
    // let Some(room) = client.get_joined_room(&room_id) else {panic!("could not get room")};
    // let widget = ActualWidget { window };

    // // ------ Setup everything, so the driver can access the room + the call callback on the widget
    // let mut wd = widget_driver.lock().expect("Mutex could not be locked");
    // *wd = Some(WidgetApi::new(
    //     room,
    //     widget,
    //     Info {
    //         id: WIDGET_ID.to_owned(),
    //         early_init: true,
    //     },
    // ));

    Ok(())
}
// struct WidgetDriver{
//     in_rx: UnboundedReceiver<String>,
//     out_rx: UnboundedReceiver<String>,
//     out_tx: UnboundedSender<String>,
// }
// impl WidgetDriver {
//     fn new(  client: &Client,
//         in_rx: UnboundedReceiver<String>,
//         mut out_rx: UnboundedReceiver<String>,
//         out_tx: UnboundedSender<String>,)->Self{
//             WidgetDriver{}

//     }
// }
fn widget_driver_setup(
    window: tauri::Window,
    client: &Client,
    in_rx: UnboundedReceiver<String>,
    mut out_rx: UnboundedReceiver<String>,
    out_tx: UnboundedSender<String>,
) {
    let room_id = <&RoomId>::try_from("!IOSsIxnwmlSWYotZEl:my.matrix.host").unwrap();
    println!("{}", room_id);
    let Some(room) = client.get_joined_room(&room_id) else {panic!("could not get room")};

    tokio::spawn(async move {
        while let Some(msg) = out_rx.recv().await {
            println!("\n## -> Outgoing msg: {}", msg);
            send_post_message(&window, &msg);
        }
    });
    // ------ Setup everything, so the driver can access the room + the call callback on the widget
    // let mut wid = widget.lock().expect("Mutex could not be locked");
    let wid = Widget {
        comm: Comm {
            from: in_rx,
            to: out_tx,
        },
        info: Info {
            id: WIDGET_ID.to_owned(),
            init_on_load: true,
        },
    };
    tokio::spawn(async move {
        let _ = run_client_widget_api(wid, PermProv {}, room)
            .await
            .map_err(|e| println!("run client widget api error: {}", e.to_description_string()));
        // tokio::time::sleep(Duration::from_millis(500)).await;
    });
    // *wd = Some(WidgetApi::new(
    //     room,
    //     widget,
    //     Info {
    //         id: WIDGET_ID.to_owned(),
    //         init_on_load: false,
    //     },
    // ));
}
#[tokio::main]
async fn main() {
    // parse the command line for homeserver, username and password
    let (homeserver_url, username, password) = (
        "http://localhost:8008".to_owned(),
        "timo".to_owned(),
        "1234".to_owned(),
    ); //cmd::get_args();
    let (out_tx, out_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let (in_tx, in_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

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

    let _sync_token = client
        .sync_once(SyncSettings::default())
        .await
        .unwrap()
        .next_batch;
    // let driver = WidgetDriver::new(&client, in_rx, out_rx, out_tx);
    // let arc_client = Arc::new(client);
    // let arc_in_rx = Arc::new(in_rx);
    // let arc_out_rx = Arc::new(out_rx);

    // Arc::<UnboundedReceiver<std::string::String>>::downgrade(&arc_in_rx);
    // let in_rx_ref = Arc::new(Mutex::new(in_rx));
    // let out_rx_ref = Arc::new(Mutex::new(out_rx));
    let on_page_load = move |window: tauri::Window, url: &str| {
        println!("LOAD: {}", url);
        // let out_tx = out_tx.clone();
        // let c = arc_client.clone();
        // let in_rx = arc_in_rx.clone();
        // let arc_in_rx2 = arc_in_rx.clone();
        // let in_rx = Arc::into_inner(arc_in_rx2).unwrap();
        // let out_rx = Arc::into_inner(arc_out_rx.clone()).unwrap();

        // let in_rx = in_rx.;
        widget_driver_setup(window, &client, in_rx, out_rx, out_tx.clone());
    };
    // let widget: Option<Widget> = None;
    // let arc_sender = Arc::new(Mutex::new(sender));
    // let sender_for_w = arc_sender.clone();
    tauri::Builder::default()
        .manage(in_tx.clone())
        .setup(move |app| Ok(app_setup(app, Box::new(on_page_load))?))
        // .on_page_load(on_page_load)
        .invoke_handler(tauri::generate_handler![handle_post_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
