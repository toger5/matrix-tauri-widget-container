use crate::custom_messages::_add_response;
use clone_all::clone_all;
use matrix_sdk::{
    async_trait,
    config::SyncSettings,
    ruma::events::StateEventType,
    widget::{
        Capabilities, CapabilitiesProvider, EventFilter, MessageLikeEventFilter, StateEventFilter,
        WidgetDriver, WidgetDriverHandle, WidgetSettings,
    },
    Client, Room,
};
use serde_json::Value;
use url::Url;

// This is also part of the rust sdk FFI but since it is element call specific it is
// only available in the FFI. Hence, we need to reimplement it here.
// This is suboptimal since it cannot be guaranteed, that we use the same version as the rust sdk.
pub fn get_element_call_required_permissions(
    own_user_id: String,
    own_device_id: String,
) -> Capabilities {
    Capabilities {
        read: vec![
            // This is required for legacy state events (using one event and a membership array)
            // TODO: remove once legacy call members are sunset
            EventFilter::State(StateEventFilter::WithType(StateEventType::CallMember)),
            // To detect leaving/kicked room members during a call.
            EventFilter::State(StateEventFilter::WithType(StateEventType::RoomMember)),
            // To decide whether to encrypt the call streams based on the room encryption setting.
            EventFilter::State(StateEventFilter::WithType(StateEventType::RoomEncryption)),
            // To read rageshake requests from other room members
            EventFilter::MessageLike(MessageLikeEventFilter::WithType(
                "org.matrix.rageshake_request".into(),
            )),
            // To read encryption keys
            // TODO change this to the appropriate to-device version once ready
            EventFilter::MessageLike(MessageLikeEventFilter::WithType(
                "io.element.call.encryption_keys".into(),
            )),
            // This allows the widget to check the room version, so it can know about
            // version-specific auth rules (namely MSC3779).
            EventFilter::State(StateEventFilter::WithType(StateEventType::RoomCreate)),
        ],
        send: vec![
            // To send the call participation state event (main MatrixRTC event)
            EventFilter::State(StateEventFilter::WithTypeAndStateKey(
                StateEventType::CallMember,
                own_user_id.clone(),
            )),
            // [MSC3779](https://github.com/matrix-org/matrix-spec-proposals/pull/3779) version, with no leading underscore
            EventFilter::State(StateEventFilter::WithTypeAndStateKey(
                StateEventType::CallMember,
                format!("{}_{}", own_user_id, own_device_id),
            )),
            // The same as above but with an underscore
            EventFilter::State(StateEventFilter::WithTypeAndStateKey(
                StateEventType::CallMember,
                format!("_{}_{}", own_user_id, own_device_id),
            )),
            // To request other room members to send rageshakes
            EventFilter::MessageLike(MessageLikeEventFilter::WithType(
                "org.matrix.rageshake_request".into(),
            )),
            // To send this user's encryption keys
            EventFilter::MessageLike(MessageLikeEventFilter::WithType(
                "io.element.call.encryption_keys".into(),
            )),
        ],
        requires_client: true,
        update_delayed_event: true,
        send_delayed_event: true,
    }
}

struct CapProv {
    own_user_id: String,
    own_device_id: String,
}
#[async_trait]
impl CapabilitiesProvider for CapProv {
    async fn acquire_capabilities(&self, cap: Capabilities) -> Capabilities {
        let approved = get_element_call_required_permissions(
            self.own_user_id.clone(),
            self.own_device_id.clone(),
        );
        print!("Requested \n {:#?}\n\n Approved\n{:#?}", cap, approved);
        approved
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

    println!(
        "\n## <- Incoming msg: {:#?}",
        serde_json::from_str::<Value>(message)
    );
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

    let cap_provider = CapProv {
        own_device_id: client.device_id().unwrap().to_string(),
        own_user_id: client.user_id().unwrap().to_string(),
    };

    let url = widget_settings.base_url().unwrap().to_string();
    tokio::spawn(async move {
        while let Some(msg) = handle.recv().await {
            send_post_message(&window, &msg, &url);
        }
    });

    tokio::spawn(async {
        let _ = driver.run(room, cap_provider).await;
    });

    let client = client.clone();
    tokio::spawn(async move {
        let sync_result = client.sync(SyncSettings::default()).await;
        println!("{:?}", sync_result);
    });
}
