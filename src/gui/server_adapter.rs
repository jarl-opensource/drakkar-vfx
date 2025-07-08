// ====================
// Std.
// ====================
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::thread;

// ====================
// Deps.
// ====================
use kanal::{Receiver, Sender, unbounded};
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone)]
pub enum ServerMessage
{
    Input(String),
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum ServerResponse
{
    Output(String),
    Error(String),
    Exited(i32),
}

pub struct ViewerServerAdapter
{
    process:         Option<Child>,
    input_sender:    Option<Sender<ServerMessage>>,
    output_receiver: Option<Receiver<ServerResponse>>,
    _handles:        Vec<thread::JoinHandle<()>>,
}

impl ViewerServerAdapter
{
    pub fn new() -> Self
    {
        Self {
            process:         None,
            input_sender:    None,
            output_receiver: None,
            _handles:        Vec::new(),
        }
    }

    pub fn start(&mut self) -> Result<(), String>
    {
        if self.process.is_some() {
            return Err("Server is already running".to_string());
        }
        info!("Starting viewer server process...");

        // TODO: Make this configurable.
        let viewer_path = if std::path::Path::new("target/release/drakkar-vfx-viewer").exists() {
            "target/release/drakkar-vfx-viewer"
        } else {
            "target/debug/drakkar-vfx-viewer"
        };

        let mut cmd = Command::new(viewer_path);
        cmd.env("RUST_LOG", "warn")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            error!("Failed to spawn viewer process: {}", e);
            format!("Failed to spawn viewer process: {}", e)
        })?;

        let stdin = child.stdin.take().ok_or("Failed to get stdin handle")?;
        let stdout = child.stdout.take().ok_or("Failed to get stdout handle")?;
        let stderr = child.stderr.take().ok_or("Failed to get stderr handle")?;

        let (input_sender, input_receiver) = unbounded::<ServerMessage>();
        let (output_sender, output_receiver) = unbounded::<ServerResponse>();

        let mut handles = Vec::new();

        let stdin_handle = {
            let input_receiver = input_receiver.clone();
            let mut stdin = stdin;
            thread::spawn(move || {
                loop {
                    match input_receiver.recv() {
                        Ok(ServerMessage::Input(line)) => {
                            debug!("Sending to server: {}", line);
                            if let Err(e) = writeln!(stdin, "{}", line) {
                                error!("Failed to write to server stdin: {}", e);
                                break;
                            }
                            if let Err(e) = stdin.flush() {
                                error!("Failed to flush server stdin: {}", e);
                                break;
                            }
                        }
                        Ok(ServerMessage::Shutdown) => {
                            debug!("Received shutdown signal for stdin thread");
                            break;
                        }
                        Err(e) => {
                            debug!("Input receiver closed: {}", e);
                            break;
                        }
                    }
                }
                debug!("Stdin thread exiting");
            })
        };
        handles.push(stdin_handle);

        let stdout_handle = {
            let output_sender = output_sender.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            debug!("Received from server stdout: {}", line);
                            if output_sender.send(ServerResponse::Output(line)).is_err() {
                                debug!("Output sender closed, stopping stdout thread");
                                break;
                            }
                        }
                        Err(e) => {
                            debug!("Error reading server stdout: {}", e);
                            let _ = output_sender
                                .send(ServerResponse::Error(format!("Stdout error: {}", e)));
                            break;
                        }
                    }
                }
                debug!("Stdout thread exiting");
            })
        };
        handles.push(stdout_handle);

        let stderr_handle = {
            let output_sender = output_sender.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            eprintln!("{}", line);
                            if output_sender.send(ServerResponse::Error(line)).is_err() {
                                debug!("Output sender closed, stopping stderr thread");
                                break;
                            }
                        }
                        Err(e) => {
                            debug!("Error reading server stderr: {}", e);
                            break;
                        }
                    }
                }
                debug!("Stderr thread exiting");
            })
        };
        handles.push(stderr_handle);

        let process_monitor = {
            let process_id = child.id();
            thread::spawn(move || {
                debug!("Process monitor thread started for PID: {}", process_id);
            })
        };
        handles.push(process_monitor);

        self.process = Some(child);
        self.input_sender = Some(input_sender);
        self.output_receiver = Some(output_receiver);
        self._handles = handles;

        info!("Viewer server process started successfully");
        Ok(())
    }

    pub fn send_input(&self, input: String) -> Result<(), String>
    {
        if let Some(sender) = &self.input_sender {
            sender.send(ServerMessage::Input(input)).map_err(|e| {
                error!("Failed to send input to server: {}", e);
                format!("Failed to send input: {}", e)
            })
        } else {
            Err("Server is not running".to_string())
        }
    }

    pub fn try_recv_output(&self) -> Option<ServerResponse>
    {
        if let Some(receiver) = &self.output_receiver {
            receiver.try_recv().unwrap_or_else(|_| None)
        } else {
            None
        }
    }

    pub fn drain_all_messages(&mut self)
    {
        while let Some(_) = self.try_recv_output() {
            // ok
        }
    }

    pub fn is_running(&mut self) -> bool
    {
        if let Some(process) = &mut self.process {
            match process.try_wait() {
                Ok(Some(status)) => {
                    info!("Server process exited with status: {:?}", status);
                    if let Some(receiver) = &self.output_receiver {
                        while receiver.try_recv().is_ok() {}
                    }
                    false
                }
                Ok(None) => true,
                Err(e) => {
                    error!("Error checking process status: {}", e);
                    false
                }
            }
        } else {
            false
        }
    }

    pub fn stop(&mut self) -> Result<(), String>
    {
        if let Some(sender) = &self.input_sender {
            let _ = sender.send(ServerMessage::Shutdown);
        }

        if let Some(mut process) = self.process.take() {
            info!("Stopping viewer server process...");

            if let Err(e) = process.terminate() {
                warn!("Failed to terminate process gracefully: {}", e);
            }

            match process.wait() {
                Ok(status) => {
                    info!("Server process exited with status: {:?}", status);
                }
                Err(e) => {
                    error!("Error waiting for process to exit: {}", e);
                    if let Err(e) = process.kill() {
                        error!("Failed to kill process: {}", e);
                    }
                }
            }
        }

        self.input_sender = None;
        self.output_receiver = None;

        info!("Server stopped successfully");
        Ok(())
    }
}

impl Drop for ViewerServerAdapter
{
    fn drop(&mut self)
    {
        debug!("ServerWrapper dropping, cleaning up...");
        if let Err(e) = self.stop() {
            error!("Error during ServerWrapper drop: {}", e);
        }
    }
}

trait ProcessExt
{
    fn terminate(&mut self) -> std::io::Result<()>;
}

impl ProcessExt for Child
{
    fn terminate(&mut self) -> std::io::Result<()>
    {
        #[cfg(unix)]
        {
            unsafe {
                libc::kill(self.id() as i32, libc::SIGTERM);
            }
            Ok(())
        }

        #[cfg(windows)]
        {
            self.kill()
        }
    }
}
