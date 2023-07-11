use xs_rust_library::cryptography::encryption::{aes256_crypto::Aes256Crypto, Encryption};

#[test]
fn known_secret_nonce() {
    let secret = [1_u8; 32];
    let mut crypto = Aes256Crypto::new(&secret.into());

    let data = [0, 1, 2, 3];
    let encrypted = crypto.encrypt(&data).unwrap();
    assert_ne!(vec![0, 1, 2, 3], encrypted);
    let decrypted = crypto.decrypt(&encrypted).unwrap();
    assert_eq!(vec![0, 1, 2, 3], decrypted);
}
