use tauri::{State, Manager};
use crate::protocol;
use crate::app_state::DbState;
use serde_json::Value;
use pqcrypto_traits::kem::SecretKey;

#[tauri::command]
pub fn protocol_establish_session(state: State<'_, DbState>, remote_hash: String, bundle: Value) -> Result<(), String> {
    let lock = state.conn.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        protocol::establish_outbound_session(conn, &remote_hash, &bundle)
    } else {
        Err("Vault not initialized".to_string())
    }
}

#[tauri::command]
pub fn protocol_encrypt(state: State<'_, DbState>, remote_hash: String, plaintext: String) -> Result<Value, String> {
    let lock = state.conn.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        protocol::ratchet_encrypt(conn, &remote_hash, &plaintext)
    } else {
        Err("Vault not initialized".to_string())
    }
}

#[tauri::command]
pub fn protocol_decrypt(state: State<'_, DbState>, remote_hash: String, msg_obj: Value) -> Result<String, String> {
    let lock = state.conn.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        protocol::ratchet_decrypt(conn, &remote_hash, &msg_obj)
    } else {
        Err("Vault not initialized".to_string())
    }
}

#[tauri::command]
pub fn protocol_get_safety_number(me_ik: String, peer_ik: String) -> Result<String, String> {
    protocol::calculate_safety_number(&me_ik, &peer_ik)
}

#[tauri::command]
pub fn protocol_init(state: State<'_, DbState>) -> Result<Value, String> {
    let lock = state.conn.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        let identity = if let Some(identity) = protocol::ProtocolIdentity::load_from_db(conn)? {
            identity
        } else {
            let identity = protocol::generate_new_identity();
            identity.save_to_db(conn)?;
            identity
        };

        Ok(serde_json::json!({
            "registration_id": identity.registration_id,
            "identity_key": identity.identity_keys.public_key,
            "pq_identity_key": identity.identity_keys.pq_public_key,
            "signed_pre_key": {
                "key_id": identity.signed_pre_key.key_id,
                "public_key": identity.signed_pre_key.public_key,
                "signature": identity.signed_pre_key.signature,
                "pq_public_key": identity.signed_pre_key.pq_public_key
            },
            "pre_keys": identity.pre_keys.iter().map(|pk| serde_json::json!({
                "key_id": pk.key_id,
                "public_key": pk.public_key
            })).collect::<Vec<_>>()
        }))
    } else {
        Err("Vault not initialized".to_string())
    }
}

#[tauri::command]
pub fn protocol_sign(state: State<'_, DbState>, message: String) -> Result<String, String> {
    let lock = state.conn.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        protocol::sign_message(conn, message.as_bytes())
    } else {
        Err("Vault not initialized".to_string())
    }
}

#[tauri::command]
pub fn protocol_replenish_pre_keys(state: State<'_, DbState>, count: u32) -> Result<Value, String> {
    let lock = state.conn.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        let mut identity = protocol::ProtocolIdentity::load_from_db(conn)?.ok_or("No identity")?;
        identity.replenish_pre_keys(count);
        identity.save_to_db(conn)?;
        Ok(serde_json::json!({
            "pre_keys": identity.pre_keys.iter().rev().take(count as usize).map(|pk| serde_json::json!({
                "key_id": pk.key_id,
                "public_key": pk.public_key
            })).collect::<Vec<_>>()
        }))
    } else {
        Err("Vault not initialized".to_string())
    }
}

#[tauri::command]
pub fn protocol_verify_session(state: State<'_, DbState>, remote_hash: String, verified: bool) -> Result<(), String> {
    let lock = state.conn.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        protocol::verify_session(conn, &remote_hash, verified)
    } else {
        Err("Vault not initialized".to_string())
    }
}

#[tauri::command]
pub fn protocol_secure_vacuum(state: State<'_, DbState>) -> Result<(), String> {
    let lock = state.conn.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        conn.execute("VACUUM;", []).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Vault not initialized".to_string())
    }
}

#[tauri::command]
pub fn protocol_encrypt_sealed(
    state: State<'_, DbState>,
    remote_public_identity_key: String,
    remote_pq_public_identity_key: String,
    message_body: Value
) -> Result<Value, String> {
    let lock = state.conn.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        let identity = protocol::ProtocolIdentity::load_from_db(conn)?.ok_or("No identity")?;
        
        let mut pk_bytes = [0u8; 32];
        pk_bytes.copy_from_slice(&protocol::decode_b64(&remote_public_identity_key)?);
        let recipient_pk = protocol::X25519PublicKey::from(pk_bytes);

        protocol::seal_sender(message_body, &identity.identity_keys.public_key, &recipient_pk, &remote_pq_public_identity_key)
    } else {
        Err("Vault not initialized".to_string())
    }
}

