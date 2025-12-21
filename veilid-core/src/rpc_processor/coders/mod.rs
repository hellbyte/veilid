mod address;
mod address_type_set;
mod byte_array_types;
mod crypto_info;
mod dial_info;
mod dial_info_class;
mod dial_info_detail;
mod node_info;
mod node_status;
mod operations;
mod peer_info;
mod private_safety_route;
mod protocol_type_set;
mod relay_info;
mod relay_kind;
mod sender_info;
mod sequencing;
mod signal_info;
mod signed_value_data;
mod signed_value_descriptor;
mod socket_address;
#[cfg(feature = "unstable-tunnels")]
mod tunnel;

pub use address::*;
pub use address_type_set::*;
pub use byte_array_types::*;
pub use crypto_info::*;
pub use dial_info::*;
pub use dial_info_class::*;
pub use dial_info_detail::*;
pub use node_info::*;
pub use node_status::*;
pub use operations::*;
pub use peer_info::*;
pub use private_safety_route::*;
pub use protocol_type_set::*;
pub use relay_info::*;
pub use relay_kind::*;
pub use sender_info::*;
pub use sequencing::*;
pub use signal_info::*;
pub use signed_value_data::*;
pub use signed_value_descriptor::*;
pub use socket_address::*;
#[cfg(feature = "unstable-tunnels")]
pub use tunnel::*;

use super::*;
use capnp::message::ReaderSegments;

impl_veilid_log_facility!("rpc");

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum QuestionContext {
    GetValue(ValidateGetValueContext),
    SetValue(ValidateSetValueContext),
    InspectValue(ValidateInspectValueContext),
    TransactBegin(ValidateTransactBeginContext),
    TransactCommand(ValidateTransactCommandContext),
}

pub struct RPCValidateContext<'a> {
    pub registry: VeilidComponentRegistry,
    pub question_context: Option<&'a QuestionContext>,
}
impl_veilid_component_accessors!(RPCValidateContext<'_>);

#[derive(Clone)]
pub struct RPCDecodeContext {
    pub registry: VeilidComponentRegistry,
    pub origin_routing_domain: RoutingDomain,
}

#[instrument(level = "trace", target = "rpc", skip_all, err)]
pub fn canonical_message_builder_to_vec_packed<'a, T>(
    builder: capnp::message::Builder<T>,
) -> Result<Vec<u8>, RPCError>
where
    T: capnp::message::Allocator + 'a,
{
    // Canonicalize builder
    let buffer = if builder.len() != 1 {
        let root = builder
            .get_root_as_reader::<capnp::any_pointer::Reader>()
            .map_err(RPCError::protocol)?;

        let size = root.target_size()?.word_count + 1;
        let mut canonical_builder = capnp::message::Builder::new(
            capnp::message::HeapAllocator::new().first_segment_words(size as u32),
        );
        canonical_builder.set_root_canonical(root)?;

        let mut buffer = Vec::<u8>::with_capacity(canonical_builder.size_in_words());
        capnp::serialize_packed::write_message(&mut buffer, &canonical_builder)
            .map_err(RPCError::protocol)?;
        buffer
    } else {
        let mut buffer = Vec::<u8>::with_capacity(builder.size_in_words());
        capnp::serialize_packed::write_message(&mut buffer, &builder)
            .map_err(RPCError::protocol)?;
        buffer
    };

    Ok(buffer)
}

#[instrument(level = "trace", target = "rpc", skip_all, err)]
pub fn canonical_message_builder_to_write_packed<'a, T, W>(
    write: W,
    builder: capnp::message::Builder<T>,
) -> Result<(), RPCError>
where
    T: capnp::message::Allocator + 'a,
    W: capnp::io::Write,
{
    // Canonicalize builder
    if builder.len() != 1 {
        let root = builder
            .get_root_as_reader::<capnp::any_pointer::Reader>()
            .map_err(RPCError::protocol)?;

        let size = root.target_size()?.word_count + 1;
        let mut canonical_builder = capnp::message::Builder::new(
            capnp::message::HeapAllocator::new().first_segment_words(size as u32),
        );
        canonical_builder.set_root_canonical(root)?;

        capnp::serialize_packed::write_message(write, &canonical_builder)
            .map_err(RPCError::protocol)?;
    } else {
        capnp::serialize_packed::write_message(write, &builder).map_err(RPCError::protocol)?;
    };
    Ok(())
}

#[instrument(level = "trace", target = "rpc", skip_all, err)]
pub fn canonical_message_builder_to_vec_unpacked<'a, T>(
    builder: capnp::message::Builder<T>,
) -> Result<Vec<u8>, RPCError>
where
    T: capnp::message::Allocator + 'a,
{
    // Canonicalize builder
    if builder.len() != 1 {
        let root = builder
            .get_root_as_reader::<capnp::any_pointer::Reader>()
            .map_err(RPCError::protocol)?;

        let size = root.target_size()?.word_count + 1;
        let mut canonical_builder = capnp::message::Builder::new(
            capnp::message::HeapAllocator::new().first_segment_words(size as u32),
        );
        canonical_builder.set_root_canonical(root)?;

        Ok(capnp::serialize::write_message_to_words(&canonical_builder))
    } else {
        Ok(capnp::serialize::write_message_to_words(&builder))
    }
}
