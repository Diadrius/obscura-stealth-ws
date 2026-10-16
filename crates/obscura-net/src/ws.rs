//! Message types shared between the stealth WebSocket pump and the JS ops.

/// Page → socket.
pub enum WsOut {
    Text(String),
    Binary(Vec<u8>),
    Close(u16, String),
}

/// Socket → page.
pub enum WsIn {
    Text(String),
    Binary(Vec<u8>),
    Close(u16, String),
    Error(String),
}
