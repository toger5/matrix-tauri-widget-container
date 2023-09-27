use async_channel::{Receiver, Sender};
use matrix_sdk::{
    async_trait,
    config::SyncSettings,
    widget::{
        run_client_widget_api, Comm, Permissions, PermissionsProvider, Widget, WidgetSettings,
    },
    Client, Room,
};
use url::Url;

struct PermProv {}
#[async_trait]
impl PermissionsProvider for PermProv {
    async fn acquire_permissions(&self, cap: Permissions) -> Permissions {
        cap
    }
}

fn send_post_message(window: &tauri::Window, message: &str, url: &str) {
    println!("\n## -> Outgoing msg: {}", message);
    let script = format!("postMessage({},'{}')", message, url);
    window.eval(&script).expect("could not eval js");
}

#[tauri::command]
pub fn handle_post_message(sender: tauri::State<Sender<String>>, message: &str) {
    println!("\n## <- Incoming msg: {}", message);
    let _ = sender
        .send_blocking(message.to_owned())
        .map_err(|err| println!("Could not send message to driver: {}", err.to_string()));
}

pub struct WidgetData {
    pub client_widget_rx: Receiver<String>,
    pub widget_client_rx: Receiver<String>,
    pub tx_client_widget: Sender<String>,
    pub room: Room,
    pub widget_settings: WidgetSettings,
    pub url: Url,
}
pub fn widget_driver_setup(window: tauri::Window, client: &Client, widget_data: WidgetData) {
    let WidgetData {
        client_widget_rx,
        widget_client_rx,
        tx_client_widget,
        room,
        widget_settings,
        url: _url,
    } = widget_data;

    let url = widget_settings.raw_url.clone();
    tokio::spawn(async move {
        while let Ok(msg) = client_widget_rx.recv().await {
            send_post_message(&window, &msg, &url);
        }
    });

    // ------ Setup everything, so the driver can access the room + the call callback on the widget
    let wid = Widget {
        comm: Comm {
            from: widget_client_rx,
            to: tx_client_widget,
        },
        settings: widget_settings,
    };

    tokio::spawn(async {
        run_client_widget_api(wid, PermProv {}, room).await;
    });

    let client = client.clone();
    tokio::spawn(async move {
        let sync_result = client.sync(SyncSettings::default()).await;
        println!("{:?}", sync_result);
    });
}
