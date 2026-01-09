use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryRequest {
    pub vec: Vec<f32>,
}

impl QueryRequest {
    pub fn from_vec(v: &[f32]) -> Self {
        Self { vec: v.to_vec() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofStepSerde {
    pub sibling_hex: String, // hex(32B)
    pub sibling_is_right: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryResult {
    pub id: String,
    pub score: f32,       // cosine
    pub leaf_hex: String, // hash da folha (doc)
    pub path: Vec<ProofStepSerde>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryEvidence {
    pub index_pack_cid: String, // root do merkle da doc_table
    pub results: Vec<QueryResult>,
    pub dim: u16,
    // se feature "manifest", opcionalmente poderíamos anexar um SignedFact aqui
}
