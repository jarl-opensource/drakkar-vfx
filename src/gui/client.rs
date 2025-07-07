use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::{Duration, Instant};

#[cfg(not(feature = "jarl"))]
use bevy_hanabi::EffectAsset;
// ====================
// Particles.
// ====================
#[cfg(feature = "jarl")]
use jarl_particles::EffectAsset;
// ====================
// Deps.
// ====================
use kanal::{Receiver, Sender, unbounded};
use tracing::{debug, error, warn};

// ====================
// Shared.
// ====================
use crate::shared::proto::ServerCommandMessage;

// Internal message types for the background thread
enum ClientMessage
{
    SendMessage(ServerCommandMessage),
    SendRaw(String),
    SendAsset(EffectAsset),
    Shutdown,
}

pub struct ViewerSyncClient
{
    sender:           Sender<ClientMessage>,
    _shutdown_sender: Sender<ClientMessage>, // Keep to prevent shutdown on drop
}

impl ViewerSyncClient
{
    pub fn new() -> Self
    {
        Self::with_address("127.0.0.1", 8080)
    }

    pub fn with_address(address: impl Into<String>, port: u16) -> Self
    {
        let (sender, receiver) = unbounded::<ClientMessage>();
        let shutdown_sender = sender.clone();

        let address = address.into();

        // Spawn background task
        std::thread::spawn(move || {
            let mut worker = ClientWorker::new(address, port, receiver);
            worker.run();
        });

        Self {
            sender,
            _shutdown_sender: shutdown_sender,
        }
    }

    pub fn send_message(&self, message: &ServerCommandMessage)
    {
        if let Err(e) = self
            .sender
            .send(ClientMessage::SendMessage(message.clone()))
        {
            error!("Failed to send message to background thread: {}", e);
        }
    }

    pub fn send_raw(&self, message: &str)
    {
        if let Err(e) = self
            .sender
            .send(ClientMessage::SendRaw(message.to_string()))
        {
            error!("Failed to send raw message to background thread: {}", e);
        }
    }

    pub fn send_open_asset_file(&self, file_path: &Path)
    {
        let path = file_path.to_string_lossy().to_string();
        self.send_message(&ServerCommandMessage::OpenAssetFile { path });
    }

    pub fn send_open_asset(&self, asset: EffectAsset)
    {
        if let Err(e) = self.sender.send(ClientMessage::SendAsset(asset)) {
            error!("Failed to send asset to background thread: {}", e);
        }
    }

    pub fn ping(&self)
    {
        self.send_message(&ServerCommandMessage::Ping);
    }
}

impl Default for ViewerSyncClient
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl Drop for ViewerSyncClient
{
    fn drop(&mut self)
    {
        let _ = self.sender.send(ClientMessage::Shutdown);
    }
}

// Background worker that handles all network communication
struct ClientWorker
{
    address:           String,
    port:              u16,
    receiver:          Receiver<ClientMessage>,
    // Delay queue state
    pending_asset:     Option<EffectAsset>,
    asset_delay_timer: Option<Instant>,
    asset_delay:       Duration,
}

impl ClientWorker
{
    fn new(address: String, port: u16, receiver: Receiver<ClientMessage>) -> Self
    {
        Self {
            address,
            port,
            receiver,
            pending_asset: None,
            asset_delay_timer: None,
            asset_delay: Duration::from_millis(150), // 150ms delay for batching
        }
    }

    fn run(&mut self)
    {
        debug!("ViewerSyncClient background worker started");

        loop {
            if let Some(timer) = self.asset_delay_timer {
                if timer.elapsed() >= self.asset_delay {
                    self.flush_pending_asset();
                }
            }

            match self.receiver.try_recv() {
                Ok(Some(ClientMessage::SendMessage(message))) => {
                    self.send_message_impl(&message);
                }
                Ok(Some(ClientMessage::SendRaw(message))) => {
                    self.send_raw_impl(&message);
                }
                Ok(Some(ClientMessage::SendAsset(asset))) => {
                    self.queue_asset(asset);
                }
                Ok(Some(ClientMessage::Shutdown)) => {
                    debug!("ViewerSyncClient background worker shutting down");
                    self.flush_pending_asset();
                    break;
                }
                Ok(None) => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(_) => {
                    debug!("ViewerSyncClient receiver closed, shutting down");
                    break;
                }
            }
        }
    }

    fn queue_asset(&mut self, asset: EffectAsset)
    {
        self.pending_asset = Some(asset);
        self.asset_delay_timer = Some(Instant::now());
        debug!("Queued asset update, delay timer reset");
    }

    fn flush_pending_asset(&mut self)
    {
        if let Some(asset) = self.pending_asset.take() {
            debug!("Flushing pending asset update");
            self.send_message_impl(&ServerCommandMessage::OpenAsset { asset });
            self.asset_delay_timer = None;
        }
    }

    fn send_message_impl(&self, message: &ServerCommandMessage)
    {
        match serde_json::to_string(message) {
            Ok(json) => self.send_raw_impl(&json),
            Err(e) => error!("Failed to serialize message: {}", e),
        }
    }

    fn send_raw_impl(&self, message: &str)
    {
        let address = format!("{}:{}", self.address, self.port);

        match self.try_send_raw(message, &address) {
            Ok(()) => {
                debug!("Successfully sent message to viewer");
            }
            Err(e) => {
                warn!("Failed to send message to viewer: {}", e);
            }
        }
    }

    fn try_send_raw(&self, message: &str, address: &str) -> Result<(), Box<dyn std::error::Error>>
    {
        let mut stream = TcpStream::connect_timeout(&address.parse()?, Duration::from_secs(5))?;

        stream.write_all(message.as_bytes())?;
        stream.write_all(b"\n")?;
        stream.flush()?;

        let mut reader = BufReader::new(&stream);
        let mut response = String::new();

        match reader.read_line(&mut response) {
            Ok(0) => {
                debug!("Server closed connection");
            }
            Ok(_) => {
                debug!("Server response: {}", response.trim());
            }
            Err(e) => {
                warn!("Failed to read server response: {}", e);
            }
        }

        Ok(())
    }
}
