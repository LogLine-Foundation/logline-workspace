//! Path helpers for UBL layout and timestamp formatting.
use ubl_types::{AppId, NodeId, TenantId};
use std::path::{Path, PathBuf};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

/// Diretório diário para UBL (app/tenant/AAAA/MM/DD).
pub fn daily_dir(
    root: impl AsRef<Path>,
    app: &AppId,
    tenant: &TenantId,
    when: OffsetDateTime,
) -> PathBuf {
    let y = when.year();
    let m = when.month() as u8;
    let d = when.day();
    Path::new(root.as_ref())
        .join("ubl")
        .join(&app.0)
        .join(&tenant.0)
        .join(format!("{y:04}"))
        .join(format!("{m:02}"))
        .join(format!("{d:02}"))
}
/// Nome base do arquivo UBL (com part opcional).
#[must_use]
pub fn base_file_name(node: &NodeId, part: Option<u32>) -> String {
    part.map_or_else(
        || format!("ubl-{}.ndjson", node.0),
        |k| format!("ubl-{}-part{:.0}.ndjson", node.0, k),
    )
}
/// Timestamp RFC3339.
#[must_use]
///
/// # Panics
///
/// Panics se o formato RFC3339 não puder ser aplicado (improvável para entradas válidas).
pub fn ts_rfc3339(when: OffsetDateTime) -> String {
    when.format(&Rfc3339).unwrap()
}
