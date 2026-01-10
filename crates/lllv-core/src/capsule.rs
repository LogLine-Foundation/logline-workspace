//! Capsule struct plus signing/verification helpers for the LLLV binary format.
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

use crate::{
    errors::LllvError,
    header::{CapsuleFlags, CapsuleHeader},
    version::HEADER_LEN,
};
use blake3::hash;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};

#[derive(Clone, Debug)]
pub struct Capsule {
    pub header: CapsuleHeader,
    pub payload: Vec<u8>,
}

impl Capsule {
    /// Cria uma cápsula assinada.
    ///
    /// # Errors
    ///
    /// - `LllvError::TimestampOverflow` se o timestamp não couber em `u64`
    /// - `LllvError::MismatchedLengths` se o payload exceder `u32::MAX`
    /// - `LllvError::Crypto` se a assinatura falhar
    pub fn create(
        dim: u16,
        payload: &[u8],
        flags: CapsuleFlags,
        sk: &SigningKey,
    ) -> Result<Self, LllvError> {
        let cid = *hash(payload).as_bytes();
        #[cfg(feature = "std")]
        let ts_ms = u64::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| LllvError::TimestampOverflow)?
                .as_millis(),
        )
        .map_err(|_| LllvError::TimestampOverflow)?;
        #[cfg(not(feature = "std"))]
        let ts_ms = 0u64; // TODO: usar clock externo em no_std
        let len = u32::try_from(payload.len()).map_err(|_| LllvError::MismatchedLengths)?;
        let mut header = CapsuleHeader::empty(dim, flags, cid, len, ts_ms);

        // sign(header_without_sig || payload)
        let mut to_sign = Vec::with_capacity(HEADER_LEN - 64 + payload.len());
        to_sign.extend_from_slice(&header.to_bytes_wo_sig());
        to_sign.extend_from_slice(payload);

        let sig: Signature = sk.sign(&to_sign);
        header.sig.copy_from_slice(&sig.to_bytes());

        Ok(Self {
            header,
            payload: payload.to_vec(),
        })
    }

    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(HEADER_LEN + self.payload.len());
        v.extend_from_slice(&self.header.to_bytes());
        v.extend_from_slice(&self.payload);
        v
    }

    /// # Errors
    ///
    /// - `LllvError::InvalidHeaderLen` se o buffer for curto
    /// - `LllvError::MismatchedLengths` se o payload não corresponder ao header
    pub fn from_bytes(raw: &[u8]) -> Result<Self, LllvError> {
        let header = CapsuleHeader::from_bytes(raw)?;
        let payload = raw[HEADER_LEN..].to_vec();
        if payload.len() != header.len as usize {
            return Err(LllvError::MismatchedLengths);
        }
        Ok(Self { header, payload })
    }

    /// Verifica **integridade** (CID cobre payload). Não verifica autoria.
    /// # Errors
    ///
    /// - `LllvError::BadSignature` se o CID recalculado divergir
    pub fn verify_cid(&self) -> Result<(), LllvError> {
        let cid_now = *hash(&self.payload).as_bytes();
        if cid_now != self.header.cid {
            return Err(LllvError::BadSignature);
        }
        Ok(())
    }

    /// Verifica **integridade e autenticidade** com a chave pública.
    ///
    /// # Errors
    ///
    /// - Propaga erros de `verify_cid`
    /// - `LllvError::BadSignature` se a verificação Ed25519 falhar
    pub fn verify_with(&self, pk: &VerifyingKey) -> Result<(), LllvError> {
        self.verify_cid()?;
        let mut to_verify = Vec::with_capacity(HEADER_LEN - 64 + self.payload.len());
        to_verify.extend_from_slice(&self.header.to_bytes_wo_sig());
        to_verify.extend_from_slice(&self.payload);

        let sig = ed25519_dalek::Signature::from_bytes(&self.header.sig);
        pk.verify_strict(&to_verify, &sig)
            .map_err(|_| LllvError::BadSignature)
    }

    #[deprecated(
        since = "0.1.0",
        note = "use verify_with(&pk) para autenticidade ou verify_cid() para integridade"
    )]
    /// Verifica integridade somente (equivalente a `verify_cid`).
    ///
    /// # Errors
    ///
    /// - Propaga os mesmos erros de `verify_cid`
    pub fn verify(&self) -> Result<(), LllvError> {
        self.verify_cid()
    }
}
