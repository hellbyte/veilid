use super::*;
use data_encoding::BASE64URL_NOPAD;

#[derive(Clone, Eq, PartialEq, GetSize, Hash)]
#[must_use]
pub struct EncryptedValueData {
    // capnp struct ValueData encoded without packing
    blob: Vec<u8>,
}

impl EncryptedValueData {
    pub const MAX_LEN: usize = 32768;

    pub fn new(
        seq: ValueSeqNum,
        data: Vec<u8>,
        writer: PublicKey,
        nonce: Option<Nonce>,
    ) -> VeilidAPIResult<Self> {
        if seq.is_none() {
            apibail_generic!("invalid sequence number");
        }
        if data.len() > Self::MAX_LEN {
            apibail_generic!("invalid size");
        }

        let mut message_builder = ::capnp::message::Builder::new_default();
        let mut builder = message_builder.init_root::<veilid_capnp::value_data::Builder>();

        builder.set_seq(seq.into());

        builder.set_data(&data);

        let mut wb = builder.reborrow().init_writer();
        capnp_encode_public_key(&writer, &mut wb);

        if let Some(nonce_val) = nonce {
            let mut nb = builder.reborrow().init_nonce();
            capnp_encode_nonce(&nonce_val, &mut nb);
        }

        let blob = canonical_message_builder_to_vec_unpacked(message_builder)
            .map_err(VeilidAPIError::internal)?;

        // Ensure the blob could be decoded without errors, allowing to do unwrap() in getter methods
        validate_value_data_blob(&blob).map_err(VeilidAPIError::internal)?;

        Ok(Self { blob })
    }

    #[must_use]
    pub fn seq(&self) -> ValueSeqNum {
        let message_reader = capnp::serialize::read_message_from_flat_slice(
            &mut &self.blob[..],
            capnp::message::ReaderOptions::new(),
        )
        .unwrap();
        let reader = message_reader
            .get_root::<veilid_capnp::value_data::Reader>()
            .unwrap();

        reader.get_seq().into()
    }

    pub fn writer(&self) -> PublicKey {
        let message_reader = capnp::serialize::read_message_from_flat_slice(
            &mut &self.blob[..],
            capnp::message::ReaderOptions::new(),
        )
        .unwrap();
        let reader = message_reader
            .get_root::<veilid_capnp::value_data::Reader>()
            .unwrap();

        let w = reader.get_writer().unwrap();
        PublicKey::new(
            CryptoKind::from(w.get_kind()),
            BarePublicKey::new(w.get_value().unwrap()),
        )
    }

    #[must_use]
    pub fn data(&self) -> Vec<u8> {
        let message_reader = capnp::serialize::read_message_from_flat_slice(
            &mut &self.blob[..],
            capnp::message::ReaderOptions::new(),
        )
        .unwrap();
        let reader = message_reader
            .get_root::<veilid_capnp::value_data::Reader>()
            .unwrap();

        // TODO: try to make this function return &[u8]
        reader.get_data().unwrap().to_vec()
    }

    #[must_use]
    pub fn nonce(&self) -> Option<Nonce> {
        let message_reader = capnp::serialize::read_message_from_flat_slice(
            &mut &self.blob[..],
            capnp::message::ReaderOptions::new(),
        )
        .unwrap();
        let reader = message_reader
            .get_root::<veilid_capnp::value_data::Reader>()
            .unwrap();

        if reader.has_nonce() {
            let n = reader.get_nonce().unwrap();
            Some(Nonce::new(n.get_value().unwrap()))
        } else {
            None
        }
    }

    #[must_use]
    pub fn data_size(&self) -> usize {
        let message_reader = capnp::serialize::read_message_from_flat_slice(
            &mut &self.blob[..],
            capnp::message::ReaderOptions::new(),
        )
        .unwrap();
        let reader = message_reader
            .get_root::<veilid_capnp::value_data::Reader>()
            .unwrap();

        reader.get_data().unwrap().len()
    }

    #[must_use]
    pub fn total_size(&self) -> usize {
        mem::size_of::<Self>() + self.data_size()
    }

    #[must_use]
    pub fn raw_blob(&self) -> &[u8] {
        &self.blob
    }
}

fn validate_value_data_blob(blob: &[u8]) -> capnp::Result<()> {
    let message_reader = capnp::serialize::read_message_from_flat_slice(
        &mut &blob[..],
        capnp::message::ReaderOptions::new(),
    )?;
    let reader = message_reader.get_root::<veilid_capnp::value_data::Reader>()?;
    if ValueSeqNum::from(reader.get_seq()).is_none() {
        return capnp::Result::Err(capnp::Error::failed("invalid seq".to_owned()));
    }
    let _ = reader.get_data()?;
    let _ = reader.get_writer()?;
    if reader.has_nonce() {
        let n = reader.get_nonce()?;
        let _ = n.get_value()?;
    }
    Ok(())
}

impl fmt::Debug for EncryptedValueData {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let seq = self.seq();
        let data = self.data();
        let writer = self.writer();
        let nonce = self.nonce();

        fmt.debug_struct("EncryptedValueData")
            .field("seq", &seq)
            .field("data", &print_data(&data, Some(64)))
            .field("writer", &writer)
            .field("nonce", &nonce)
            .finish()
    }
}

impl serde::Serialize for EncryptedValueData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let encoded = BASE64URL_NOPAD.encode(&self.blob);
        serializer.serialize_str(&encoded)
    }
}

impl<'de> serde::Deserialize<'de> for EncryptedValueData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct LegacyValueData {
            /// An increasing sequence number to time-order the DHT record changes
            seq: ValueSeqNum,

            /// The contents of a DHT Record
            #[cfg_attr(
                not(all(target_arch = "wasm32", target_os = "unknown")),
                serde(with = "as_human_base64")
            )]
            data: Vec<u8>,

            /// The public identity key of the writer of the data
            #[serde(with = "public_key_try_untyped_vld0")]
            writer: PublicKey,
        }

        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum Helper {
            Base64Str(String),
            Legacy(LegacyValueData),
        }

        match Helper::deserialize(deserializer)? {
            Helper::Base64Str(value) => {
                let blob = BASE64URL_NOPAD.decode(value.as_bytes()).map_err(|e| {
                    <D::Error as serde::de::Error>::custom(format!(
                        "Failed to decode base64: {}",
                        e
                    ))
                })?;

                validate_value_data_blob(&blob).map_err(|e| {
                    <D::Error as serde::de::Error>::custom(format!(
                        "Decoded blob is not a valid ValueData capnp struct: {}",
                        e
                    ))
                })?;

                Ok(EncryptedValueData { blob })
            }
            Helper::Legacy(legacy) => {
                EncryptedValueData::new(legacy.seq, legacy.data, legacy.writer, None)
                    .map_err(serde::de::Error::custom)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::tests::fixtures::*;

    #[test]
    fn value_data_ok() {
        assert!(EncryptedValueData::new(
            ValueSeqNum::ZERO,
            vec![0; EncryptedValueData::MAX_LEN],
            fix_fake_public_key(),
            None,
        )
        .is_ok());
    }

    #[test]
    fn value_data_too_long() {
        assert!(EncryptedValueData::new(
            ValueSeqNum::ZERO,
            vec![0; EncryptedValueData::MAX_LEN + 1],
            fix_fake_public_key(),
            None,
        )
        .is_err());
    }
}
