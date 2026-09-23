use crate::model::protocol::{
    MAX_FRAME_SIZE_BYTES, ProtocolEnvelope, decode_frame_length, encode_frame,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::io::{Error, ErrorKind};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub async fn send_envelope<W, T>(
    writer: &mut W,
    envelope: &ProtocolEnvelope<T>,
) -> Result<(), Error>
where
    W: AsyncWrite + Unpin,
    T: Serialize,
{
    let json_bytes = serde_json::to_vec(envelope).map_err(|e| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Error al serializar mensaje de protocolo: {}", e),
        )
    })?;

    let frame = encode_frame(&json_bytes).map_err(|e| {
        Error::new(
            ErrorKind::InvalidInput,
            format!("Error al codificar trama: {}", e),
        )
    })?;

    writer.write_all(&frame).await?;
    writer.flush().await?;
    Ok(())
}

pub async fn recv_envelope<R, T>(reader: &mut R) -> Result<ProtocolEnvelope<T>, Error>
where
    R: AsyncRead + Unpin,
    T: DeserializeOwned,
{
    // 1. Leer encabezado de 4 bytes con la longitud
    let mut header = [0u8; 4];
    reader.read_exact(&mut header).await?;
    let body_len = decode_frame_length(header);

    if body_len > MAX_FRAME_SIZE_BYTES {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "Trama entrante excede el límite de 1 MiB (longitud: {} bytes)",
                body_len
            ),
        ));
    }

    // 2. Leer exactamente el cuerpo
    let mut body = vec![0u8; body_len];
    reader.read_exact(&mut body).await?;

    // 3. Deserializar envelope JSON
    let envelope: ProtocolEnvelope<T> = serde_json::from_slice(&body).map_err(|e| {
        Error::new(
            ErrorKind::InvalidData,
            format!("JSON de protocolo no válido: {}", e),
        )
    })?;

    Ok(envelope)
}
