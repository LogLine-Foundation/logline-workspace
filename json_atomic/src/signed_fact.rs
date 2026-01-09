#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use ed25519_dalek::{Signature, VerifyingKey};
use hex::ToHex;

/// Signed Fact — fato assinado imutável (Paper II).
///
/// Representa um valor canônico com CID (BLAKE3) e assinatura Ed25519.
/// Um `SignedFact` é imutável e verificável por qualquer pessoa que tenha acesso
/// ao fato, sem necessidade de confiar em terceiros.
///
/// # Estrutura
///
/// - `canonical`: bytes canônicos JSON✯Atomic do valor original
/// - `cid`: BLAKE3 hash dos bytes canônicos (32 bytes)
/// - `signature`: assinatura Ed25519 sobre o CID (64 bytes)
/// - `public_key`: chave pública Ed25519 (32 bytes)
/// - `hash_alg`: algoritmo de hash ("blake3")
/// - `sig_alg`: algoritmo de assinatura ("ed25519")
/// - `canon_ver`: versão do formato canônico ("1")
/// - `format_id`: identificador do formato ("json-atomic/1")
///
/// # Exemplo
///
/// ```rust
/// use ed25519_dalek::SigningKey;
/// use json_atomic::{seal_value, verify_seal, errors::SealError};
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Data { value: u64 }
///
/// // Chave de exemplo (em produção, derive de seed/keystore)
/// let sk = SigningKey::from_bytes(&[0u8; 32]);
/// let signed = seal_value(&Data { value: 42 }, &sk)?;
///
/// // Verifica integridade e autenticidade
/// verify_seal(&signed).unwrap();
///
/// // Acessa o CID em hexadecimal
/// println!("CID: {}", signed.cid_hex());
/// # Ok::<(), SealError>(())
/// ```
#[derive(Clone, Debug)]
pub struct SignedFact {
    /// Bytes canônicos JSON✯Atomic do valor original.
    pub canonical: Vec<u8>,
    /// BLAKE3 hash dos bytes canônicos (32 bytes).
    pub cid: [u8; 32],
    /// Assinatura Ed25519 sobre o CID (64 bytes).
    pub signature: [u8; 64],
    /// Chave pública Ed25519 (32 bytes).
    pub public_key: [u8; 32],
    /// Algoritmo de hash (sempre "blake3").
    pub hash_alg: &'static str,
    /// Algoritmo de assinatura (sempre "ed25519").
    pub sig_alg: &'static str,
    /// Versão do formato canônico (sempre "1").
    pub canon_ver: &'static str,
    /// Identificador do formato (sempre "json-atomic/1").
    pub format_id: &'static str,
}

impl SignedFact {
    /// Retorna a chave pública Ed25519 para verificação.
    ///
    /// # Panics
    ///
    /// Panics se a chave pública não for válida (não deve acontecer em SignedFacts válidos).
    pub fn verifying_key(&self) -> VerifyingKey {
        VerifyingKey::from_bytes(&self.public_key).expect("valid pk")
    }
    /// Retorna a assinatura como objeto `ed25519_dalek::Signature`.
    pub fn signature_obj(&self) -> Signature {
        Signature::from_bytes(&self.signature)
    }
    /// Retorna o CID em formato hexadecimal (64 caracteres).
    ///
    /// # Exemplo
    ///
    /// ```rust
    /// use ed25519_dalek::SigningKey;
    /// use json_atomic::{seal_value, verify_seal};
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Data { x: u32 }
    ///
    /// // Chave de exemplo (em produção, derive de seed/keystore)
    /// let sk = SigningKey::from_bytes(&[0u8; 32]);
    /// let signed = seal_value(&Data { x: 1 }, &sk).unwrap();
    /// verify_seal(&signed).unwrap();
    /// let hex = signed.cid_hex();
    /// assert_eq!(hex.len(), 64); // 32 bytes = 64 hex chars
    /// ```
    pub fn cid_hex(&self) -> String {
        self.cid.encode_hex::<String>()
    }
}
