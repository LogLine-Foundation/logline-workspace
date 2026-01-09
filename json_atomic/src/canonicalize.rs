#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
use serde::Serialize;
use serde_json::{Number, Value};

use crate::errors::CanonicalError;

#[cfg(feature = "unicode")]
use unicode_normalization::UnicodeNormalization;

/// Canoniza qualquer `Serialize` em bytes determinísticos JSON-compatíveis.
///
/// Converte um valor serializável em bytes canônicos seguindo as regras do Paper II:
/// - **Objetos**: chaves ordenadas lexicograficamente
/// - **Strings**: normalização Unicode NFC (quando `feature = "unicode"`)
/// - **Números**: apenas inteiros em forma mínima (floats são rejeitados)
/// - **Literais**: `true`/`false`/`null` em minúsculas (padrão JSON)
/// - **Sem whitespace estrutural extra**
///
/// O resultado garante que valores semanticamente equivalentes produzem exatamente
/// os mesmos bytes, permitindo hashing determinístico e assinaturas verificáveis.
///
/// # Erros
///
/// - `CanonicalError::FloatNotAllowed` se o valor contiver números de ponto flutuante
/// - `CanonicalError::Serde` se a serialização falhar
///
/// # Exemplo
///
/// ```rust
/// use json_atomic::canonize;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Note {
///     title: String,
///     done: bool,
/// }
///
/// let note = Note {
///     title: "Hello".into(),
///     done: false,
/// };
///
/// let canonical = canonize(&note)?;
/// // Mesmo valor sempre produz os mesmos bytes
/// let canonical2 = canonize(&note)?;
/// assert_eq!(canonical, canonical2);
/// # Ok::<(), json_atomic::errors::CanonicalError>(())
/// ```
pub fn canonize<T: Serialize>(value: &T) -> Result<Vec<u8>, CanonicalError> {
    let v = serde_json::to_value(value).map_err(|e| CanonicalError::Serde(e.to_string()))?;
    let mut out = Vec::with_capacity(256);
    write_canonical(&v, &mut out)?;
    Ok(out)
}

fn write_canonical(v: &Value, out: &mut Vec<u8>) -> Result<(), CanonicalError> {
    match v {
        Value::Null => out.extend_from_slice(b"null"),
        Value::Bool(b) => out.extend_from_slice(if *b { b"true" } else { b"false" }),
        Value::Number(n) => write_number(n, out)?,
        Value::String(s) => write_string(s, out)?,
        Value::Array(arr) => {
            out.push(b'[');
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                write_canonical(item, out)?;
            }
            out.push(b']');
        }
        Value::Object(obj) => {
            out.push(b'{');
            let mut keys: Vec<&String> = obj.keys().collect();
            keys.sort_unstable();
            for (i, k) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                write_string(k, out)?;
                out.push(b':');
                write_canonical(&obj[*k], out)?;
            }
            out.push(b'}');
        }
    }
    Ok(())
}

fn write_number(n: &Number, out: &mut Vec<u8>) -> Result<(), CanonicalError> {
    if n.is_f64() {
        return Err(CanonicalError::FloatNotAllowed);
    }
    // Preferir u64, senão i64 (serde_json garante forma mínima no to_string())
    let s = if let Some(u) = n.as_u64() {
        u.to_string()
    } else if let Some(i) = n.as_i64() {
        i.to_string()
    } else {
        // números muito grandes como strings; ainda proibimos floats
        n.to_string()
    };
    out.extend_from_slice(s.as_bytes());
    Ok(())
}

fn write_string(s: &str, out: &mut Vec<u8>) -> Result<(), CanonicalError> {
    // NFC opcional (feature "unicode")
    #[cfg(feature = "unicode")]
    let normalized: String = s.nfc().collect();

    #[cfg(not(feature = "unicode"))]
    let normalized: String = s.to_string();

    // usa o escapamento JSON oficial do serde_json
    let escaped =
        serde_json::to_string(&normalized).map_err(|e| CanonicalError::Serde(e.to_string()))?;
    out.extend_from_slice(escaped.as_bytes());
    Ok(())
}
