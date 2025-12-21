mod dh_cache;
mod envelope;
mod guard;
mod receipt;
mod types;

pub mod crypto_system;
#[doc(hidden)]
pub mod tests;

pub use crypto_system::*;
use dh_cache::*;
pub(crate) use envelope::*;
pub use guard::*;
pub(crate) use receipt::*;
pub use types::*;

use super::*;
use core::convert::TryInto;
use hashlink::linked_hash_map::Entry;
use hashlink::LruCache;

impl_veilid_log_facility!("crypto");

cfg_if! {
    if #[cfg(all(feature = "enable-crypto-none", feature = "enable-crypto-vld0"))] {
        /// Crypto kinds in order of preference, best cryptosystem is the first one, worst is the last one
        pub const VALID_CRYPTO_KINDS: [CryptoKind; 2] = [CRYPTO_KIND_VLD0, CRYPTO_KIND_NONE];
    }
    else if #[cfg(feature = "enable-crypto-none")] {
        /// Crypto kinds in order of preference, best cryptosystem is the first one, worst is the last one
        pub const VALID_CRYPTO_KINDS: [CryptoKind; 1] = [CRYPTO_KIND_NONE];
    }
    else if #[cfg(feature = "enable-crypto-vld0")] {
        /// Crypto kinds in order of preference, best cryptosystem is the first one, worst is the last one
        pub const VALID_CRYPTO_KINDS: [CryptoKind; 1] = [CRYPTO_KIND_VLD0];
    }
    // else if #[cfg(feature = "enable-crypto-vld1")] {
    //     /// Crypto kinds in order of preference, best cryptosystem is the first one, worst is the last one
    //     pub const VALID_CRYPTO_KINDS: [CryptoKind; 2] = [CRYPTO_KIND_VLD1, CRYPTO_KIND_VLD0];
    // }
    else {
        compile_error!("No crypto kinds enabled, specify an enable-crypto- feature");
    }
}
/// Number of cryptosystem signatures to keep on structures if many are present beyond the ones we consider valid
pub const MAX_CRYPTO_KINDS: usize = 3;

/// Return the best cryptosystem kind we support
pub(crate) fn best_crypto_kind() -> CryptoKind {
    VALID_CRYPTO_KINDS[0]
}

struct CryptoInner {
    dh_cache: DHCache,
    dh_cache_misses: usize,
    dh_cache_hits: usize,
    dh_cache_lru: usize,
}

impl fmt::Debug for CryptoInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CryptoInner")
            //.field("dh_cache", &self.dh_cache)
            .field("dh_cache_misses", &self.dh_cache_misses)
            .field("dh_cache_hits", &self.dh_cache_hits)
            .field("dh_cache_lru", &self.dh_cache_lru)
            // .field("crypto_vld0", &self.crypto_vld0)
            // .field("crypto_none", &self.crypto_none)
            .finish()
    }
}

/// Crypto factory implementation
#[must_use]
pub struct Crypto {
    registry: VeilidComponentRegistry,
    inner: Mutex<CryptoInner>,
    #[cfg(feature = "enable-crypto-vld0")]
    crypto_vld0: Arc<dyn CryptoSystem + Send + Sync>,
    #[cfg(feature = "enable-crypto-none")]
    crypto_none: Arc<dyn CryptoSystem + Send + Sync>,
}

impl_veilid_component!(Crypto);

impl fmt::Debug for Crypto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Crypto")
            //.field("registry", &self.registry)
            .field("inner", &self.inner)
            // .field("crypto_vld0", &self.crypto_vld0)
            // .field("crypto_none", &self.crypto_none)
            .finish()
    }
}

impl Crypto {
    fn new_inner() -> CryptoInner {
        CryptoInner {
            dh_cache: DHCache::new(DH_CACHE_SIZE),
            dh_cache_misses: 0,
            dh_cache_hits: 0,
            dh_cache_lru: 0,
        }
    }

    pub(crate) fn new(registry: VeilidComponentRegistry) -> Self {
        Self {
            registry: registry.clone(),
            inner: Mutex::new(Self::new_inner()),
            #[cfg(feature = "enable-crypto-vld0")]
            crypto_vld0: Arc::new(vld0::CryptoSystemVLD0::new(registry.clone())),
            #[cfg(feature = "enable-crypto-none")]
            crypto_none: Arc::new(none::CryptoSystemNONE::new(registry.clone())),
        }
    }

