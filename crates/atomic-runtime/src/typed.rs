//! Typed handler adapter: canonical decode → handler → canonical encode.
use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
/// Handler que (de)serializa em forma canônica.
///
/// # Errors
///
/// - Propaga erros de (de)serialização ou do handler interno
pub fn handle_typed<TReq, TRes, F>(bytes: &[u8], f: F) -> Result<Vec<u8>>
where
    TReq: DeserializeOwned,
    TRes: Serialize,
    F: Fn(TReq) -> Result<TRes>,
{
    let req: TReq = ubl_codec::from_canon_slice(bytes)?;
    let res = f(req)?;
    Ok(ubl_codec::to_canon_vec(&res)?)
}
