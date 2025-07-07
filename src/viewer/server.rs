// ====================
// Viewer modules.
// ====================
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

// ====================
// Deps
// ====================
use bevy::app::AppExit;
use bevy::prelude::*;
#[cfg(not(feature = "jarl"))]
use bevy_hanabi::EffectAsset;
// ====================
// Particles.
// ====================
#[cfg(feature = "jarl")]
use jarl_particles::EffectAsset;
use kanal::{Receiver, Sender};

// ====================
// Shared.
// ====================
use crate::shared::proto::{ServerCommandMessage, ServerCommandResponse};

#[derive(Event)]
pub enum ViewerCommandEvent
{
    OpenAsset
    {
        asset: EffectAsset
    },
    OpenAssetFile
    {
        path: String
    },
}

/// Bevy resource that implements communication with the server.
#[derive(Resource)]
pub struct ViewerServerState
{
    pub receiver:       Receiver<ServerCommandMessage>,
    pub is_running:     Arc<AtomicBool>,
    pub channel_closed: bool,
}

/// System that spawns the server thread.
pub fn sys_start_server(mut cmds: Commands)
{
    debug!("Starting server system...");
    let (sender, receiver) = kanal::unbounded();
    let is_running = Arc::new(AtomicBool::new(true));
    let is_running_clone = is_running.clone();
    let builder = thread::Builder::new().name("server-thread".to_string());

    match builder.spawn(move || {
        debug!("Server thread spawned, entering main loop");
        server_thread_main(sender, is_running_clone);
    }) {
        Ok(_) => info!("Server thread started successfully"),
        Err(e) => {
            error!("Failed to spawn server thread: {}", e);
            return;
        }
    }

    cmds.insert_resource(ViewerServerState {
        receiver,
        is_running,
        channel_closed: false,
    });
}

/// System that handles communication with the server for each tick
/// and transforms server messages into Bevy events.
///
pub fn sys_handle_server_messages(
    mut server_state: ResMut<ViewerServerState>,
    mut event_writer: EventWriter<ViewerCommandEvent>,
)
{
    if server_state.channel_closed {
        return;
    }

    match server_state.receiver.try_recv_realtime() {
        Ok(Some(msg)) => match msg {
            ServerCommandMessage::OpenAsset { asset } => {
                debug!("Received OpenAsset command: {}", asset.name);
                event_writer.send(ViewerCommandEvent::OpenAsset { asset });
            }
            ServerCommandMessage::OpenAssetFile { path } => {
                debug!("Received OpenAssetFile command: {}", path);
                event_writer.send(ViewerCommandEvent::OpenAssetFile { path });
            }
            ServerCommandMessage::Ping => {
                debug!("Received Ping command");
            }
        },
        Ok(None) => (),
        Err(_) => {
            if !server_state.channel_closed {
                info!("Server channel closed, stopping message processing");
                server_state.channel_closed = true;
            }
        }
    }
}

/// Basic server loop.
///
fn server_thread_main(sender: Sender<ServerCommandMessage>, is_running: Arc<AtomicBool>)
{
    info!(
        "Server thread main started, is_running: {}",
        is_running.load(Ordering::Relaxed)
    );
    let ports = crate::common::SERVER_PORTS;
    let listener = match try_bind_ports(&ports) {
        Some((listener, port)) => {
            println!("=== PARTICLE EDITOR SERVER STARTED ===");
            println!("Server listening on 127.0.0.1:{}", port);
            println!("Socket: 127.0.0.1:{}", port);
            println!(
                "Test with: echo '{{\"OpenAssetFile\":{{\"path\":\"path/to/effect.ron\"}}}}' | nc 127.0.0.1 {}",
                port
            );
            println!("=======================================");
            listener
        }

        None => {
            error!("Failed to bind server socket on any port: {:?}", ports);
            is_running.store(false, Ordering::Relaxed);
            return;
        }
    };

    if let Err(e) = listener.set_nonblocking(true) {
        error!("Failed to set listener to non-blocking: {}", e);
        is_running.store(false, Ordering::Relaxed);
        return;
    }

    info!("Server ready to accept connections, entering main loop");

    while is_running.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, addr)) => {
                debug!("New client connected: {}", addr);
                let sender_clone = sender.clone();
                let builder = thread::Builder::new().name(format!("client-{}", addr));

                match builder.spawn(move || {
                    handle_client(stream, sender_clone);
                }) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Failed to spawn client handler thread: {}", e);
                    }
                }
            }

            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(std::time::Duration::from_millis(100));
            }

            Err(e) => {
                error!("Error accepting connection: {}", e);
                break;
            }
        }
    }

    info!(
        "Server thread shutting down, final is_running: {}",
        is_running.load(Ordering::Relaxed),
    );
}

