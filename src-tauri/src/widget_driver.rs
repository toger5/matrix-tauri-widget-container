use crate::custom_messages::_add_response;
use clone_all::clone_all;
use matrix_sdk::{
    async_trait,
    config::SyncSettings,
    widget::{Permissions, PermissionsProvider, WidgetDriver, WidgetDriverHandle, WidgetSettings},
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
    println!(
        "\n## -> Outgoing msg: {:#?}",
        serde_json::from_str::<serde_json::Value>(message).unwrap()
    );
    let script = format!("postMessage({},'{}')", message, url);
    window.eval(&script).expect("could not eval js");
}

#[tauri::command]
pub fn handle_post_message(
    handle: tauri::State<WidgetDriverHandle>,
    client: tauri::State<Client>,
    message: &str,
) {
    let handle = handle.inner();
    let client = client.inner();

    println!("\n## <- Incoming msg: {}", message);
    let msg = message.to_owned();
    if msg.contains("im.vector.hangup") {
        clone_all!(msg, handle, client);
        tokio::spawn(async move {
            let logout_res = client.matrix_auth().logout().await;
            println!("Logout_result: {:?}", logout_res);
            let res = _add_response(&msg, "".into()).unwrap();
            handle.send(res).await;
        });
    } else {
        clone_all!(handle);
        tokio::spawn(async move {
            if !handle.send(msg).await {
                println!("Could not send message to driver");
            };
        });
    }
}

pub struct WidgetData {
    pub driver: WidgetDriver,
    pub handle: WidgetDriverHandle,
    pub room: Room,
    pub widget_settings: WidgetSettings,
    pub generated_url: Url,
}
pub fn widget_driver_setup(window: tauri::Window, client: &Client, widget_data: WidgetData) {
    let WidgetData {
        driver,
        handle,
        room,
        widget_settings,
        generated_url: _url,
    } = widget_data;

    let url = widget_settings.base_url().unwrap().to_string();
    tokio::spawn(async move {
        while let Some(msg) = handle.recv().await {
            send_post_message(&window, &msg, &url);
        }
    });

    tokio::spawn(async {
        let _ = driver.run(room, PermProv {}).await;
    });

    let client = client.clone();
    tokio::spawn(async move {
        let sync_result = client.sync(SyncSettings::default()).await;
        println!("{:?}", sync_result);
    });
}
