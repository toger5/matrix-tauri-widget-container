use serde_json::json;
use uuid::Uuid;

pub(crate) fn join_action(widget_id: &str) -> String {
    let msg = json!({
        "api": "toWidget",
        "requestId": Uuid::new_v4().to_string(),
        "action": "io.element.join",
        "widgetId": widget_id,
        "data":{
            "audioInput": "",
            "videoInput": "",
        }
    });
    msg.to_string()
}
