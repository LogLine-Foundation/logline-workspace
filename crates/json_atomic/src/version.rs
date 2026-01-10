/// Versão do formato canônico JSON✯Atomic (Paper II).
///
/// Atualmente "1". Incrementa quando houver mudanças incompatíveis
/// nas regras de canonicalização.
pub const CANON_VERSION: &str = "1";

/// Identificador curto do formato JSON✯Atomic.
///
/// Formato: `"json-atomic/{version}"`. Atualmente `"json-atomic/1"`.
/// Usado em `SignedFact` para identificar o formato do fato assinado.
pub const FORMAT_ID: &str = "json-atomic/1";