#[tauri::command]
pub fn protocol_decrypt_sealed(
    state: State<'_, DbState>,
    sealed_obj: Value
) -> Result<Value, String> {
    let lock = state.conn.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        let identity = protocol::ProtocolIdentity::load_from_db(conn)?.ok_or("No identity")?;
        
        let mut sk_bytes = [0u8; 32];
        sk_bytes.copy_from_slice(&protocol::decode_b64(&identity.identity_keys.private_key)?);
        let my_sk = protocol::StaticSecret::from(sk_bytes);

        let my_pq_sk = pqcrypto_kyber::kyber1024::SecretKey::from_bytes(&protocol::decode_b64(&identity.identity_keys.pq_private_key)?).map_err(|_| "Invalid PQ SK")?;

        let (sender, message) = protocol::unseal_sender(&sealed_obj, &my_sk, &my_pq_sk)?;
        Ok(serde_json::json!({
            "sender": sender,
            "message": message
        }))
    } else {
        Err("Vault not initialized".to_string())
    }
}

#[tauri::command]
pub fn protocol_encrypt_media(state: State<'_, DbState>, data: Vec<u8>, file_name: String, file_type: String) -> Result<Value, String> {
    let lock = state.conn.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        let (ct, bundle) = protocol::encrypt_media(conn, &data, &file_name, &file_type)?;
        Ok(serde_json::json!({
            "ciphertext": hex::encode(ct),
            "bundle": bundle
        }))
    } else {
        Err("Vault not initialized".to_string())
    }
}

#[tauri::command]
pub fn protocol_decrypt_media(state: State<'_, DbState>, hex_data: String, bundle: Value) -> Result<Vec<u8>, String> {
    let lock = state.conn.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        let ct = hex::decode(hex_data).map_err(|e| e.to_string())?;
        let b: protocol::MediaKeyBundle = serde_json::from_value(bundle).map_err(|e| e.to_string())?;
        protocol::decrypt_media(conn, &ct, &b)
    } else {
        Err("Vault not initialized".to_string())
    }
}

#[tauri::command]
pub fn protocol_encrypt_media_chunk(key_b64: String, nonce_b64: String, chunk_index: u32, data: Vec<u8>) -> Result<Vec<u8>, String> {
    let key = protocol::decode_b64(&key_b64)?;
    let nonce = protocol::decode_b64(&nonce_b64)?;
    protocol::encrypt_media_chunk(&key, &nonce, chunk_index, &data)
}

#[tauri::command]
pub fn protocol_decrypt_media_chunk(key_b64: String, nonce_b64: String, chunk_index: u32, ciphertext: Vec<u8>) -> Result<Vec<u8>, String> {
    let key = protocol::decode_b64(&key_b64)?;
    let nonce = protocol::decode_b64(&nonce_b64)?;
    protocol::decrypt_media_chunk(&key, &nonce, chunk_index, &ciphertext)
}

#[tauri::command]
pub fn protocol_create_group_distribution(state: State<'_, DbState>, group_id: String) -> Result<Value, String> {
    let lock = state.conn.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        let mut stmt = conn.prepare("SELECT state FROM groups WHERE group_id = ?1;").map_err(|e| e.to_string())?;
        let row: String = stmt.query_row([&group_id], |r| r.get(0)).map_err(|e| e.to_string())?;
        let gs: protocol::GroupState = serde_json::from_str(&row).map_err(|e| e.to_string())?;
        protocol::create_group_distribution_message(&gs)
    } else {
        Err("Vault not initialized".to_string())
    }
}

#[tauri::command]
pub fn protocol_group_init(state: State<'_, DbState>, group_id: String) -> Result<Value, String> {
    let lock = state.conn.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        let gs = protocol::GroupState {
            group_id: group_id.clone(),
            my_sender_key: Some(protocol::create_group_sender_key()),
            member_sender_keys: std::collections::HashMap::new(),
            members: vec![]
        };
        gs.save_to_db(conn)?;
        let dist = protocol::create_group_distribution_message(&gs)?;
        Ok(dist)
    } else {
        Err("Vault not initialized".to_string())
    }
}

