use std::sync::Arc;

use matrix_sdk::async_trait;
use matrix_sdk::room::Joined;
use matrix_sdk::widget_api::run_client_widget_api;
use matrix_sdk::widget_api::{
    api::run,
    api::widget::{Comm, Widget as TransportWidget},
    matrix::Driver,
};
use matrix_sdk::widget_api::handler::Client;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender as Sender};

pub use matrix_sdk::widget_api::{
    api::widget::Info, messages::capabilities::Options, Result, PermissionProvider
};

// #[async_trait]
// pub trait Widget {
//     fn send(&self, json: &str) -> Result<()>;
//     fn id(&self) -> &str;
// }

pub struct WidgetApi {
    raw_from_tx: Sender<String>,
}
impl WidgetApi {
    pub fn new<W: Widget + PermissionProvider>(room: Joined, widget: W, info: Info) -> Self {
        let arc_widget = Arc::new(widget);
        let (raw_from_tx, raw_from_rx) = unbounded_channel::<String>();
        let (raw_to_tx, mut raw_to_rx) = unbounded_channel::<String>();

        let trasport_widget = TransportWidget {
            info,
            comm: Comm {
                from: raw_from_rx,
                to: raw_to_tx,
            },
        };
        let w = arc_widget.clone();
        tokio::spawn(async move {
            while let Some(json) = raw_to_rx.recv().await {
                let _ = w.send(json.as_str());
            }
        });

        let d = Driver::new(room, arc_widget.clone());
        tokio::spawn(async move {
            // let _ = run(d, trasport_widget).await;
            run_client_widget_api(widget,
                widget,
                room: JoinedRoom,);
        });

        WidgetApi { raw_from_tx }
    }
    pub fn handle(&self, msg: &str) -> std::result::Result<(), SendError<String>> {
        Ok(self.raw_from_tx.send(msg.to_owned())?)
    }
}
