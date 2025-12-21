use super::*;

const MAX_APP_CALL_Q_MESSAGE_LEN: usize = 32768;
const MAX_APP_CALL_A_MESSAGE_LEN: usize = 32768;

#[derive(Debug, Clone)]
pub(in crate::rpc_processor) struct RPCOperationAppCallQ {
    message: Vec<u8>,
}

impl RPCOperationAppCallQ {
    pub fn new(message: Vec<u8>) -> Result<Self, RPCError> {
        if message.len() > MAX_APP_CALL_Q_MESSAGE_LEN {
            return Err(RPCError::internal("AppCallQ message too long to set"));
        }
        Ok(Self { message })
    }
    pub fn validate(&self, _validate_context: &RPCValidateContext) -> Result<(), RPCError> {
        Ok(())
    }

    pub fn destructure(self) -> Vec<u8> {
        self.message
    }

    pub fn decode(
        _decode_context: &RPCDecodeContext,
        reader: &veilid_capnp::operation_app_call_q::Reader,
    ) -> Result<Self, RPCError> {
        rpc_ignore_missing_property!(reader, message);
        let mr = reader.get_message()?;
        rpc_ignore_max_len!(mr, MAX_APP_CALL_Q_MESSAGE_LEN);

        RPCOperationAppCallQ::new(mr.to_vec())
    }
    pub fn encode(
        &self,
        builder: &mut veilid_capnp::operation_app_call_q::Builder,
    ) -> Result<(), RPCError> {
        builder.set_message(&self.message);
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub(in crate::rpc_processor) struct RPCOperationAppCallA {
    message: Vec<u8>,
}

impl RPCOperationAppCallA {
    pub fn new(message: Vec<u8>) -> Result<Self, RPCError> {
        if message.len() > MAX_APP_CALL_A_MESSAGE_LEN {
            return Err(RPCError::ignore("AppCallA message too long to set"));
        }
        Ok(Self { message })
    }

    pub fn validate(&self, _validate_context: &RPCValidateContext) -> Result<(), RPCError> {
        Ok(())
    }

    pub fn destructure(self) -> Vec<u8> {
        self.message
    }

    pub fn decode(
        _decode_context: &RPCDecodeContext,
        reader: &veilid_capnp::operation_app_call_a::Reader,
    ) -> Result<Self, RPCError> {
        rpc_ignore_missing_property!(reader, message);
        let mr = reader.get_message()?;
        rpc_ignore_max_len!(mr, MAX_APP_CALL_A_MESSAGE_LEN);
        Self::new(mr.to_vec())
    }
    pub fn encode(
        &self,
        builder: &mut veilid_capnp::operation_app_call_a::Builder,
    ) -> Result<(), RPCError> {
        builder.set_message(&self.message);
        Ok(())
    }
}
