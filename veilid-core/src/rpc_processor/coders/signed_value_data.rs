use super::*;
use crate::storage_manager::*;

fn decode_value_data(
    reader: &veilid_capnp::value_data::Reader,
) -> Result<EncryptedValueData, RPCError> {
    let seq = ValueSeqNum::from(reader.get_seq());

    rpc_ignore_missing_property!(reader, data);
    let data = reader.get_data()?.to_vec();

    rpc_ignore_missing_property!(reader, writer);
    let wr = reader.get_writer()?;
    let writer = decode_public_key(&wr)?;

    let n = reader.get_nonce()?;
    let nonce = if n.has_value() {
        let nonce = decode_nonce(&n)?;
        if nonce.len() != 24 {
            return Err(RPCError::protocol("value data nonce has invalid size"));
        }
        Some(nonce)
    } else {
        None
    };

    EncryptedValueData::new(seq, data, writer, nonce).map_err(RPCError::protocol)
}

pub fn decode_signed_value_data(
    reader: &veilid_capnp::signed_value_data::Reader,
) -> Result<SignedValueData, RPCError> {
    rpc_ignore_missing_property!(reader, value_data);
    let value_data_buf = reader.get_value_data()?;
    let mut value_data_cursor = &mut &value_data_buf[..];
    let tmp_reader = capnp::serialize::read_message(
        &mut value_data_cursor,
        capnp::message::ReaderOptions::new(),
    )?;
    let value_data_reader = tmp_reader.get_root::<veilid_capnp::value_data::Reader>()?;

    let encrypted_value_data = decode_value_data(&value_data_reader)?;

    rpc_ignore_missing_property!(reader, signature);
    let sr = reader.get_signature()?;
    let signature = decode_signature(&sr)?;

    Ok(SignedValueData::new(encrypted_value_data, signature))
}

pub fn encode_signed_value_data(
    signed_value_data: &SignedValueData,
    builder: &mut veilid_capnp::signed_value_data::Builder,
) -> Result<(), RPCError> {
    let encoded_value_data = signed_value_data.value_data().raw_blob();
    builder.set_value_data(encoded_value_data);

    let mut sb = builder.reborrow().init_signature();
    encode_signature(signed_value_data.signature(), &mut sb);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{decode_signed_value_data, encode_signed_value_data};
    use crate::crypto::tests::fixtures::*;
    use crate::rpc_processor::canonical_message_builder_to_vec_packed;
    use crate::storage_manager::{EncryptedValueData, SignedValueData};
    use crate::{veilid_capnp, BareSignature, Nonce, Signature};

    #[test]
    fn test_encode_and_decode_signed_value_data() {
        let keypair = fix_keypair();
        let fake_nonce = [0x22; 24];
        let fake_signature = [0x55; 64];

        let mut message_builder = ::capnp::message::Builder::new_default();
        let mut builder = message_builder.init_root::<veilid_capnp::signed_value_data::Builder>();

        let signed_value_data = SignedValueData::new(
            EncryptedValueData::new(
                10.into(),
                vec![1, 2, 3, 4, 5, 6],
                keypair.key(),
                Some(Nonce::new(&fake_nonce)),
            )
            .unwrap(),
            Signature::new(keypair.kind(), BareSignature::new(&fake_signature)),
        );
        encode_signed_value_data(&signed_value_data, &mut builder).unwrap();

        let buffer = canonical_message_builder_to_vec_packed(message_builder).unwrap();

        println!("buffer[{}] = {:02x?}", buffer.len(), &buffer);

        let mut value_data_cursor = &mut &buffer[..];
        let tmp_reader = capnp::serialize_packed::read_message(
            &mut value_data_cursor,
            capnp::message::ReaderOptions::new(),
        )
        .unwrap();
        let reader = tmp_reader
            .get_root::<veilid_capnp::signed_value_data::Reader>()
            .unwrap();

        let decoded = decode_signed_value_data(&reader).unwrap();

        assert_eq!(
            signed_value_data.value_data().seq(),
            decoded.value_data().seq()
        );
        assert_eq!(
            signed_value_data.value_data().data(),
            decoded.value_data().data()
        );
        assert_eq!(
            signed_value_data.value_data().writer(),
            decoded.value_data().writer()
        );
        assert_eq!(signed_value_data.signature(), decoded.signature());
    }
}
