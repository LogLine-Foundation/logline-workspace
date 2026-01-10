#![forbid(unsafe_code)]
#![allow(clippy::multiple_crate_versions)]

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logline::cli::run("logline").await
}
