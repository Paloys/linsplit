use std::{collections::VecDeque, fs::read, sync::Arc};

use tokio::sync::{Mutex, Notify};

use anyhow::Result;
use futures_util::{
    SinkExt, TryStreamExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{WebSocketStream, tungstenite::protocol::Message};

use crate::livesplitone::commands::{Command, CommandError, CommandResult, Event, Response};

pub struct SplitterSocket {
    outcoming: Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>,
    incoming: Mutex<SplitStream<WebSocketStream<TcpStream>>>,
    responses: Mutex<VecDeque<CommandResult<Response, CommandError>>>,
    response_notification: Notify,
    events: Arc<Mutex<VecDeque<Event>>>,
    event_notifications: Arc<Notify>,
}

impl SplitterSocket {
    pub async fn new(
        addr: &str,
        events: Arc<Mutex<VecDeque<Event>>>,
        event_notifications: Arc<Notify>,
    ) -> Result<Arc<Self>> {
        let socket: TcpListener = TcpListener::bind(&addr).await?;
        println!("Waiting for LiveSplitOne Connection...");
        println!("Enter ws://{addr} in the LiveSplitOne \"Server Connection\" setting");
        if let Ok((stream, _addr)) = socket.accept().await {
            let ws_stream = tokio_tungstenite::accept_async(stream)
                .await
                .expect("websocket failed");
            let (outcoming, incoming) = ws_stream.split();
            println!("Connected to LiveSplitOne");
            let sock = Arc::new(SplitterSocket {
                outcoming: Mutex::new(outcoming),
                incoming: Mutex::new(incoming),
                responses: Default::default(),
                response_notification: Notify::new(),
                event_notifications,
                events,
            });
            let reader = Arc::clone(&sock);
            tokio::spawn(async move { reader.listener_loop().await });
            return Ok(sock);
        }
        Err(anyhow::anyhow!("Failed to start socket"))
    }

    pub async fn send_command(
        &self,
        command: Command,
    ) -> Result<Option<CommandResult<Response, CommandError>>> {
        self.outcoming
            .lock()
            .await
            .send(Message::text(serde_json::to_string(&command)?))
            .await?;

        // Wait for the response
        self.response_notification.notified().await;
        let response = self.responses.lock().await.pop_front();
        Ok(response)
    }

    async fn listener_loop(self: Arc<Self>) {
        loop {
            if let Ok(Some(Message::Text(message))) = self.incoming.lock().await.try_next().await {
                if let Ok(response) =
                    serde_json::from_str::<CommandResult<Response, CommandError>>(&message)
                {
                    self.responses.lock().await.push_back(response);
                    self.response_notification.notify_one();
                } else if let Ok(response) = serde_json::from_str::<Event>(&message) {
                    self.events.lock().await.push_back(response);
                    self.event_notifications.notify_one();
                }
            }
        }
    }
}
