/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (andypangaribuan@treasury.id)
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use std::fs;
use std::path::Path;

pub const ENCRYPTED_HEADER_PREFIX: &str = "# SQUIRREL:ENCRYPTED:v1:";

pub fn is_encrypted(content: &str) -> bool {
    content.lines().next().is_some_and(|line| line.trim().starts_with(ENCRYPTED_HEADER_PREFIX))
}

pub fn read_password(prompt_msg: &str) -> Result<String, String> {
    print!("{}", prompt_msg);
    use std::io::Write;
    let _ = std::io::stdout().flush();
    let pwd = rpassword::read_password().map_err(|e| format!("Failed to read password: {}", e))?;
    print!("\x1b[1A\x1b[2K\r");
    let _ = std::io::stdout().flush();
    Ok(pwd)
}

fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100_000, &mut key);
    key
}

pub fn encrypt_text(text: &str, password: &str) -> Result<String, String> {
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);

    let key = derive_key(password, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("Cipher init error: {}", e))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, text.as_bytes()).map_err(|_| "Encryption failed".to_string())?;

    let salt_b64 = BASE64.encode(salt);
    let nonce_b64 = BASE64.encode(nonce_bytes);
    let cipher_b64 = BASE64.encode(ciphertext);

    Ok(format!("{}{}:{}:{}\n", ENCRYPTED_HEADER_PREFIX, salt_b64, nonce_b64, cipher_b64))
}

pub fn decrypt_text(encrypted_content: &str, password: &str) -> Result<String, String> {
    let first_line = encrypted_content.lines().next().ok_or_else(|| "empty content".to_string())?.trim();
    if !first_line.starts_with(ENCRYPTED_HEADER_PREFIX) {
        return Err("content is not encrypted with SQUIRREL header".to_string());
    }

    let payload = &first_line[ENCRYPTED_HEADER_PREFIX.len()..];
    let parts: Vec<&str> = payload.split(':').collect();
    if parts.len() != 3 {
        return Err("invalid encrypted payload format".to_string());
    }

    let salt = BASE64.decode(parts[0]).map_err(|_| "invalid salt base64".to_string())?;
    let nonce_bytes = BASE64.decode(parts[1]).map_err(|_| "invalid nonce base64".to_string())?;
    let ciphertext = BASE64.decode(parts[2]).map_err(|_| "invalid ciphertext base64".to_string())?;

    if nonce_bytes.len() != 12 {
        return Err("invalid nonce length".to_string());
    }

    let key = derive_key(password, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("cipher init error: {}", e))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let decrypted_bytes =
        cipher.decrypt(nonce, ciphertext.as_ref()).map_err(|_| "decryption failed (incorrect password or corrupted data)".to_string())?;

    String::from_utf8(decrypted_bytes).map_err(|_| "decrypted data is not valid UTF-8".to_string())
}

pub fn prompt_and_decrypt(encrypted_content: &str, file_name: &str) -> String {
    let prompt_msg = format!("password to decrypt {}: ", file_name);
    loop {
        let password = match read_password(&prompt_msg) {
            Ok(pwd) => pwd,
            Err(e) => {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        };

        match decrypt_text(encrypted_content, &password) {
            Ok(decrypted) => return decrypted,
            Err(e) => {
                use std::io::Write;
                print!("error decrypting {}: {}", file_name, e);
                let _ = std::io::stdout().flush();
                std::thread::sleep(std::time::Duration::from_secs(3));
                print!("\r\x1b[2K");
                let _ = std::io::stdout().flush();
            }
        }
    }
}

pub fn toggle_encrypt_decrypt_file(file_path: &str) -> Result<String, String> {
    if !Path::new(file_path).exists() {
        return Err(format!("File '{}' not found", file_path));
    }

    let content = fs::read_to_string(file_path).map_err(|e| format!("failed to read file: {}", e))?;
    if is_encrypted(&content) {
        let file_name = Path::new(file_path).file_name().map_or(".secret.yml", |n| n.to_str().unwrap_or(".secret.yml"));
        let decrypted = prompt_and_decrypt(&content, file_name);
        fs::write(file_path, &decrypted).map_err(|e| format!("failed to write file: {}", e))?;
        Ok(format!("successfully decrypted: {}", file_path))
    } else {
        let password = read_password("password to encrypt: ")?;
        if password.is_empty() {
            return Err("password cannot be empty".to_string());
        }
        // let confirm = read_password("Confirm password: ")?;
        // if password != confirm {
        //     return Err("passwords do not match".to_string());
        // }
        let encrypted = encrypt_text(&content, &password)?;
        fs::write(file_path, &encrypted).map_err(|e| format!("failed to write file: {}", e))?;
        Ok(format!("successfully encrypted: {}", file_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_cycle() {
        let plain = "stringData:\n  APP_NAME: partner-api\n  DB_PASS: mysecretpassword\n";
        let password = "my_strong_password_123";

        let encrypted = encrypt_text(plain, password).expect("Encryption failed");
        assert!(is_encrypted(&encrypted));

        let decrypted = decrypt_text(&encrypted, password).expect("Decryption failed");
        assert_eq!(plain, decrypted);
    }

    #[test]
    fn test_wrong_password_fails() {
        let plain = "hello world";
        let encrypted = encrypt_text(plain, "correct_pass").unwrap();
        let result = decrypt_text(&encrypted, "wrong_pass");
        assert!(result.is_err());
    }
}