    #[instrument(level = "trace", target = "crypto", skip_all, err)]
    async fn init_async(&self) -> EyreResult<()> {
        // Nothing to initialize at this time
        Ok(())
    }

    // Setup called by table store after it get initialized
    #[instrument(level = "trace", target = "crypto", skip_all, err)]
    pub(crate) async fn table_store_setup(&self, table_store: &TableStore) -> EyreResult<()> {
        // load caches if they are valid for this node id
        let caches_valid = {
            let db = table_store
                .open("crypto_caches", 1)
                .await
                .wrap_err("failed to open crypto_caches")?;

            let mut caches_valid = true;
            if let Some(b) = db.load(0, b"dh_cache").await? {
                let mut inner = self.inner.lock();
                if let Ok(dh_cache) = bytes_to_cache(&b) {
                    inner.dh_cache = dh_cache;
                } else {
                    caches_valid = false;
                }
            }

            caches_valid
        };

        if !caches_valid {
            table_store.delete("crypto_caches").await?;
        }

        Ok(())
    }

    #[instrument(level = "trace", target = "crypto", skip_all, err)]
    async fn post_init_async(&self) -> EyreResult<()> {
        Ok(())
    }

    pub async fn flush(&self) -> EyreResult<()> {
        let cache_bytes = {
            let inner = self.inner.lock();
            cache_to_bytes(&inner.dh_cache)
        };

        let db = self.table_store().open("crypto_caches", 1).await?;
        db.store(0, b"dh_cache", &cache_bytes).await?;
        Ok(())
    }

    async fn pre_terminate_async(&self) {
        veilid_log!(self trace "starting termination flush");
        match self.flush().await {
            Ok(_) => {
                veilid_log!(self trace "finished termination flush");
            }
            Err(e) => {
                error!("failed termination flush: {}", e);
            }
        };
    }

    #[expect(clippy::unused_async)]
    async fn terminate_async(&self) {
        // Nothing to terminate at this time
    }

