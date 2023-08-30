pub const INIT_SCRIPT: &str = r#"
console.log("injecting tauri listener");
window.addEventListener(
    "message",
    (event) => {
        
        let message = {data: event.data, origin: event.origin}
        if (message.data.response && message.data.api == "toWidget" 
        || !message.data.response && message.data.api == "fromWidget") {
          window.__TAURI__.tauri.invoke("handle_post_message", { message: JSON.stringify(message.data) });
        }else{
          console.log("-- skipped event handling by the client because it is send from the client itself.");
        }
    },
    false,
  );
  console.log("done injection js");
"#;