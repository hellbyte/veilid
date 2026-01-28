use super::*;

pub const HASH_COORDINATE_LENGTH: usize = 32;

// Internal types

impl HashCoordinate {
    pub(crate) fn distance(&self, other: &HashCoordinate) -> HashDistance {
        assert_eq!(self.kind(), other.kind());
        self.ref_value().distance(other.ref_value())
    }
}

impl NodeId {
    pub(crate) fn to_hash_coordinate(&self) -> HashCoordinate {
        HashCoordinate::new(self.kind(), self.ref_value().to_bare_hash_coordinate())
    }
}
impl BareNodeId {
    pub(crate) fn to_bare_hash_coordinate(&self) -> BareHashCoordinate {
        BareHashCoordinate::new(self)
    }
}

impl OpaqueRecordKey {
    pub(crate) fn to_hash_coordinate(&self) -> HashCoordinate {
        HashCoordinate::new(self.kind(), self.ref_value().to_bare_hash_coordinate())
    }
}
impl BareOpaqueRecordKey {
    pub(crate) fn to_bare_hash_coordinate(&self) -> BareHashCoordinate {
        BareHashCoordinate::new(self)
    }
}

impl RecordKey {
    pub(crate) fn to_hash_coordinate(&self) -> HashCoordinate {
        HashCoordinate::new(self.kind(), self.ref_value().to_bare_hash_coordinate())
    }
}
impl BareRecordKey {
    pub(crate) fn to_bare_hash_coordinate(&self) -> BareHashCoordinate {
        BareHashCoordinate::new(self.ref_key())
    }
}

impl HashDigest {
    #[allow(dead_code)]
    pub(crate) fn to_hash_coordinate(&self) -> HashCoordinate {
        HashCoordinate::new(self.kind(), self.ref_value().to_bare_hash_coordinate())
    }
}
impl BareHashDigest {
    #[allow(dead_code)]
    pub(crate) fn to_bare_hash_coordinate(&self) -> BareHashCoordinate {
        BareHashCoordinate::new(self)
    }
}

impl BareHashCoordinate {
    pub(crate) fn distance(&self, other: &BareHashCoordinate) -> HashDistance {
        assert_eq!(self.len(), HASH_COORDINATE_LENGTH);
        assert_eq!(other.len(), HASH_COORDINATE_LENGTH);

        let mut bytes = [0u8; HASH_COORDINATE_LENGTH];

        (0..HASH_COORDINATE_LENGTH).for_each(|n| {
            bytes[n] = self[n] ^ other[n];
        });

        HashDistance::new(&bytes)
    }
}
