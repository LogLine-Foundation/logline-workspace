//! DIM router that maps DIM codes to handlers and runs before/after middleware.
use crate::mw::Middleware;
use anyhow::Result;
use atomic_types::Dim;
use std::{collections::HashMap, sync::Arc};

/// Função de tratamento (DIM → bytes).
pub type HandlerFn = dyn Fn(&[u8]) -> Result<Vec<u8>> + Send + Sync + 'static;
/// Wrapper para mover closures p/ Arc.
pub struct FnHandler<F>(pub F);
impl<F> FnHandler<F>
where
    F: Fn(&[u8]) -> Result<Vec<u8>> + Send + Sync + 'static,
{
    /// Converte para `Arc<HandlerFn>`.
    pub fn into_arc(self) -> Arc<HandlerFn> {
        Arc::new(self.0)
    }
}

/// Router com middlewares before/after.
#[derive(Default)]
pub struct Router {
    pub(crate) map: HashMap<u16, Arc<HandlerFn>>,
    pub(crate) mw_before: Vec<Box<dyn Middleware>>,
    pub(crate) mw_after: Vec<Box<dyn Middleware>>,
}
impl Router {
    /// Registra handler para DIM.
    pub fn add<F>(&mut self, dim: Dim, h: FnHandler<F>)
    where
        F: Fn(&[u8]) -> Result<Vec<u8>> + Send + Sync + 'static,
    {
        self.map.insert(dim.0, h.into_arc());
    }
    /// Busca handler para DIM.
    #[must_use]
    pub fn get(&self, dim: Dim) -> Option<Arc<HandlerFn>> {
        self.map.get(&dim.0).cloned()
    }
    /// Adiciona middleware "before".
    pub fn use_before<M: Middleware + 'static>(&mut self, m: M) {
        self.mw_before.push(Box::new(m));
    }
    /// Adiciona middleware "after".
    pub fn use_after<M: Middleware + 'static>(&mut self, m: M) {
        self.mw_after.push(Box::new(m));
    }
}
