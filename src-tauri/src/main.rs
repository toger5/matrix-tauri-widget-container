// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde_json::Value;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn handle_post_message(window: tauri::Window, message: &str) {
    // let s = format!("Message: {}", message);
    println!("{}\n", message);
    let msg_val: Value = serde_json::from_str(message).expect("should be json");
    // serde_json.pa
    println!("json: {}", msg_val);
    if let Value::Object(all_map) = msg_val.clone() {
        println!("json?: {:?}", all_map);
        if let Value::Object(m_data) = all_map.get("data").unwrap(){
            if !m_data.contains_key("response"){
                println!("json??: {:?}",m_data);
                let mut m = m_data.clone();
                m.insert("response".to_owned(), 
                serde_json::json!({"supported_versions":[
                    "0.0.1",
                    "0.0.2",
                    "org.matrix.msc2762",
                    "org.matrix.msc2871",
                    "org.matrix.msc2931",
                    "org.matrix.msc2974",
                    "org.matrix.msc2876",
                    "org.matrix.msc3819",
                    "town.robin.msc3846",
                    "org.matrix.msc3869",
                    "org.matrix.msc3973"]}));
                
                let m_string = Value::Object(m).to_string();
                println!("message string: {}", m_string);
                send_post_message(window, &m_string);
            }
        }
    }
}

fn send_post_message(window: tauri::Window, message: &str) {
    let script = format!("postMessage({},'*')", message);
    println!("eval js: {}", script);
    window.eval(&script);
}

const INIT_SCRIPT: &str = r#"
console.log("injecting tauri listener");
window.addEventListener(
    "message",
    (event) => {
        let message = {data: event.data, origin: event.origin}
        console.log(JSON.stringify(message))
        window.__TAURI__.tauri.invoke("handle_post_message", { message: JSON.stringify(message) });
        console.log("webapp received event:", event);
    },
    false,
  );
  console.log("done injection js");
"#;
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let window =
                tauri::WindowBuilder::new(app, "label", tauri::WindowUrl::App("index.html".into()))
                    .initialization_script(INIT_SCRIPT)
                    .build()?;
            Ok(())
        })
        .on_page_load(|window: tauri::Window, _| {
            println!("WINDOW DID LOAD");
        })
        .invoke_handler(tauri::generate_handler![handle_post_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
