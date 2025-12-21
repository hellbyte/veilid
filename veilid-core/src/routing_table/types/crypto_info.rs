use super::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CryptoInfo {
    #[cfg(feature = "enable-crypto-none")]
    NONE { public_key: BarePublicKey },
    #[cfg(feature = "enable-crypto-vld0")]
    VLD0 { public_key: BarePublicKey },
    // #[cfg(feature = "enable-crypto-vld1")]
    // VLD1 {
    //     encapsulation_key: BareEncapsulationKey,
    //     signing_key: BarePublicKey,
    // },
}

impl CryptoInfo {
    pub fn kind(&self) -> CryptoKind {
        match self {
            #[cfg(feature = "enable-crypto-none")]
            CryptoInfo::NONE { public_key: _ } => CRYPTO_KIND_NONE,
            #[cfg(feature = "enable-crypto-vld0")]
            CryptoInfo::VLD0 { public_key: _ } => CRYPTO_KIND_VLD0,
            // #[cfg(feature = "enable-crypto-vld1")]
            // CryptoInfo::VLD1 {
            //     encapsulation_key: _,
            //     signing_key: _,
            // } => CRYPTO_KIND_VLD1,
        }
    }
}
