use std::result::Result::Ok;

use anyhow::Result;
use futures_util::{
    SinkExt, TryStreamExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{WebSocketStream, tungstenite::protocol::Message};

use crate::livesplitone::commands::{Command, CommandError, CommandResult, Event, Response};

pub struct SplitterSocket {
    outcoming: SplitSink<WebSocketStream<TcpStream>, Message>,
    incoming: SplitStream<WebSocketStream<TcpStream>>,
}

impl SplitterSocket {
    pub async fn new(addr: &str) -> Result<Self> {
        let socket: TcpListener = TcpListener::bind(&addr).await?;
        println!("Waiting for LiveSplitOne Connection...");
        if let Ok((stream, _addr)) = socket.accept().await {
            let ws_stream = tokio_tungstenite::accept_async(stream)
                .await
                .expect("websocket failed");
            let (outcoming, incoming) = ws_stream.split();
            println!("Connected to LiveSplitOne");
            return Ok(SplitterSocket {
                outcoming,
                incoming,
            });
        }
        Err(anyhow::anyhow!("Failed to start socket"))
    }

    pub async fn send_command(
        &mut self,
        command: Command,
    ) -> Result<Option<CommandResult<Response, CommandError>>> {
        self.outcoming
            .send(Message::text(serde_json::to_string(&command)?))
            .await?;
        let mut msg = self
            .incoming
            .try_next()
            .await?
            .ok_or(anyhow::anyhow!("failed to get response"))?;
        let mut response =
            serde_json::from_str::<CommandResult<Response, CommandError>>(msg.to_text()?);
        match response {
            Ok(result) => return Ok(Some(result)),
            Err(_) => {
                if let Ok(_) =
                    serde_json::from_str::<Event>(msg.to_text()?)
                {
                    // Hack in case we recieved an event
                    msg = self
                        .incoming
                        .try_next()
                        .await?
                        .ok_or(anyhow::anyhow!("failed to get response"))?;
                    response = serde_json::from_str::<CommandResult<Response, CommandError>>(
                        msg.to_text()?,
                    );
                } else {
                    return Err(anyhow::anyhow!("Recieved weird response"));
                }
            }
        };
        Ok(Some(response?))
    }
}
