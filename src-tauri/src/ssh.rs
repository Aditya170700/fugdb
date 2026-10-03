use std::net::{TcpListener as StdTcpListener, TcpStream as StdTcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::sync::oneshot;

use crate::error::AppError;
use crate::models::connection::ConnectionConfig;

pub struct SshTunnelHandle {
    pub local_port: u16,
    pub shutdown_tx: Option<oneshot::Sender<()>>,
}

impl SshTunnelHandle {
    pub fn close(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

impl Drop for SshTunnelHandle {
    fn drop(&mut self) {
        self.close();
    }
}

pub struct SshTunnelManager;

impl SshTunnelManager {
    /// Expand tilde `~` to the user's home directory if present
    pub fn expand_home_dir(path_str: &str) -> PathBuf {
        if path_str.starts_with("~/") || path_str == "~" {
            if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
                let mut path = PathBuf::from(home);
                if path_str.len() > 2 {
                    path.push(&path_str[2..]);
                }
                return path;
            }
        }
        PathBuf::from(path_str)
    }

    /// Establish an authenticated SSH2 session
    pub fn create_session(config: &ConnectionConfig) -> Result<ssh2::Session, AppError> {
        let ssh_host = config.ssh_host.as_deref().unwrap_or("").trim();
        if ssh_host.is_empty() {
            return Err(AppError::ConnectionError("SSH Host is required when SSH Tunnel is enabled".into()));
        }
        let ssh_port = config.ssh_port.unwrap_or(22);
        let ssh_user = config.ssh_user.as_deref().unwrap_or("").trim();
        if ssh_user.is_empty() {
            return Err(AppError::ConnectionError("SSH Username is required".into()));
        }

        let addr = format!("{}:{}", ssh_host, ssh_port);
        let mut addrs = addr.to_socket_addrs()
            .map_err(|e| AppError::ConnectionError(format!("Could not resolve SSH hostname {}: {}", ssh_host, e)))?;
        let socket_addr = addrs.next()
            .ok_or_else(|| AppError::ConnectionError(format!("Could not resolve SSH hostname: {}", ssh_host)))?;

        let tcp = StdTcpStream::connect_timeout(&socket_addr, Duration::from_secs(10))
            .map_err(|e| AppError::ConnectionError(format!("Failed to connect to SSH host {}: {}", addr, e)))?;

        let mut session = ssh2::Session::new()
            .map_err(|e| AppError::ConnectionError(format!("Failed to initialize SSH session: {}", e)))?;
        
        session.set_tcp_stream(tcp);
        session.set_timeout(10000); // 10s timeout
        session.handshake()
            .map_err(|e| AppError::ConnectionError(format!("SSH handshake failed for {}:{}: {}", ssh_host, ssh_port, e)))?;

        let auth_type = config.ssh_auth_type.as_deref().unwrap_or("key");

        // 1. Password authentication
        if auth_type == "password" || (auth_type != "agent" && config.ssh_password.is_some() && !config.ssh_password.as_deref().unwrap_or("").is_empty()) {
            if let Some(ref pwd) = config.ssh_password {
                session.userauth_password(ssh_user, pwd)
                    .map_err(|e| AppError::ConnectionError(format!("SSH password authentication failed for user '{}': {}", ssh_user, e)))?;
            }
        } 
        // 2. SSH Agent authentication
        else if auth_type == "agent" {
            let mut agent = session.agent()
                .map_err(|e| AppError::ConnectionError(format!("Failed to initialize SSH Agent client: {}", e)))?;
            agent.connect()
                .map_err(|e| AppError::ConnectionError(format!("Failed to connect to local SSH Agent socket: {}", e)))?;
            agent.list_identities()
                .map_err(|e| AppError::ConnectionError(format!("Failed to list identities from SSH Agent: {}", e)))?;
            
            let identities = agent.identities()
                .map_err(|e| AppError::ConnectionError(format!("Failed to read SSH Agent identities: {}", e)))?;
            
            let mut authenticated = false;
            for identity in identities {
                if agent.userauth(ssh_user, &identity).is_ok() {
                    authenticated = true;
                    break;
                }
            }
            if !authenticated {
                return Err(AppError::ConnectionError(format!("SSH Agent did not have a valid key for user '{}'", ssh_user)));
            }
        } 
        // 3. Private Key File authentication
        else {
            let key_path_str = config.ssh_key_path.as_deref().unwrap_or("~/.ssh/id_rsa");
            let key_path = Self::expand_home_dir(key_path_str);
            
            if !key_path.exists() {
                // Try fallback keys if default doesn't exist
                let fallback_ed25519 = Self::expand_home_dir("~/.ssh/id_ed25519");
                let fallback_ecdsa = Self::expand_home_dir("~/.ssh/id_ecdsa");
                
                let chosen_path = if key_path.exists() {
                    key_path
                } else if fallback_ed25519.exists() {
                    fallback_ed25519
                } else if fallback_ecdsa.exists() {
                    fallback_ecdsa
                } else {
                    return Err(AppError::ConnectionError(format!("SSH Private Key file not found at '{}'", key_path_str)));
                };

                let passphrase = config.ssh_key_passphrase.as_deref();
                session.userauth_pubkey_file(ssh_user, None, &chosen_path, passphrase)
                    .map_err(|e| AppError::ConnectionError(format!("SSH key authentication failed with key '{}': {}", chosen_path.display(), e)))?;
            } else {
                let passphrase = config.ssh_key_passphrase.as_deref();
                session.userauth_pubkey_file(ssh_user, None, &key_path, passphrase)
                    .map_err(|e| AppError::ConnectionError(format!("SSH key authentication failed with key '{}': {}", key_path.display(), e)))?;
            }
        }

        if !session.authenticated() {
            return Err(AppError::ConnectionError(format!("SSH authentication failed for user '{}' on {}:{}", ssh_user, ssh_host, ssh_port)));
        }

        Ok(session)
    }

    /// Start a local port forwarding tunnel to remote target (host:port) via SSH Bastion
    pub async fn start_tunnel(config: &ConnectionConfig) -> Result<SshTunnelHandle, AppError> {
        let remote_host = config.host.as_deref().unwrap_or("127.0.0.1").to_string();
        let remote_port = config.port.unwrap_or(5432);

        // Bind local OS loopback listener on port 0 to get an ephemeral free port
        let listener = StdTcpListener::bind("127.0.0.1:0")
            .map_err(|e| AppError::ConnectionError(format!("Failed to bind local loopback port for SSH tunnel: {}", e)))?;
        
        let local_port = listener.local_addr()
            .map_err(|e| AppError::ConnectionError(format!("Failed to get local port: {}", e)))?
            .port();

        // Set nonblocking to allow timeout checking for shutdown
        listener.set_nonblocking(true)
            .map_err(|e| AppError::ConnectionError(format!("Failed to set listener non-blocking: {}", e)))?;

        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

        // Create initial SSH session to verify connection
        let session = Self::create_session(config)?;
        let session_arc = Arc::new(Mutex::new(session));

        // Spawn a background listener thread to accept database driver connections
        let remote_host_clone = remote_host.clone();
        let config_clone = config.clone();

        thread::spawn(move || {
            loop {
                // Check if shutdown was signaled
                match shutdown_rx.try_recv() {
                    Ok(_) | Err(oneshot::error::TryRecvError::Closed) => {
                        break;
                    }
                    Err(oneshot::error::TryRecvError::Empty) => {}
                }

                match listener.accept() {
                    Ok((mut local_stream, _)) => {
                        let sess = Arc::clone(&session_arc);
                        let r_host = remote_host_clone.clone();
                        let cfg = config_clone.clone();

                        thread::spawn(move || {
                            let _ = local_stream.set_nonblocking(false);
                            
                            // Open a direct-tcpip channel on the SSH session
                            let channel_res = {
                                let s = sess.lock().unwrap();
                                s.channel_direct_tcpip(&r_host, remote_port, None)
                            };

                            let mut channel = match channel_res {
                                Ok(ch) => ch,
                                Err(_) => {
                                    // Try reconnecting session if stale
                                    if let Ok(new_sess) = Self::create_session(&cfg) {
                                        let mut s = sess.lock().unwrap();
                                        *s = new_sess;
                                        match s.channel_direct_tcpip(&r_host, remote_port, None) {
                                            Ok(ch) => ch,
                                            Err(_) => return,
                                        }
                                    } else {
                                        return;
                                    }
                                }
                            };

                            let mut local_clone = match local_stream.try_clone() {
                                Ok(c) => c,
                                Err(_) => return,
                            };
                            let mut channel_clone = match channel.stream(0) {
                                stream => stream,
                            };

                            // Bidirectional copy: local -> SSH channel
                            let t1 = thread::spawn(move || {
                                let _ = std::io::copy(&mut local_stream, &mut channel);
                                let _ = channel.send_eof();
                            });

                            // Bidirectional copy: SSH channel -> local
                            let _ = std::io::copy(&mut channel_clone, &mut local_clone);
                            let _ = t1.join();
                        });
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(50));
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        });

        Ok(SshTunnelHandle {
            local_port,
            shutdown_tx: Some(shutdown_tx),
        })
    }
}
