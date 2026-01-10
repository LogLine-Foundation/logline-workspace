//! NDJSON reader and tail helpers for UBL event streams.
use crate::event::UblEvent;
use anyhow::Result;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    thread,
    time::Duration,
};

/// Leitor simplificado de NDJSON.
pub struct UblReader;
impl UblReader {
    /// Cria iterador para um arquivo NDJSON UBL.
    ///
    /// # Errors
    ///
    /// - Retorna erros de I/O ao abrir o arquivo
    pub fn iter_file<P: AsRef<Path>>(path: P) -> Result<UblIter> {
        let f = File::open(path)?;
        Ok(UblIter {
            reader: BufReader::new(f),
        })
    }
}
/// Iterador linha-a-linha.
pub struct UblIter {
    reader: BufReader<File>,
}
impl Iterator for UblIter {
    type Item = Result<UblEvent, anyhow::Error>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => None,
            Ok(_) => {
                if line.trim().is_empty() {
                    return self.next();
                }
                Some(serde_json::from_str::<UblEvent>(&line).map_err(Into::into))
            }
            Err(e) => Some(Err(e.into())),
        }
    }
}
/// "Tail -f" simplificado (bloqueante).
///
/// # Errors
///
/// - Erros de I/O ao ler/seekar o arquivo
pub fn tail_file<P: AsRef<Path>, F: Fn(UblEvent)>(path: P, on_event: F) -> Result<()> {
    use std::io::{Seek, SeekFrom};
    let mut f = File::open(path)?;
    let mut pos = f.seek(SeekFrom::End(0))?;
    loop {
        let mut r = BufReader::new(&mut f);
        r.seek(SeekFrom::Start(pos))?;
        let mut line = String::new();
        while r.read_line(&mut line)? > 0 {
            if !line.trim().is_empty() {
                if let Ok(ev) = serde_json::from_str::<UblEvent>(&line) {
                    on_event(ev);
                }
            }
            line.clear();
            pos = r.stream_position()?;
        }
        thread::sleep(Duration::from_millis(300));
    }
}
