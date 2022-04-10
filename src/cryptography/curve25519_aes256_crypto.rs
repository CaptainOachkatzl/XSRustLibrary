use aes_gcm::{aead::{generic_array::GenericArray, Aead, Payload}, Aes256Gcm, NewAead};
use rand_core::{OsRng, RngCore};
use x25519_dalek::{EphemeralSecret, PublicKey};

static HANDSHAKE_CONFIRMATION: &'static [u8] =
  &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

pub struct Ec25519Aes256Crypto;

impl Ec25519Aes256Crypto {
  pub fn handshake_active(
    mut send: Box<dyn FnMut(&[u8])>,
    mut receive: Box<dyn FnMut() -> Vec<u8>>,
  ) -> bool {
    let mut rng = OsRng;

    let secret = EphemeralSecret::new(rng);
    let public_key = PublicKey::from(&secret);
    // send public key
    send(public_key.as_bytes());
    // receive opposing public key
    let mut other_pub_key: [u8; 32] = [0 as u8; 32];
    other_pub_key.clone_from_slice(&receive());
    // calculate shared secret
    let shared_secret = secret.diffie_hellman(&PublicKey::from(other_pub_key));
    // set shared secret in aes
    let cipher = Aes256Gcm::new(GenericArray::from_slice(shared_secret.as_bytes()));
    // send IV
    let mut nonce_bytes = [0 as u8; 12];
    rng.fill_bytes(&mut nonce_bytes);
    send(&nonce_bytes);
    let nonce = GenericArray::from_slice(&nonce_bytes);

    // receive secret
    let payload = Payload {
      msg: &receive(),
      aad: &[0 as u8; 0],
    };

    // assert secret
    let decrypted_confirmation = cipher.decrypt(nonce, payload).unwrap();
    return decrypted_confirmation == HANDSHAKE_CONFIRMATION;
  }

  pub fn handshake_passive(
    mut send: Box<dyn FnMut(&[u8])>,
    mut receive: Box<dyn FnMut() -> Vec<u8>>,
  ) -> bool {
    let secret = EphemeralSecret::new(OsRng);
    let public_key = PublicKey::from(&secret);
    // receive pub key
    let mut other_pub_key: [u8; 32] = [0 as u8; 32];
    other_pub_key.clone_from_slice(&receive());
    // send public key
    send(public_key.as_bytes());
    // calculate shared secret
    let shared_secret = secret.diffie_hellman(&PublicKey::from(other_pub_key));
    // set shared secret in aes
    let cipher = Aes256Gcm::new(GenericArray::from_slice(shared_secret.as_bytes()));
    // receive iv/nonce
    let mut nonce_bytes = [0 as u8; 12];
    nonce_bytes.clone_from_slice(&receive());
    let nonce = GenericArray::from_slice(&nonce_bytes);
    // send encryped secret
    let payload = Payload {
      msg: HANDSHAKE_CONFIRMATION,
      aad: &[0 as u8; 0],
    };

    // assert secret
    let encryped_confirmation = cipher.encrypt(nonce, payload).unwrap();
    send(&encryped_confirmation);
    return true;
  }
}
