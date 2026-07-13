/// Transport protocol selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TransportKind {
    /// HTTP/REST.
    Http,
    /// gRPC.
    Grpc,
}

/// Transport configuration.
#[derive(Clone, Debug)]
pub struct TransportConfig {
    /// Protocol kind.
    pub kind: TransportKind,
    /// Bind address (e.g. `"127.0.0.1:8080"`).
    pub bind_addr: String,
}

/// Pluggable API transport.
pub trait Transport: Send + Sync {
    /// Transport kind.
    fn kind(&self) -> TransportKind;

    /// Bind address.
    fn bind_addr(&self) -> &str;
}
