use async_channel::{Receiver, Sender};
use matrix_sdk::{
    async_trait,
    ruma::RoomId,
    widget::{
        run_client_widget_api, Comm, Permissions, PermissionsProvider, Widget, WidgetSettings,
    },
    Client,
};

use crate::{send_post_message, WIDGET_ID}; // mod placeholder_matrix_sdk_widget_driver;

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
) {
    let room_id = <&RoomId>::try_from("!IOSsIxnwmlSWYotZEl:my.matrix.host").unwrap();
    println!("{}", room_id);
    let Some(room) = client.get_room(&room_id) else {panic!("could not get room")};

    tokio::spawn(async move {
        while let Ok(msg) = out_rx.recv().await {
            println!("\n## -> Outgoing msg: {}", msg);
            send_post_message(&window, &msg);
        }
    });
    // ------ Setup everything, so the driver can access the room + the call callback on the widget
    let wid = Widget {
        comm: Comm {
            from: in_rx,
            to: out_tx,
        },
        settings: WidgetSettings {
            id: WIDGET_ID.to_owned(),
            init_on_load: true,
        },
    };
    tokio::spawn(async move {
        let _ = run_client_widget_api(wid, PermProv {}, room)
            .await
            .map_err(|e| println!("run client widget api error: {}", e.to_string()));
    });
}