    /// Factory method to get a specific crypto version
    pub fn get(&self, kind: CryptoKind) -> Option<CryptoSystemGuard<'_>> {
        match kind {
            #[cfg(feature = "enable-crypto-vld0")]
            CRYPTO_KIND_VLD0 => Some(CryptoSystemGuard::new(self.crypto_vld0.clone())),
            #[cfg(feature = "enable-crypto-none")]
            CRYPTO_KIND_NONE => Some(CryptoSystemGuard::new(self.crypto_none.clone())),
            _ => None,
        }
    }

    /// Factory method to get a specific crypto version for async use
    pub fn get_async(&self, kind: CryptoKind) -> Option<AsyncCryptoSystemGuard<'_>> {
        self.get(kind).map(|x| x.as_async())
    }

    // Factory method to get the best crypto version
    pub(crate) fn best(&self) -> CryptoSystemGuard<'_> {
        self.get(best_crypto_kind()).unwrap()
    }

    // Factory method to get the best crypto version for async use
    pub(crate) fn best_async(&self) -> AsyncCryptoSystemGuard<'_> {
        self.get_async(best_crypto_kind()).unwrap()
    }

    // Convenience validators
    pub fn check_shared_secret(&self, secret: &SharedSecret) -> VeilidAPIResult<()> {
        let Some(vcrypto) = self.get(secret.kind()) else {
            apibail_generic!("unsupported crypto kind");
        };
        vcrypto.check_shared_secret(secret)
    }

    pub fn check_hash_digest(&self, hash: &HashDigest) -> VeilidAPIResult<()> {
        let Some(vcrypto) = self.get(hash.kind()) else {
            apibail_generic!("unsupported crypto kind");
        };
        vcrypto.check_hash_digest(hash)
    }
    pub fn check_public_key(&self, key: &PublicKey) -> VeilidAPIResult<()> {
        let Some(vcrypto) = self.get(key.kind()) else {
            apibail_generic!("unsupported crypto kind");
        };
        vcrypto.check_public_key(key)
    }
    pub fn check_secret_key(&self, key: &SecretKey) -> VeilidAPIResult<()> {
        let Some(vcrypto) = self.get(key.kind()) else {
            apibail_generic!("unsupported crypto kind");
        };
        vcrypto.check_secret_key(key)
    }
    pub fn check_signature(&self, signature: &Signature) -> VeilidAPIResult<()> {
        let Some(vcrypto) = self.get(signature.kind()) else {
            apibail_generic!("unsupported crypto kind");
        };
        vcrypto.check_signature(signature)
    }
    pub fn check_keypair(&self, key_pair: &KeyPair) -> VeilidAPIResult<()> {
        let Some(vcrypto) = self.get(key_pair.kind()) else {
            apibail_generic!("unsupported crypto kind");
        };
        vcrypto.check_keypair(key_pair)
    }

    /// BareSignature set verification
    /// Returns Some() the set of signature cryptokinds that validate and are supported
    /// Returns None if any cryptokinds are supported and do not validate
    pub fn verify_signatures(
        &self,
        public_keys: &[PublicKey],
        data: &[u8],
        signatures: &[Signature],
    ) -> VeilidAPIResult<Option<PublicKeyGroup>> {
        let mut out = PublicKeyGroup::with_capacity(public_keys.len());
        for signature in signatures {
            for public_key in public_keys {
                if public_key.kind() == signature.kind() {
                    if let Some(vcrypto) = self.get(signature.kind()) {
                        if !vcrypto.verify(public_key, data, signature)? {
                            return Ok(None);
                        }
                        out.add(public_key.clone());
                    }
                }
            }
        }
        Ok(Some(out))
    }

    /// BareSignature set generation
    /// Generates the set of signatures that are supported
    /// Any cryptokinds that are not supported are silently dropped
    pub fn generate_signatures<F, R>(
        &self,
        data: &[u8],
        key_pairs: &[KeyPair],
        transform: F,
    ) -> VeilidAPIResult<Vec<R>>
    where
        F: Fn(&KeyPair, Signature) -> R,
    {
        let mut out = Vec::<R>::with_capacity(key_pairs.len());
        for kp in key_pairs {
            if let Some(vcrypto) = self.get(kp.kind()) {
                let sig = vcrypto.sign(&kp.key(), &kp.secret(), data)?;
                out.push(transform(kp, sig))
            }
        }
        Ok(out)
    }

    /// Generate keypair
    /// Does not require startup/init
    pub fn generate_keypair(crypto_kind: CryptoKind) -> VeilidAPIResult<KeyPair> {
        #[cfg(feature = "enable-crypto-vld0")]
        if crypto_kind == CRYPTO_KIND_VLD0 {
            let kp = vld0_generate_keypair();
            return Ok(kp);
        }
        #[cfg(feature = "enable-crypto-none")]
        if crypto_kind == CRYPTO_KIND_NONE {
            let kp = none_generate_keypair();
            return Ok(kp);
        }
        Err(VeilidAPIError::generic("invalid crypto kind"))
    }

    // Internal utilities

    fn cached_dh_internal<T: CryptoSystem>(
        &self,
        vcrypto: &T,
        key: &PublicKey,
        secret: &SecretKey,
    ) -> VeilidAPIResult<SharedSecret> {
        vcrypto.check_public_key(key)?;
        vcrypto.check_secret_key(secret)?;
        let inner = &mut *self.inner.lock();
        let dh_cache_key = DHCacheKey {
            key: key.clone(),
            secret: secret.clone(),
        };
        let res = inner.dh_cache.entry_with_callback(dh_cache_key, |_, _| {
            inner.dh_cache_lru += 1;
        });
        Ok(match res {
            Entry::Occupied(e) => {
                inner.dh_cache_hits += 1;
                e.get().shared_secret.clone()
            }
            Entry::Vacant(e) => {
                inner.dh_cache_misses += 1;

                let shared_secret = vcrypto.compute_dh(key, secret)?;
                e.insert(DHCacheValue {
                    shared_secret: shared_secret.clone(),
                });
                shared_secret
            }
        })
    }

    pub(crate) fn validate_crypto_kind(kind: CryptoKind) -> VeilidAPIResult<()> {
        if !VALID_CRYPTO_KINDS.contains(&kind) {
            apibail_generic!("invalid crypto kind");
        }
        Ok(())
    }

    pub(crate) fn debug_info_nodeinfo(&self) -> String {
        let inner = self.inner.lock();
        format!(
            "Crypto Stats:\n  DH Cache Hits/Misses/LRU: {} / {} / {}\n",
            inner.dh_cache_hits, inner.dh_cache_misses, inner.dh_cache_lru
        )
    }
}
