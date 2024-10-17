# Matrix widget container

This is a simple app that starts the matrix rust SDK.

It will load the url defined in the command line arguments and put it into a widget environment.

This is not build to the standards of production grade software but is very usefulto test the widget driver capabilities of the matrix-rust-sdk.



## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

Tauri also needs the gtk and C build tools see: https://tauri.app/v1/guides/getting-started/prerequisites#setting-up-linux

"devPath2": "http://localhost:8000/#/?widgetId=1234-matrix_widget_id-1234&userId=1234-matrix_user_id-1234",

# Run the app

`yarn tauri dev`

using these command line parameters:

```
"widget_url",
"user_id",
"password",
"room_id_for_for_the_widget_driver"
```