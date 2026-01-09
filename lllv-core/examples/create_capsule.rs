//! Exemplo básico: criar e verificar uma Capsule LLLV
//!
//! Este exemplo demonstra:
//! - Criação de uma capsule com payload de vetor
//! - Verificação de integridade (CID)
//! - Verificação de autenticidade (assinatura)

use ed25519_dalek::{SigningKey, VerifyingKey};
use lllv_core::{Capsule, CapsuleFlags};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Gerar chave de assinatura
    let sk = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
    let pk: VerifyingKey = sk.verifying_key();

    // Criar payload de exemplo (vetor de 128 dimensões, f32)
    let dim: u16 = 128;
    let mut payload = Vec::with_capacity(dim as usize * 4);
    for i in 0..dim as usize {
        payload.extend_from_slice(&(i as f32).to_le_bytes());
    }

    // Criar capsule
    let capsule = Capsule::create(dim, &payload, CapsuleFlags::NONE, &sk)?;
    println!("✅ Capsule criada com sucesso!");
    println!("   Dimensão: {}", capsule.header.dim);
    println!("   Tamanho do payload: {} bytes", capsule.payload.len());

    // Verificar integridade (CID)
    capsule.verify_cid()?;
    println!("✅ Verificação de integridade (CID) passou!");

    // Verificar autenticidade (assinatura)
    capsule.verify_with(&pk)?;
    println!("✅ Verificação de autenticidade (assinatura) passou!");

    // Serializar para bytes
    let bytes = capsule.to_bytes();
    println!("✅ Capsule serializada: {} bytes", bytes.len());

    // Deserializar
    let capsule2 = Capsule::from_bytes(&bytes)?;
    println!("✅ Capsule deserializada com sucesso!");

    // Verificar novamente
    capsule2.verify_cid()?;
    capsule2.verify_with(&pk)?;
    println!("✅ Verificação após deserialização passou!");

    Ok(())
}
