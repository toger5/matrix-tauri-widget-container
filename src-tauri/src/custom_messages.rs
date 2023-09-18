use serde::de::Error;
use serde_json::{from_str, json, to_string, Result, Value};
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
pub(crate) fn add_response(msg: &str, res_val: Value) -> Result<String> {
    let mut value = from_str::<Value>(msg)?;
    value
        .as_object_mut()
        .ok_or(Error::custom("msg"))?
        .insert("response".to_string(), res_val);
    to_string(&value)
}
