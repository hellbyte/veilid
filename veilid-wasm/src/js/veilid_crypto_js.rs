#![allow(non_snake_case)]
use super::*;

#[wasm_bindgen(js_name = veilidCrypto)]
pub struct VeilidCrypto {
    pub(crate) kind: CryptoKind,
}

#[wasm_bindgen(js_class = veilidCrypto)]
impl VeilidCrypto {
    // --------------------------------
    // Constants
    // (written as getters since wasm_bindgen doesn't support export of const)
    // --------------------------------

    /// The VLD0 crypto kind
    #[cfg(feature = "enable-crypto-vld0")]
    #[wasm_bindgen(getter, unchecked_return_type = "CryptoKind")]
    #[must_use]
    pub fn CRYPTO_KIND_VLD0() -> JsValue {
        crate::CRYPTO_KIND_VLD0.into()
    }

    /// The NONE crypto kind
    #[cfg(feature = "enable-crypto-none")]
    #[wasm_bindgen(getter, unchecked_return_type = "CryptoKind")]
    #[must_use]
    pub fn CRYPTO_KIND_NONE() -> JsValue {
        crate::CRYPTO_KIND_NONE.into()
    }

    // /// The VLD1 crypto kind
    // #[cfg(feature = "enable-crypto-vld1")]
    // #[wasm_bindgen(getter)]
    // #[must_use]
    // pub fn CRYPTO_KIND_VLD1() -> CryptoKind {
    //     CRYPTO_KIND_VLD1
    // }

    /// All crypto kinds supported by this configuration of Veilid
    #[wasm_bindgen(getter, unchecked_return_type = "CryptoKind[]")]
    #[must_use]
    pub fn VALID_CRYPTO_KINDS() -> JsValue {
        js_sys::Array::from_iter(
            crate::VALID_CRYPTO_KINDS
                .iter()
                .map(|x| JsValue::from(x.to_string())),
        )
        .into()
    }

    ////////////////////////////////////////////////////////////////////////////////

    #[wasm_bindgen(getter, unchecked_return_type = "CryptoKind")]
    #[must_use]
    pub fn kind(&self) -> JsValue {
        self.kind.into()
    }

