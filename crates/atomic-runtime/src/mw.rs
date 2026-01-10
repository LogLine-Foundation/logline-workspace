//! Middleware traits and budget tracking for DIM processing.
use anyhow::Result;
use ubl_types::{ActorId, Dim};
use std::collections::HashMap;

/// Middleware simples (before/after).
pub trait Middleware: Send + Sync {
    /// Executa antes do handler.
    ///
    /// # Errors
    ///
    /// - Implementações podem sinalizar erros de autorização/validação
    fn before(&self, _dim: Dim, _actor: &ActorId, _in: &[u8], _ctx: &crate::AppCtx) -> Result<()> {
        Ok(())
    }
    /// Executa depois do handler.
    ///
    /// # Errors
    ///
    /// - Implementações podem sinalizar erros de auditoria/telemetria
    fn after(
        &self,
        _dim: Dim,
        _actor: &ActorId,
        _in: &[u8],
        _out: &[u8],
        _ctx: &crate::AppCtx,
    ) -> Result<()> {
        Ok(())
    }
}

/// Controle de budgets (cotas) por ator.
#[derive(Default)]
pub struct Budgets {
    map: HashMap<String, i64>,
}
impl Budgets {
    /// Configura quota para um ator.
    pub fn set(&mut self, actor: &ActorId, quota: i64) {
        self.map.insert(actor.0.clone(), quota);
    }
    /// Consome 1 e retorna o restante (se houver registro). None => sem budget.
    pub fn consume(&mut self, actor: &ActorId, amount: i64) -> Option<i64> {
        self.map.get_mut(&actor.0).map(|v| {
            *v = (*v - amount).max(0);
            *v
        })
    }
}
