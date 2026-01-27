//! Basic TCP HTTP Server
//!
//! Low-level server without routing — gives full control to frameworks.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

use super::parser::{HttpParser, ParseError};
use super::request::Request;
use super::response::Response;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Read buffer size (default: 8KB)
    pub read_buffer_size: usize,
    /// Write buffer size (default: 8KB)
    pub write_buffer_size: usize,
    /// Read timeout in milliseconds (0 = no timeout)
    pub read_timeout_ms: u64,
    /// Write timeout in milliseconds (0 = no timeout)
    pub write_timeout_ms: u64,
    /// Maximum request size (default: 1MB)
    pub max_request_size: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            read_buffer_size: 8 * 1024,
            write_buffer_size: 8 * 1024,
            read_timeout_ms: 30_000,
            write_timeout_ms: 30_000,
            max_request_size: 1024 * 1024,
        }
    }
}

/// Connection state for handling keep-alive
pub struct Connection {
    stream: TcpStream,
    read_buf: Vec<u8>,
    write_buf: Vec<u8>,
    read_pos: usize,
    config: ServerConfig,
}

impl Connection {
    /// Create new connection
    pub fn new(stream: TcpStream, config: ServerConfig) -> std::io::Result<Self> {
        // Set timeouts if configured
        if config.read_timeout_ms > 0 {
            stream.set_read_timeout(Some(std::time::Duration::from_millis(
                config.read_timeout_ms,
            )))?;
        }
        if config.write_timeout_ms > 0 {
            stream.set_write_timeout(Some(std::time::Duration::from_millis(
                config.write_timeout_ms,
            )))?;
        }

        Ok(Self {
            stream,
            read_buf: vec![0u8; config.read_buffer_size],
            write_buf: Vec::with_capacity(config.write_buffer_size),
            read_pos: 0,
            config,
        })
    }

    /// Handle request with a callback (zero-copy friendly)
    /// Returns Ok(true) if connection should be kept alive
    /// Returns Ok(false) if connection should close
    /// Returns Err on error
    pub fn handle_request<F>(&mut self, handler: F) -> Result<bool, ConnectionError>
    where
        F: FnOnce(&Request) -> Response,
    {
        let parser = HttpParser::new();

        // Read until we have a complete request
        loop {
            // Try to parse what we have
            match parser.parse(&self.read_buf[..self.read_pos]) {
                Ok((req, consumed)) => {
                    let keep_alive = req.is_keep_alive();

                    // Call handler
                    let response = handler(&req);

                    // Write response
                    self.write_response(&response)?;

                    // Shift remaining data to front
                    if consumed < self.read_pos {
                        self.read_buf.copy_within(consumed..self.read_pos, 0);
                        self.read_pos -= consumed;
                    } else {
                        self.read_pos = 0;
                    }

                    return Ok(keep_alive);
                }
                Err(ParseError::Incomplete) => {
                    // Need more data
                    if self.read_pos >= self.config.max_request_size {
                        return Err(ConnectionError::RequestTooLarge);
                    }

                    // Grow buffer if needed
                    if self.read_pos >= self.read_buf.len() {
                        let new_size = (self.read_buf.len() * 2).min(self.config.max_request_size);
                        self.read_buf.resize(new_size, 0);
                    }

                    // Read more data
                    match self.stream.read(&mut self.read_buf[self.read_pos..]) {
                        Ok(0) => {
                            // Connection closed
                            if self.read_pos == 0 {
                                return Err(ConnectionError::Closed);
                            } else {
                                return Err(ConnectionError::UnexpectedEof);
                            }
                        }
                        Ok(n) => {
                            self.read_pos += n;
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            continue;
                        }
                        Err(e) => return Err(ConnectionError::Io(e)),
                    }
                }
                Err(e) => return Err(ConnectionError::Parse(e)),
            }
        }
    }

    /// Write response
    pub fn write_response(&mut self, response: &Response) -> Result<(), ConnectionError> {
        response.write_to(&mut self.write_buf);
        self.stream
            .write_all(&self.write_buf)
            .map_err(ConnectionError::Io)?;
        self.stream.flush().map_err(ConnectionError::Io)?;
        Ok(())
    }

    /// Get peer address
    pub fn peer_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.stream.peer_addr()
    }
}

