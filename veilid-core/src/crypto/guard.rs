use core::marker::PhantomData;

use super::*;

/// Guard to access a particular cryptosystem
#[must_use]
pub struct CryptoSystemGuard<'a> {
    crypto_system: Arc<dyn CryptoSystem + Send + Sync>,
    _phantom: core::marker::PhantomData<&'a (dyn CryptoSystem + Send + Sync)>,
}

impl<'a> CryptoSystemGuard<'a> {
    pub(super) fn new(crypto_system: Arc<dyn CryptoSystem + Send + Sync>) -> Self {
        Self {
            crypto_system,
            _phantom: PhantomData,
        }
    }
    pub fn as_async(self) -> AsyncCryptoSystemGuard<'a> {
        AsyncCryptoSystemGuard { guard: self }
    }
}

impl core::ops::Deref for CryptoSystemGuard<'_> {
    type Target = dyn CryptoSystem + Send + Sync;

    fn deref(&self) -> &Self::Target {
        self.crypto_system.as_ref()
    }
}

/// Async cryptosystem guard to help break up heavy blocking operations
#[must_use]
pub struct AsyncCryptoSystemGuard<'a> {
    guard: CryptoSystemGuard<'a>,
}

async fn yielding<R, T: FnOnce() -> R>(x: T) -> R {
    let out = x();
    sleep(0).await;
    out
}