    fn with_crypto_system<
        T,
        F: FnOnce(&(dyn CryptoSystem + Send + Sync + 'static)) -> VeilidAPIResult<T>,
    >(
        &self,
        closure: F,
    ) -> VeilidAPIResult<T> {
        let veilid_api = get_veilid_api()?;
        let crypto = veilid_api.crypto()?;
        let crypto_system = crypto.get(self.kind).ok_or_else(|| {
            VeilidAPIError::invalid_argument("with_crypto_system", "kind", self.kind.to_string())
        })?;
        closure(crypto_system.deref())
    }

    pub fn cachedDh(&self, key: &PublicKey, secret: &SecretKey) -> VeilidAPIResult<SharedSecret> {
        self.with_crypto_system(|crypto_system| {
            let out = crypto_system.cached_dh(key, secret)?;
            Ok(out)
        })
    }

    pub fn computeDh(&self, key: &PublicKey, secret: &SecretKey) -> VeilidAPIResult<SharedSecret> {
        self.with_crypto_system(|crypto_system| crypto_system.compute_dh(key, secret))
    }

    pub fn generateSharedSecret(
        &self,
        key: &PublicKey,
        secret: &SecretKey,
        domain: Box<[u8]>,
    ) -> VeilidAPIResult<SharedSecret> {
        self.with_crypto_system(|crypto_system| {
            crypto_system.generate_shared_secret(key, secret, &domain)
        })
    }

    pub fn randomBytes(&self, len: usize) -> VeilidAPIResult<Box<[u8]>> {
        self.with_crypto_system(|crypto_system| {
            let out = crypto_system.random_bytes(len);
            let out = out.into_boxed_slice();
            Ok(out)
        })
    }

    pub fn sharedSecretLength(&self) -> VeilidAPIResult<usize> {
        self.with_crypto_system(|crypto_system| {
            let out = crypto_system.shared_secret_length();
            Ok(out)
        })
    }

    pub fn nonceLength(&self) -> VeilidAPIResult<usize> {
        self.with_crypto_system(|crypto_system| {
            let out = crypto_system.nonce_length();
            Ok(out)
        })
    }

    pub fn hashDigestLength(&self) -> VeilidAPIResult<usize> {
        self.with_crypto_system(|crypto_system| {
            let out = crypto_system.hash_digest_length();
            Ok(out)
        })
    }

    pub fn publicKeyLength(&self) -> VeilidAPIResult<usize> {
        self.with_crypto_system(|crypto_system| {
            let out = crypto_system.public_key_length();
            Ok(out)
        })
    }

    pub fn secretKeyLength(&self) -> VeilidAPIResult<usize> {
        self.with_crypto_system(|crypto_system| {
            let out = crypto_system.secret_key_length();
            Ok(out)
        })
    }

    pub fn signatureLength(&self) -> VeilidAPIResult<usize> {
        self.with_crypto_system(|crypto_system| {
            let out = crypto_system.signature_length();
            Ok(out)
        })
    }

    pub fn defaultSaltLength(&self) -> VeilidAPIResult<usize> {
        self.with_crypto_system(|crypto_system| {
            let out = crypto_system.default_salt_length();
            Ok(out)
        })
    }

    pub fn aeadOverhead(&self) -> VeilidAPIResult<usize> {
        self.with_crypto_system(|crypto_system| {
            let out = crypto_system.aead_overhead();
            Ok(out)
        })
    }

    pub fn checkSharedSecret(&self, secret: &SharedSecret) -> VeilidAPIResult<()> {
        self.with_crypto_system(|crypto_system| crypto_system.check_shared_secret(secret))
    }

    pub fn checkNonce(&self, nonce: &Nonce) -> VeilidAPIResult<()> {
        self.with_crypto_system(|crypto_system| crypto_system.check_nonce(nonce))
    }

    pub fn checkHashDigest(&self, digest: &HashDigest) -> VeilidAPIResult<()> {
        self.with_crypto_system(|crypto_system| crypto_system.check_hash_digest(digest))
    }

    pub fn checkPublicKey(&self, key: &PublicKey) -> VeilidAPIResult<()> {
        self.with_crypto_system(|crypto_system| crypto_system.check_public_key(key))
    }

    pub fn checkSecretKey(&self, key: &SecretKey) -> VeilidAPIResult<()> {
        self.with_crypto_system(|crypto_system| crypto_system.check_secret_key(key))
    }

    pub fn checkSignature(&self, signature: &Signature) -> VeilidAPIResult<()> {
        self.with_crypto_system(|crypto_system| crypto_system.check_signature(signature))
    }

    pub fn hashPassword(&self, password: Box<[u8]>, salt: Box<[u8]>) -> VeilidAPIResult<String> {
        self.with_crypto_system(|crypto_system| crypto_system.hash_password(&password, &salt))
    }

    pub fn verifyPassword(
        &self,
        password: Box<[u8]>,
        password_hash: String,
    ) -> VeilidAPIResult<bool> {
        self.with_crypto_system(|crypto_system| {
            crypto_system.verify_password(&password, &password_hash)
        })
    }

    pub fn deriveSharedSecret(
        &self,
        password: Box<[u8]>,
        salt: Box<[u8]>,
    ) -> VeilidAPIResult<SharedSecret> {
        self.with_crypto_system(|crypto_system| {
            crypto_system.derive_shared_secret(&password, &salt)
        })
    }

    pub fn randomNonce(&self) -> VeilidAPIResult<Nonce> {
        self.with_crypto_system(|crypto_system| {
            let out = crypto_system.random_nonce();
            Ok(out)
        })
    }

    pub fn randomSharedSecret(&self) -> VeilidAPIResult<SharedSecret> {
        self.with_crypto_system(|crypto_system| {
            let out = crypto_system.random_shared_secret();
            Ok(out)
        })
    }

    pub fn generateKeyPair(&self) -> VeilidAPIResult<KeyPair> {
        self.with_crypto_system(|crypto_system| {
            let out = crypto_system.generate_keypair();
            Ok(out)
        })
    }

    pub fn generateHash(&self, data: Box<[u8]>) -> VeilidAPIResult<HashDigest> {
        self.with_crypto_system(|crypto_system| {
            let out = crypto_system.generate_hash(&data);
            Ok(out)
        })
    }

    pub fn validateKeyPair(&self, key: &PublicKey, secret: &SecretKey) -> VeilidAPIResult<bool> {
        self.with_crypto_system(|crypto_system| crypto_system.validate_keypair(key, secret))
    }

    pub fn validateHash(&self, data: Box<[u8]>, hash: &HashDigest) -> VeilidAPIResult<bool> {
        self.with_crypto_system(|crypto_system| crypto_system.validate_hash(&data, hash))
    }

    pub fn sign(
        &self,
        key: &PublicKey,
        secret: &SecretKey,
        data: Box<[u8]>,
    ) -> VeilidAPIResult<Signature> {
        self.with_crypto_system(|crypto_system| crypto_system.sign(key, secret, &data))
    }

    pub fn verify(
        &self,
        key: &PublicKey,
        data: Box<[u8]>,
        signature: &Signature,
    ) -> VeilidAPIResult<bool> {
        self.with_crypto_system(|crypto_system| crypto_system.verify(key, &data, signature))
    }

    pub fn decryptAead(
        &self,
        body: Box<[u8]>,
        nonce: &Nonce,
        shared_secret: &SharedSecret,
        associated_data: Option<Box<[u8]>>,
    ) -> VeilidAPIResult<Box<[u8]>> {
        self.with_crypto_system(|crypto_system| {
            let out = crypto_system.decrypt_aead(
                &body,
                nonce,
                shared_secret,
                match &associated_data {
                    Some(ad) => Some(ad),
                    None => None,
                },
            )?;
            let out = out.into_boxed_slice();
            Ok(out)
        })
    }

    pub fn encryptAead(
        &self,
        body: Box<[u8]>,
        nonce: &Nonce,
        shared_secret: &SharedSecret,
        associated_data: Option<Box<[u8]>>,
    ) -> VeilidAPIResult<Box<[u8]>> {
        self.with_crypto_system(|crypto_system| {
            let out = crypto_system.encrypt_aead(
                &body,
                nonce,
                shared_secret,
                match &associated_data {
                    Some(ad) => Some(ad),
                    None => None,
                },
            )?;
            Ok(out.into_boxed_slice())
        })
    }

    pub fn cryptNoAuth(
        &self,
        mut body: Box<[u8]>,
        nonce: &Nonce,
        shared_secret: &SharedSecret,
    ) -> VeilidAPIResult<Box<[u8]>> {
        self.with_crypto_system(|crypto_system| {
            crypto_system.crypt_in_place_no_auth(&mut body, nonce, shared_secret)?;
            Ok(body)
        })
    }
}