/// Connection error
#[derive(Debug)]
pub enum ConnectionError {
    Io(std::io::Error),
    Parse(ParseError),
    RequestTooLarge,
    UnexpectedEof,
    Closed,
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionError::Io(e) => write!(f, "I/O error: {}", e),
            ConnectionError::Parse(e) => write!(f, "Parse error: {}", e),
            ConnectionError::RequestTooLarge => write!(f, "Request too large"),
            ConnectionError::UnexpectedEof => write!(f, "Unexpected end of connection"),
            ConnectionError::Closed => write!(f, "Connection closed"),
        }
    }
}

impl std::error::Error for ConnectionError {}

/// Basic HTTP Server
///
/// Provides simple accept loop - no routing, no middleware.
/// Frameworks build on top of this.
pub struct HttpServer {
    listener: TcpListener,
    config: ServerConfig,
}

impl HttpServer {
    /// Bind server to address
    pub fn bind<A: ToSocketAddrs>(addr: A) -> std::io::Result<Self> {
        Self::bind_with_config(addr, ServerConfig::default())
    }

    /// Bind server with custom config
    pub fn bind_with_config<A: ToSocketAddrs>(
        addr: A,
        config: ServerConfig,
    ) -> std::io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        Ok(Self { listener, config })
    }

    /// Get local address
    pub fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.listener.local_addr()
    }

    /// Accept next connection
    pub fn accept(&self) -> std::io::Result<Connection> {
        let (stream, _addr) = self.listener.accept()?;
        Connection::new(stream, self.config.clone())
    }

    /// Run server with handler function
    ///
    /// This is the simplest way to run a server.
    /// For more control, use accept() directly.
    pub fn run<F>(&self, handler: F) -> std::io::Result<()>
    where
        F: Fn(&Request) -> Response,
    {
        loop {
            let mut conn = self.accept()?;

            // Handle connection (supports keep-alive)
            loop {
                match conn.handle_request(&handler) {
                    Ok(true) => continue, // keep-alive
                    Ok(false) => break,   // close
                    Err(ConnectionError::Closed) => break,
                    Err(e) => {
                        eprintln!("Connection error: {}", e);
                        break;
                    }
                }
            }
        }
    }

    /// Run server with handler, processing connections in separate threads
    pub fn run_threaded<F>(&self, handler: F) -> std::io::Result<()>
    where
        F: Fn(&Request) -> Response + Send + Sync + 'static,
    {
        let handler = std::sync::Arc::new(handler);

        loop {
            let mut conn = self.accept()?;
            let handler = handler.clone();

            std::thread::spawn(move || loop {
                match conn.handle_request(|req| handler(req)) {
                    Ok(true) => continue,
                    Ok(false) => break,
                    Err(ConnectionError::Closed) => break,
                    Err(e) => {
                        eprintln!("Connection error: {}", e);
                        break;
                    }
                }
            });
        }
    }
}

/// Quick helper to start a server
///
/// ```ignore
/// use phprs_runtime::http::{serve, Request, Response};
///
/// serve("127.0.0.1:8080", |req| {
///     Response::ok().text("Hello, World!")
/// }).unwrap();
/// ```
pub fn serve<A, F>(addr: A, handler: F) -> std::io::Result<()>
where
    A: ToSocketAddrs,
    F: Fn(&Request) -> Response,
{
    let server = HttpServer::bind(addr)?;
    println!("Listening on {}", server.local_addr()?);
    server.run(handler)
}

/// Quick helper to start a threaded server
pub fn serve_threaded<A, F>(addr: A, handler: F) -> std::io::Result<()>
where
    A: ToSocketAddrs,
    F: Fn(&Request) -> Response + Send + Sync + 'static,
{
    let server = HttpServer::bind(addr)?;
    println!("Listening on {}", server.local_addr()?);
    server.run_threaded(handler)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::thread;

    #[test]
    fn test_server_basic() {
        // Start server in background
        let server = HttpServer::bind("127.0.0.1:0").unwrap();
        let addr = server.local_addr().unwrap();

        thread::spawn(move || {
            // Handle just one connection for test
            let mut conn = server.accept().unwrap();
            let _ = conn.handle_request(|_req| Response::ok().text("Hello"));
        });

        // Give server time to start
        thread::sleep(std::time::Duration::from_millis(10));

        // Connect and send request
        let mut client = TcpStream::connect(addr).unwrap();
        client
            .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .unwrap();

        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();

        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("Hello"));
    }
}
