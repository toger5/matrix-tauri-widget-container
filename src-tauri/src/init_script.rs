pub const INIT_SCRIPT: &str = r#"
console.log("injecting tauri listener");
window.addEventListener(
    "message",
    (event) => {
        let message = {data: event.data, origin: event.origin}
        window.__TAURI__.tauri.invoke("handle_post_message", { message: JSON.stringify(message.data) });
    },
    false,
  );
  console.log("done injection js");
"#;