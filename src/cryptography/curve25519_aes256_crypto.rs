use rand_core::{OsRng, RngCore};
use x25519_dalek::{EphemeralSecret, PublicKey};
use aes::Aes256;
use aes::cipher::{KeyIvInit, block_padding::Pkcs7};
use aes::cipher::{BlockEncryptMut, BlockDecryptMut};

type Aes256Enc = cbc::Encryptor<Aes256>;
type Aes256Dec = cbc::Decryptor<Aes256>;

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
    let mut nonce = [0 as u8; 16];
    rng.fill_bytes(&mut nonce);
    let cipher = Aes256Dec::new(shared_secret.as_bytes().into(), &nonce.into());
    // send IV
    send(&nonce);

    // receive secret
    let mut confirmation_block = [0 as u8; 32];
    confirmation_block.copy_from_slice(&receive());

    // assert secret
    cipher.decrypt_padded_mut::<Pkcs7>(&mut confirmation_block).unwrap();
    return &confirmation_block[..16] == HANDSHAKE_CONFIRMATION;
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
    // receive iv/nonce
    let mut nonce = [0 as u8; 16];
    nonce.clone_from_slice(&receive());
    // set shared secret in aes
    let cipher = Aes256Enc::new(shared_secret.as_bytes().into(), &nonce.into());

    let mut confirmation_block = [0 as u8; 32];
    confirmation_block[..16].copy_from_slice(HANDSHAKE_CONFIRMATION);
    // send encryped secret
    let _result = cipher.encrypt_padded_mut::<Pkcs7>(&mut confirmation_block, 16);
    send(&confirmation_block);

    return true;
  }
}