fn try_bind_ports(ports: &[u16]) -> Option<(TcpListener, u16)>
{
    for &port in ports {
        let addr = format!("127.0.0.1:{}", port);
        match TcpListener::bind(&addr) {
            Ok(listener) => {
                return Some((listener, port));
            }
            Err(e) => {
                warn!("Failed to bind to {}: {}", addr, e);
            }
        }
    }
    None
}

fn handle_client(stream: TcpStream, sender: Sender<ServerCommandMessage>)
{
    let peer_addr = match stream.peer_addr() {
        Ok(addr) => addr.to_string(),
        Err(_) => "unknown".to_string(),
    };

    debug!("Handling client: {}", peer_addr);
    if let Err(e) = stream.set_read_timeout(Some(std::time::Duration::from_secs(5))) {
        error!("Failed to set read timeout for client {}: {}", peer_addr, e);
        return;
    }

    let mut write_stream = match stream.try_clone() {
        Ok(stream) => stream,
        Err(e) => {
            error!("Failed to clone stream for client {}: {}", peer_addr, e);
            return;
        }
    };

    let mut reader = BufReader::new(&stream);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => {
                info!("Client {} disconnected", peer_addr);
                break;
            }
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                match serde_json::from_str::<ServerCommandMessage>(trimmed) {
                    Ok(message) => {
                        info!("Received message from {}", peer_addr);

                        if let Err(e) = sender.send(message.clone()) {
                            error!("Failed to send message to main thread: {}", e);
                            break;
                        }

                        let response = ServerCommandResponse::Ok;

                        if let Ok(response_json) = serde_json::to_string(&response) {
                            if let Err(e) = writeln!(write_stream, "{}", response_json) {
                                error!("Failed to send response to client {}: {}", peer_addr, e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Invalid JSON from client {}: {} - JSON content: '{}'",
                            peer_addr, e, trimmed
                        );

                        let response = ServerCommandResponse::Error {
                            message: format!("Invalid JSON: {}", e),
                        };

                        if let Ok(response_json) = serde_json::to_string(&response) {
                            let _ = writeln!(write_stream, "{}", response_json);
                        }
                    }
                }
            }
            Err(e) => {
                trace!("Error reading from client {}: {}", peer_addr, e);
                break;
            }
        }
    }

    info!("Client handler for {} terminated", peer_addr);
}

pub fn sys_cleanup_server(
    mut app_exit_events: EventReader<AppExit>,
    server_state: Option<ResMut<ViewerServerState>>,
)
{
    if !app_exit_events.is_empty() {
        app_exit_events.clear();
        if let Some(state) = server_state {
            let was_running = state.is_running.load(Ordering::Relaxed);
            info!("App exiting, server was_running: {}", was_running);
            if was_running {
                state.is_running.store(false, Ordering::Relaxed);
                info!("Server shutdown requested");
            }
        }
    }
}

pub struct ServerPlugin;

impl Plugin for ServerPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_event::<ViewerCommandEvent>()
            .add_systems(Startup, sys_start_server)
            .add_systems(Update, (sys_handle_server_messages, sys_cleanup_server));
    }
}
