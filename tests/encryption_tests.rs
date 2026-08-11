use xs_rust_library::{
    cryptography::encryption::{Encryption, aes256_crypto::Aes256Crypto},
    encryption::aes256_crypto,
};

#[test]
fn aes_256_crypto() {
    let secret = [1_u8; 32];
    let mut crypto = Aes256Crypto::new(secret.into());

    let data = [0, 1, 2, 3];
    let encrypted = crypto.encrypt(&data).unwrap();
    assert_ne!(vec![0, 1, 2, 3], encrypted);
    let decrypted = crypto.decrypt(&encrypted).unwrap();
    assert_eq!(vec![0, 1, 2, 3], decrypted);
}

#[test]
fn encrypted_size() {
    let secret = [1_u8; 32];
    let mut crypto = Aes256Crypto::new(secret.into());

    let mut data = Vec::new();
    crypto.encrypt_in_place(&mut data).unwrap();
    assert_eq!(
        data.len(),
        aes256_crypto::NONCE_SIZE + aes256_crypto::TAG_SIZE
    );
}