impl AsyncCryptoSystemGuard<'_> {
    // Accessors
    pub fn kind(&self) -> CryptoKind {
        self.guard.kind()
    }
    #[must_use]
    pub fn crypto(&self) -> VeilidComponentGuard<'_, Crypto> {
        self.guard.crypto()
    }

    // Cached Operations
    pub async fn cached_dh(
        &self,
        key: &PublicKey,
        secret: &SecretKey,
    ) -> VeilidAPIResult<SharedSecret> {
        yielding(|| self.guard.cached_dh(key, secret)).await
    }

    // Generation
    pub async fn random_bytes(&self, len: u32) -> Vec<u8> {
        yielding(|| self.guard.random_bytes(len)).await
    }

    pub async fn hash_password(&self, password: &[u8], salt: &[u8]) -> VeilidAPIResult<String> {
        yielding(|| self.guard.hash_password(password, salt)).await
    }
    pub async fn verify_password(
        &self,
        password: &[u8],
        password_hash: &str,
    ) -> VeilidAPIResult<bool> {
        yielding(|| self.guard.verify_password(password, password_hash)).await
    }
    pub async fn derive_shared_secret(
        &self,
        password: &[u8],
        salt: &[u8],
    ) -> VeilidAPIResult<SharedSecret> {
        yielding(|| self.guard.derive_shared_secret(password, salt)).await
    }
    pub async fn random_nonce(&self) -> Nonce {
        yielding(|| self.guard.random_nonce()).await
    }
    pub async fn random_shared_secret(&self) -> SharedSecret {
        yielding(|| self.guard.random_shared_secret()).await
    }
    pub async fn compute_dh(
        &self,
        key: &PublicKey,
        secret: &SecretKey,
    ) -> VeilidAPIResult<SharedSecret> {
        yielding(|| self.guard.compute_dh(key, secret)).await
    }
    pub async fn generate_shared_secret(
        &self,
        key: &PublicKey,
        secret: &SecretKey,
        domain: &[u8],
    ) -> VeilidAPIResult<SharedSecret> {
        let dh = self.compute_dh(key, secret).await?;
        let hash = self
            .generate_hash(&[&dh.into_value(), domain, VEILID_DOMAIN_API].concat())
            .await;
        Ok(SharedSecret::new(
            hash.kind(),
            BareSharedSecret::new(&hash.into_value()),
        ))
    }

    pub async fn generate_keypair(&self) -> KeyPair {
        yielding(|| self.guard.generate_keypair()).await
    }

    pub async fn generate_hash(&self, data: &[u8]) -> HashDigest {
        yielding(|| self.guard.generate_hash(data)).await
    }

    pub async fn generate_hash_reader(
        &self,
        reader: &mut dyn std::io::Read,
    ) -> VeilidAPIResult<PublicKey> {
        yielding(|| self.guard.generate_hash_reader(reader)).await
    }

    // Validation
    #[must_use]
    pub fn shared_secret_length(&self) -> usize {
        self.guard.shared_secret_length()
    }
    #[must_use]
    pub fn nonce_length(&self) -> usize {
        self.guard.nonce_length()
    }
    #[must_use]
    pub fn hash_digest_length(&self) -> usize {
        self.guard.hash_digest_length()
    }
    #[must_use]
    pub fn public_key_length(&self) -> usize {
        self.guard.public_key_length()
    }
    #[must_use]
    pub fn secret_key_length(&self) -> usize {
        self.guard.secret_key_length()
    }
    #[must_use]
    pub fn signature_length(&self) -> usize {
        self.guard.signature_length()
    }
    #[must_use]
    pub fn aead_overhead(&self) -> usize {
        self.guard.aead_overhead()
    }
    #[must_use]
    pub fn default_salt_length(&self) -> usize {
        self.guard.default_salt_length()
    }
    pub fn check_shared_secret(&self, secret: &SharedSecret) -> VeilidAPIResult<()> {
        self.guard.check_shared_secret(secret)
    }
    pub fn check_nonce(&self, nonce: &Nonce) -> VeilidAPIResult<()> {
        self.guard.check_nonce(nonce)
    }
    pub fn check_hash_digest(&self, hash: &HashDigest) -> VeilidAPIResult<()> {
        self.guard.check_hash_digest(hash)
    }
    pub fn check_public_key(&self, key: &PublicKey) -> VeilidAPIResult<()> {
        self.guard.check_public_key(key)
    }
    pub fn check_secret_key(&self, key: &SecretKey) -> VeilidAPIResult<()> {
        self.guard.check_secret_key(key)
    }
    pub fn check_signature(&self, signature: &Signature) -> VeilidAPIResult<()> {
        self.guard.check_signature(signature)
    }
    pub async fn validate_keypair(
        &self,
        key: &PublicKey,
        secret: &SecretKey,
    ) -> VeilidAPIResult<bool> {
        yielding(|| self.guard.validate_keypair(key, secret)).await
    }

    pub async fn validate_hash(&self, data: &[u8], hash: &HashDigest) -> VeilidAPIResult<bool> {
        yielding(|| self.guard.validate_hash(data, hash)).await
    }

    pub async fn validate_hash_reader(
        &self,
        reader: &mut dyn std::io::Read,
        hash: &HashDigest,
    ) -> VeilidAPIResult<bool> {
        yielding(|| self.guard.validate_hash_reader(reader, hash)).await
    }

    // Authentication
    pub async fn sign(
        &self,
        key: &PublicKey,
        secret: &SecretKey,
        data: &[u8],
    ) -> VeilidAPIResult<Signature> {
        yielding(|| self.guard.sign(key, secret, data)).await
    }
    pub async fn verify(
        &self,
        key: &PublicKey,
        data: &[u8],
        signature: &Signature,
    ) -> VeilidAPIResult<bool> {
        yielding(|| self.guard.verify(key, data, signature)).await
    }

    // AEAD Encrypt/Decrypt
    pub async fn decrypt_in_place_aead(
        &self,
        body: &mut Vec<u8>,
        nonce: &Nonce,
        shared_secret: &SharedSecret,
        associated_data: Option<&[u8]>,
    ) -> VeilidAPIResult<()> {
        yielding(|| {
            self.guard
                .decrypt_in_place_aead(body, nonce, shared_secret, associated_data)
        })
        .await
    }

    pub async fn decrypt_aead(
        &self,
        body: &[u8],
        nonce: &Nonce,
        shared_secret: &SharedSecret,
        associated_data: Option<&[u8]>,
    ) -> VeilidAPIResult<Vec<u8>> {
        yielding(|| {
            self.guard
                .decrypt_aead(body, nonce, shared_secret, associated_data)
        })
        .await
    }

    pub async fn encrypt_in_place_aead(
        &self,
        body: &mut Vec<u8>,
        nonce: &Nonce,
        shared_secret: &SharedSecret,
        associated_data: Option<&[u8]>,
    ) -> VeilidAPIResult<()> {
        yielding(|| {
            self.guard
                .encrypt_in_place_aead(body, nonce, shared_secret, associated_data)
        })
        .await
    }

    pub async fn encrypt_aead(
        &self,
        body: &[u8],
        nonce: &Nonce,
        shared_secret: &SharedSecret,
        associated_data: Option<&[u8]>,
    ) -> VeilidAPIResult<Vec<u8>> {
        yielding(|| {
            self.guard
                .encrypt_aead(body, nonce, shared_secret, associated_data)
        })
        .await
    }

    // NoAuth Encrypt/Decrypt
    pub async fn crypt_in_place_no_auth(
        &self,
        body: &mut [u8],
        nonce: &Nonce,
        shared_secret: &SharedSecret,
    ) -> VeilidAPIResult<()> {
        yielding(|| {
            self.guard
                .crypt_in_place_no_auth(body, nonce, shared_secret)
        })
        .await
    }

    pub async fn crypt_b2b_no_auth(
        &self,
        in_buf: &[u8],
        out_buf: &mut [u8],
        nonce: &Nonce,
        shared_secret: &SharedSecret,
    ) -> VeilidAPIResult<()> {
        yielding(|| {
            self.guard
                .crypt_b2b_no_auth(in_buf, out_buf, nonce, shared_secret)
        })
        .await
    }

    pub async fn crypt_no_auth_aligned_8(
        &self,
        body: &[u8],
        nonce: &Nonce,
        shared_secret: &SharedSecret,
    ) -> VeilidAPIResult<Vec<u8>> {
        yielding(|| {
            self.guard
                .crypt_no_auth_aligned_8(body, nonce, shared_secret)
        })
        .await
    }

    pub async fn crypt_no_auth_unaligned(
        &self,
        body: &[u8],
        nonce: &Nonce,
        shared_secret: &SharedSecret,
    ) -> VeilidAPIResult<Vec<u8>> {
        yielding(|| {
            self.guard
                .crypt_no_auth_unaligned(body, nonce, shared_secret)
        })
        .await
    }
}
