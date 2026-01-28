use super::*;
mod blake3digest512;

#[cfg(feature = "enable-crypto-none")]
pub(crate) mod none;
#[cfg(feature = "enable-crypto-vld0")]
pub(crate) mod vld0;
// #[cfg(feature = "enable-crypto-vld1")]
// pub(crate) mod vld1;

pub(crate) const VEILID_DOMAIN_API: &[u8] = b"VEILID_API";

#[cfg(feature = "enable-crypto-none")]
pub use none::sizes::*;
#[cfg(feature = "enable-crypto-none")]
pub use none::*;
#[cfg(feature = "enable-crypto-vld0")]
pub use vld0::sizes::*;
#[cfg(feature = "enable-crypto-vld0")]
pub use vld0::*;
// #[cfg(feature = "enable-crypto-vld1")]
// pub use vld1::*;

pub use blake3digest512::*;

pub trait CryptoSystem {
    // Accessors
    fn kind(&self) -> CryptoKind;
    fn crypto(&self) -> VeilidComponentGuard<'_, Crypto>;

    // Cached Operations
    fn cached_dh(&self, key: &PublicKey, secret: &SecretKey) -> VeilidAPIResult<SharedSecret>;

    // Generation
    fn random_bytes(&self, len: usize) -> Vec<u8>;
    fn hash_password(&self, password: &[u8], salt: &[u8]) -> VeilidAPIResult<String>;
    fn verify_password(&self, password: &[u8], password_hash: &str) -> VeilidAPIResult<bool>;
    fn derive_shared_secret(&self, password: &[u8], salt: &[u8]) -> VeilidAPIResult<SharedSecret>;
    fn random_nonce(&self) -> Nonce;
    fn random_shared_secret(&self) -> SharedSecret;
    fn compute_dh(&self, key: &PublicKey, secret: &SecretKey) -> VeilidAPIResult<SharedSecret>;
    fn generate_shared_secret(
        &self,
        key: &PublicKey,
        secret: &SecretKey,
        domain: &[u8],
    ) -> VeilidAPIResult<SharedSecret> {
        let dh = self.compute_dh(key, secret)?;
        let hash = self.generate_hash(&[&dh.into_value(), domain, VEILID_DOMAIN_API].concat());
        Ok(SharedSecret::new(
            hash.kind(),
            BareSharedSecret::new(&hash.into_value()),
        ))
    }
    fn generate_keypair(&self) -> KeyPair;
    fn generate_hash(&self, data: &[u8]) -> HashDigest;
    fn generate_hash_reader(&self, reader: &mut dyn std::io::Read) -> VeilidAPIResult<PublicKey>;

    // Validation
    fn shared_secret_length(&self) -> usize;
    fn nonce_length(&self) -> usize;
    fn hash_digest_length(&self) -> usize;
    fn public_key_length(&self) -> usize;
    fn secret_key_length(&self) -> usize;
    fn signature_length(&self) -> usize;
    fn default_salt_length(&self) -> usize;
    fn aead_overhead(&self) -> usize;

