use super::*;
use veilid_api::VeilidAPIResult;

/// A DHT value and its metadata
#[derive(Clone, PartialOrd, PartialEq, Eq, Ord, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    derive(Tsify),
    tsify(from_wasm_abi, into_wasm_abi)
)]
#[cfg_attr(feature = "json-camel-case", serde(rename_all = "camelCase"))]
#[must_use]
pub struct ValueData {
    /// An increasing sequence number to time-order the DHT record changes
    seq: ValueSeqNum,

    /// The contents of a DHT Record
    #[cfg_attr(
        not(all(target_arch = "wasm32", target_os = "unknown")),
        serde(with = "as_human_base64")
    )]
    #[schemars(with = "String")]
    #[cfg_attr(
        all(target_arch = "wasm32", target_os = "unknown"),
        serde(with = "serde_bytes"),
        tsify(type = "Uint8Array")
    )]
    data: Vec<u8>,

    /// The public identity key of the writer of the data
    #[schemars(with = "String")]
    #[serde(with = "public_key_try_untyped_vld0")]
    writer: PublicKey,
}

impl ValueData {
    pub const MAX_LEN: usize = 32768;

    pub fn new(data: Vec<u8>, writer: PublicKey) -> VeilidAPIResult<Self> {
        if data.len() > Self::MAX_LEN {
            apibail_generic!("invalid size");
        }
        Ok(Self {
            seq: ValueSeqNum::ZERO,
            data,
            writer,
        })
    }
    pub fn new_with_seq(
        seq: ValueSeqNum,
        data: Vec<u8>,
        writer: PublicKey,
    ) -> VeilidAPIResult<Self> {
        if seq.is_none() {
            apibail_generic!("invalid sequence number");
        }
        if data.len() > Self::MAX_LEN {
            apibail_generic!("invalid size");
        }
        Ok(Self { seq, data, writer })
    }

    pub fn ref_writer(&self) -> &PublicKey {
        &self.writer
    }

    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    #[must_use]
    pub fn seq(&self) -> ValueSeqNum {
        self.seq
    }

    pub fn writer(&self) -> PublicKey {
        self.writer.clone()
    }

    #[must_use]
    pub fn data_size(&self) -> usize {
        self.data.len()
    }

    #[must_use]
    pub fn total_size(&self) -> usize {
        mem::size_of::<Self>() + self.data.len()
    }
}

impl fmt::Debug for ValueData {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("ValueData")
            .field("seq", &u32::from(self.seq))
            .field("data", &print_data(&self.data, Some(64)))
            .field("writer", &self.writer)
            .finish()
    }
}

impl fmt::Display for ValueData {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "seq={},len(data)={},writer={}",
            self.seq,
            self.data.len(),
            self.writer
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::tests::fixtures::*;

    #[test]
    fn value_data_ok() {
        assert!(ValueData::new(vec![0; ValueData::MAX_LEN], fix_fake_public_key()).is_ok());
        assert!(ValueData::new_with_seq(
            ValueSeqNum::ZERO,
            vec![0; ValueData::MAX_LEN],
            fix_fake_public_key()
        )
        .is_ok());
    }

    #[test]
    fn value_data_too_long() {
        assert!(ValueData::new(vec![0; ValueData::MAX_LEN + 1], fix_fake_public_key()).is_err());
        assert!(ValueData::new_with_seq(
            ValueSeqNum::ZERO,
            vec![0; ValueData::MAX_LEN + 1],
            fix_fake_public_key()
        )
        .is_err());
    }
}