#[tauri::command]
pub fn protocol_group_encrypt(state: State<'_, DbState>, group_id: String, plaintext: String) -> Result<Value, String> {
    let lock = state.conn.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        let mut gs = protocol::GroupState::load_from_db(conn, &group_id)?.ok_or("Group not found")?;
        let res = protocol::group_encrypt(conn, &mut gs, &plaintext)?;
        gs.save_to_db(conn)?;
        Ok(res)
    } else {
        Err("Vault not initialized".to_string())
    }
}

#[tauri::command]
pub fn protocol_group_decrypt(state: State<'_, DbState>, group_id: String, sender_hash: String, msg_obj: Value) -> Result<String, String> {
    let lock = state.conn.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        let mut gs = protocol::GroupState::load_from_db(conn, &group_id)?.ok_or("Group not found")?;
        let res = protocol::group_decrypt(&mut gs, &sender_hash, &msg_obj)?;
        gs.save_to_db(conn)?;
        Ok(res)
    } else {
        Err("Vault not initialized".to_string())
    }
}

#[tauri::command]
pub fn protocol_process_group_distribution(state: State<'_, DbState>, sender_hash: String, dist_obj: Value) -> Result<(), String> {
    let lock = state.conn.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        let group_id = dist_obj["group_id"].as_str().ok_or("Missing group_id")?;
        let mut gs = protocol::GroupState::load_from_db(conn, group_id)?.unwrap_or_else(|| protocol::GroupState {
            group_id: group_id.to_string(),
            my_sender_key: None,
            member_sender_keys: std::collections::HashMap::new(),
            members: vec![]
        });
        
        let sk = protocol::SenderKey {
            key_id: dist_obj["key_id"].as_u64().ok_or("Missing key_id")? as u32,
            chain_key: dist_obj["chain_key"].as_str().ok_or("Missing chain_key")?.to_string(),
            signature_key_private: "".to_string(), 
            signature_key_public: dist_obj["signature_key_public"].as_str().ok_or("Missing signature_key_public")?.to_string(),
        };
        
        gs.member_sender_keys.insert(sender_hash, sk);
        gs.save_to_db(conn)?;
        Ok(())
    } else {
        Err("Vault not initialized".to_string())
    }
}

#[tauri::command]
pub fn protocol_get_pending(state: State<'_, DbState>) -> Result<Vec<protocol::PendingMessage>, String> {
    let lock = state.conn.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        protocol::get_pending_messages(conn)
    } else {
        Err("Vault not initialized".to_string())
    }
}

#[tauri::command]
pub fn protocol_remove_pending(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let lock = state.conn.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        protocol::remove_pending_message(conn, &id)
    } else {
        Err("Vault not initialized".to_string())
    }
}

#[tauri::command]
pub fn protocol_save_pending(state: State<'_, DbState>, msg: protocol::PendingMessage) -> Result<(), String> {
    let lock = state.conn.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        protocol::save_pending_message(conn, &msg)
    } else {
        Err("Vault not initialized".to_string())
    }
}

#[tauri::command]
pub fn protocol_export_vault(app: tauri::AppHandle) -> Result<Vec<u8>, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let db_path = app_data_dir.join("vault.db");
    if !db_path.exists() { return Err("Vault does not exist".to_string()); }
    std::fs::read(db_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn protocol_import_vault(app: tauri::AppHandle, state: State<'_, DbState>, bytes: Vec<u8>) -> Result<(), String> {
    {
        let mut lock = state.conn.lock().unwrap();
        *lock = None;
    }

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    if !app_data_dir.exists() {
        std::fs::create_dir_all(&app_data_dir).map_err(|e| e.to_string())?;
    }
    let db_path = app_data_dir.join("vault.db");
    std::fs::write(db_path, bytes).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub fn protocol_save_vault_to_path(path: String, bytes: Vec<u8>) -> Result<(), String> {
    std::fs::write(path, bytes).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn protocol_read_vault_from_path(path: String) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn protocol_save_message(state: State<'_, DbState>, peer_hash: String, msg: Value) -> Result<(), String> {
    let lock = state.conn.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        protocol::save_decrypted_message(conn, &peer_hash, &msg)
    } else {
        Err("Vault not initialized".to_string())
    }
}

#[tauri::command]
pub fn protocol_search_messages(state: State<'_, DbState>, query: String) -> Result<Vec<Value>, String> {
    let lock = state.conn.lock().unwrap();
    if let Some(conn) = lock.as_ref() {
        protocol::search_messages(conn, &query)
    } else {
        Err("Vault not initialized".to_string())
    }
}