    fn check_shared_secret(&self, secret: &SharedSecret) -> VeilidAPIResult<()> {
        if secret.kind() != self.kind() {
            apibail_generic!("incorrect shared secret kind");
        }
        if secret.value().len() != self.shared_secret_length() {
            apibail_generic!(
                "invalid shared secret length: {} != {}",
                secret.value().len(),
                self.shared_secret_length()
            );
        }
        Ok(())
    }
    fn check_nonce(&self, nonce: &Nonce) -> VeilidAPIResult<()> {
        if nonce.len() != self.nonce_length() {
            apibail_generic!(
                "invalid nonce length: {} != {}",
                nonce.len(),
                self.nonce_length()
            );
        }
        Ok(())
    }
    fn check_hash_digest(&self, hash: &HashDigest) -> VeilidAPIResult<()> {
        if hash.kind() != self.kind() {
            apibail_generic!("incorrect hash digest kind");
        }
        if hash.value().len() != self.hash_digest_length() {
            apibail_generic!(
                "invalid hash digest length: {} != {}",
                hash.value().len(),
                self.hash_digest_length()
            );
        }
        Ok(())
    }
    fn check_public_key(&self, key: &PublicKey) -> VeilidAPIResult<()> {
        if key.kind() != self.kind() {
            apibail_generic!("incorrect public key kind");
        }
        if key.value().len() != self.public_key_length() {
            apibail_generic!(
                "invalid public key length: {} != {}",
                key.value().len(),
                self.public_key_length()
            );
        }
        Ok(())
    }
    fn check_secret_key(&self, key: &SecretKey) -> VeilidAPIResult<()> {
        if key.kind() != self.kind() {
            apibail_generic!("incorrect secret key kind");
        }
        if key.value().len() != self.secret_key_length() {
            apibail_generic!(
                "invalid secret key length: {} != {}",
                key.value().len(),
                self.secret_key_length()
            );
        }
        Ok(())
    }
    fn check_signature(&self, signature: &Signature) -> VeilidAPIResult<()> {
        if signature.kind() != self.kind() {
            apibail_generic!("incorrect signature kind");
        }
        if signature.value().len() != self.signature_length() {
            apibail_generic!(
                "invalid signature length: {} != {}",
                signature.value().len(),
                self.signature_length()
            );
        }
        Ok(())
    }
    fn check_keypair(&self, keypair: &KeyPair) -> VeilidAPIResult<()> {
        if keypair.kind() != self.kind() {
            apibail_generic!("incorrect keypair kind");
        }
        self.check_public_key(&keypair.key())?;
        self.check_secret_key(&keypair.secret())?;
        Ok(())
    }

    fn validate_keypair(&self, key: &PublicKey, secret: &SecretKey) -> VeilidAPIResult<bool>;
    fn validate_hash(&self, data: &[u8], hash: &HashDigest) -> VeilidAPIResult<bool>;
    fn validate_hash_reader(
        &self,
        reader: &mut dyn std::io::Read,
        hash: &HashDigest,
    ) -> VeilidAPIResult<bool>;

    // Authentication
    fn sign(&self, key: &PublicKey, secret: &SecretKey, data: &[u8]) -> VeilidAPIResult<Signature>;
    fn verify(&self, key: &PublicKey, data: &[u8], signature: &Signature) -> VeilidAPIResult<bool>;

    // AEAD Encrypt/Decrypt
    fn decrypt_in_place_aead(
        &self,
        body: &mut Vec<u8>,
        nonce: &Nonce,
        shared_secret: &SharedSecret,
        associated_data: Option<&[u8]>,
    ) -> VeilidAPIResult<()>;
    fn decrypt_aead(
        &self,
        body: &[u8],
        nonce: &Nonce,
        shared_secret: &SharedSecret,
        associated_data: Option<&[u8]>,
    ) -> VeilidAPIResult<Vec<u8>>;
    fn encrypt_in_place_aead(
        &self,
        body: &mut Vec<u8>,
        nonce: &Nonce,
        shared_secret: &SharedSecret,
        associated_data: Option<&[u8]>,
    ) -> VeilidAPIResult<()>;
    fn encrypt_aead(
        &self,
        body: &[u8],
        nonce: &Nonce,
        shared_secret: &SharedSecret,
        associated_data: Option<&[u8]>,
    ) -> VeilidAPIResult<Vec<u8>>;

    // NoAuth Encrypt/Decrypt
    fn crypt_in_place_no_auth(
        &self,
        body: &mut [u8],
        nonce: &Nonce,
        shared_secret: &SharedSecret,
    ) -> VeilidAPIResult<()>;
    fn crypt_b2b_no_auth(
        &self,
        in_buf: &[u8],
        out_buf: &mut [u8],
        nonce: &Nonce,
        shared_secret: &SharedSecret,
    ) -> VeilidAPIResult<()>;
    fn crypt_no_auth_aligned_8(
        &self,
        body: &[u8],
        nonce: &Nonce,
        shared_secret: &SharedSecret,
    ) -> VeilidAPIResult<Vec<u8>>;
    fn crypt_no_auth_unaligned(
        &self,
        body: &[u8],
        nonce: &Nonce,
        shared_secret: &SharedSecret,
    ) -> VeilidAPIResult<Vec<u8>>;
}
