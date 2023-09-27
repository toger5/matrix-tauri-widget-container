use async_channel::{Receiver, Sender};
use matrix_sdk::{
    async_trait,
    config::SyncSettings,
    ruma::RoomId,
    widget::{
        run_client_widget_api, Comm, Permissions, PermissionsProvider, Widget, WidgetSettings,
    },
    Client,
};

use crate::send_post_message;

struct PermProv {}
#[async_trait]
impl PermissionsProvider for PermProv {
    async fn acquire_permissions(&self, cap: Permissions) -> Permissions {
        cap
    }
}

pub fn widget_driver_setup(
    window: tauri::Window,
    client: &Client,
    in_rx: Receiver<String>,
    out_rx: Receiver<String>,
    out_tx: Sender<String>,
    room_id: &str,
    widget_settings: WidgetSettings,
) {
    let room_id = <&RoomId>::try_from(room_id).unwrap();
    println!("room id used by the driver: {} ", room_id);
    let Some(room) = client.get_room(&room_id) else {panic!("could not get room")};

    tokio::spawn(async move {
        while let Ok(msg) = out_rx.recv().await {
            send_post_message(&window, &msg);
        }
    });

    // ------ Setup everything, so the driver can access the room + the call callback on the widget
    let wid = Widget {
        comm: Comm {
            from: in_rx,
            to: out_tx,
        },
        settings: widget_settings,
    };

    tokio::spawn(async {
        run_client_widget_api(wid, PermProv {}, room).await;
    });

    let s_client = client.clone();
    tokio::spawn(async move {
        let sync_result = s_client.sync(SyncSettings::default()).await;
        println!("{:?}", sync_result);
    });
}
