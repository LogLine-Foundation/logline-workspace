//! Stdio transport for MCP (line-delimited JSON).

use crate::{JsonRpcRequest, JsonRpcResponse};
use bytes::BytesMut;
use std::io;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio_util::codec::{Decoder, Encoder};

/// Line-delimited JSON codec.
pub struct LineCodec;

impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(pos) = src.iter().position(|b| *b == b'\n') {
            let line = src.split_to(pos + 1);
            let s = String::from_utf8_lossy(&line[..line.len() - 1]).to_string();
            if s.is_empty() {
                Ok(None)
            } else {
                Ok(Some(s))
            }
        } else {
            Ok(None)
        }
    }
}

impl Encoder<String> for LineCodec {
    type Error = io::Error;

    fn encode(&mut self, item: String, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.extend_from_slice(item.as_bytes());
        dst.extend_from_slice(b"\n");
        Ok(())
    }
}

/// Stdio transport using newline-delimited JSON.
pub struct StdioTransport<R, W> {
    reader: BufReader<R>,
    writer: W,
}

impl<R, W> StdioTransport<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    /// Create a new stdio transport.
    pub fn new(reader: R, writer: W) -> Self {
        Self {
            reader: BufReader::new(reader),
            writer,
        }
    }

    /// Send a JSON-RPC message.
    pub async fn send(&mut self, msg: &JsonRpcResponse) -> io::Result<()> {
        let json = serde_json::to_string(msg).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        self.writer.write_all(json.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.writer.flush().await?;
        Ok(())
    }

    /// Receive a JSON-RPC request.
    pub async fn recv(&mut self) -> io::Result<Option<JsonRpcRequest>> {
        let mut line = String::new();
        let n = self.reader.read_line(&mut line).await?;
        if n == 0 {
            return Ok(None);
        }

        let line = line.trim();
        if line.is_empty() {
            return Ok(None);
        }

        let request: JsonRpcRequest = serde_json::from_str(line)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(Some(request))
    }
}

/// Standard stdio transport (stdin + stdout).
pub type StandardStdioTransport = StdioTransport<tokio::io::Stdin, tokio::io::Stdout>;

impl StandardStdioTransport {
    /// Create a transport using stdin and stdout.
    pub fn stdio() -> Self {
        Self::new(tokio::io::stdin(), tokio::io::stdout())
    }
}

// Note: StdioTransport requires mutable access and cannot implement
// McpTransport directly. Use it with McpServer for request handling,
// or wrap in a mutex for client use.

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn stdio_codec_roundtrip() {
        // Test the codec directly
        let mut codec = LineCodec;
        let mut buf = BytesMut::new();

        // Encode a line
        codec.encode("hello".to_string(), &mut buf).unwrap();
        assert_eq!(&buf[..], b"hello\n");

        // Decode should return the line (newline is present)
        let decoded = codec.decode(&mut buf).unwrap();
        assert_eq!(decoded, Some("hello".to_string()));

        // Buffer should be empty now
        assert!(buf.is_empty());

        // Encode another and decode
        buf.extend_from_slice(b"world\n");
        let decoded = codec.decode(&mut buf).unwrap();
        assert_eq!(decoded, Some("world".to_string()));
    }
}
