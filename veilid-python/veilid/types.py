import base64
import json
from abc import ABC, abstractmethod
from enum import StrEnum
from functools import total_ordering
from typing import Any, Optional, Self, final

####################################################################


def urlsafe_b64encode_no_pad(b: bytes) -> str:
    """
    Removes any `=` used as padding from the encoded string.
    """
    return base64.urlsafe_b64encode(b).decode().rstrip("=")


def urlsafe_b64decode_no_pad(s: str) -> bytes:
    """
    Adds back in the required padding before decoding.
    """
    padding = 4 - (len(s) % 4)
    s = s + ("=" * padding)
    return base64.urlsafe_b64decode(s)


class VeilidJSONEncoder(json.JSONEncoder):
    def default(self, o):
        if isinstance(o, bytes):
            return urlsafe_b64encode_no_pad(o)
        if hasattr(o, "to_json") and callable(o.to_json):
            return o.to_json()
        return json.JSONEncoder.default(self, o)

    @staticmethod
    def dumps(req: Any, *args, **kwargs) -> str:
        return json.dumps(req, cls=VeilidJSONEncoder, *args, **kwargs)


####################################################################


class VeilidLogLevel(StrEnum):
    ERROR = "Error"
    WARN = "Warn"
    INFO = "Info"
    DEBUG = "Debug"
    TRACE = "Trace"


class CryptoKind(StrEnum):
    CRYPTO_KIND_NONE = "NONE"
    CRYPTO_KIND_VLD0 = "VLD0"
    CRYPTO_KIND_VLD1 = "VLD1"


class VeilidCapability(StrEnum):
    VEILID_CAPABILITY_ROUTE = "ROUT"
    VEILID_CAPABILITY_TUNNEL = "TUNL"
    VEILID_CAPABILITY_SIGNAL = "SGNL"
    VEILID_CAPABILITY_RELAY = "RLAY"
    VEILID_CAPABILITY_VALIDATE_DIAL_INFO = "DIAL"
    VEILID_CAPABILITY_DHT = "DHTV"
    VEILID_CAPABILITY_APPMESSAGE = "APPM"
    VEILID_CAPABILITY_BLOCKSTORE = "BLOC"


class Stability(StrEnum):
    LOW_LATENCY = "LowLatency"
    RELIABLE = "Reliable"


class Sequencing(StrEnum):
    NO_PREFERENCE = "NoPreference"
    PREFER_ORDERED = "PreferOrdered"
    ENSURE_ORDERED = "EnsureOrdered"


class DHTSchemaKind(StrEnum):
    DFLT = "DFLT"
    SMPL = "SMPL"


class SafetySelectionKind(StrEnum):
    UNSAFE = "Unsafe"
    SAFE = "Safe"

class TargetKind(StrEnum):
    ROUTE_ID= "RouteId"
    NODE_ID = "NodeId"

class DHTReportScope(StrEnum):
    LOCAL = "Local"
    SYNC_GET = "SyncGet"
    SYNC_SET = "SyncSet"
    UPDATE_GET = "UpdateGet"
    UPDATE_SET = "UpdateSet"


####################################################################


class Timestamp(int):
    pass


class TimestampDuration(int):
    pass


class ByteCount(int):
    pass


class OperationId(str):
    pass

class EncodedString(str):
    def to_bytes(self) -> bytes:
        return urlsafe_b64decode_no_pad(self)

    @classmethod
    def from_bytes(cls, b: bytes) -> Self:
        assert isinstance(b, bytes)
        return cls(urlsafe_b64encode_no_pad(b))

class BarePublicKey(EncodedString):
    pass

class BareSecretKey(EncodedString):
    pass

class BareSharedSecret(EncodedString):
    pass

class BareHashDigest(EncodedString):
    pass

class BareSignature(EncodedString):
    pass

class Nonce(EncodedString):
    pass

class BareRouteId(EncodedString):
    pass

class BareNodeId(EncodedString):
    pass

class BareMemberId(EncodedString):
    pass

class BareOpaqueRecordKey(EncodedString):
    pass

class BareRecordKey(str):
    @classmethod
    def from_parts(cls, key: BareOpaqueRecordKey, encryption_key: Optional[BareSharedSecret]) -> Self:
        assert isinstance(key, BareOpaqueRecordKey)
        if encryption_key is not None:
            assert isinstance(encryption_key, BareSharedSecret)
            return cls(f"{key}:{encryption_key}")
        return cls(f"{key}")

    def key(self) -> BareOpaqueRecordKey:
        parts = self.split(":", 1)
        return BareOpaqueRecordKey(parts[0])

    def encryption_key(self) -> Optional[BareSharedSecret]:
        parts = self.split(":", 1)
        if len(parts) == 2:
            return BareSharedSecret(self.split(":", 1)[1])
        return None

class BareKeyPair(str):
    @classmethod
    def from_parts(cls, key: BarePublicKey, secret: BareSecretKey) -> Self:
        assert isinstance(key, BarePublicKey)
        assert isinstance(secret, BareSecretKey)
        return cls(f"{key}:{secret}")

    def key(self) -> BarePublicKey:
        return BarePublicKey(self.split(":", 1)[0])

    def secret(self) -> BareSecretKey:
        return BareSecretKey(self.split(":", 1)[1])

class CryptoTyped(str):
    def kind(self) -> CryptoKind:
        if self[4] != ":":
            raise ValueError("Not CryptoTyped")
        return CryptoKind(self[0:4])

    def _value(self) -> str:
        if self[4] != ":":
            raise ValueError("Not CryptoTyped")
        return self[5:]

class SharedSecret(CryptoTyped):
    @classmethod
    def from_value(cls, kind: CryptoKind, value: BareSharedSecret) -> Self:
        assert isinstance(kind, CryptoKind)
        assert isinstance(value, BareSharedSecret)
        return cls(f"{kind}:{value}")

    def value(self) -> BareSharedSecret:
        return BareSharedSecret(self._value())

class RecordKey(CryptoTyped):
    @classmethod
    def from_value(cls, kind: CryptoKind, value: BareRecordKey) -> Self:
        assert isinstance(kind, CryptoKind)
        assert isinstance(value, BareRecordKey)
        return cls(f"{kind}:{value}")

    def value(self) -> BareRecordKey:
        return BareRecordKey(self._value())

    def encryption_key(self) -> Optional[SharedSecret]:
        ek = self.value().encryption_key()
        return None if ek == None else SharedSecret.from_value(self.kind(), ek)

class HashDigest(CryptoTyped):
    @classmethod
    def from_value(cls, kind: CryptoKind, value: BareHashDigest) -> Self:
        assert isinstance(kind, CryptoKind)
        assert isinstance(value, BareHashDigest)
        return cls(f"{kind}:{value}")

    def value(self) -> BareHashDigest:
        return BareHashDigest(self._value())

class PublicKey(CryptoTyped):
    @classmethod
    def from_value(cls, kind: CryptoKind, value: BarePublicKey) -> Self:
        assert isinstance(kind, CryptoKind)
        assert isinstance(value, BarePublicKey)
        return cls(f"{kind}:{value}")

    def value(self) -> BarePublicKey:
        return BarePublicKey(self._value())


class SecretKey(CryptoTyped):
    @classmethod
    def from_value(cls, kind: CryptoKind, value: BareSecretKey) -> Self:
        assert isinstance(kind, CryptoKind)
        assert isinstance(value, BareSecretKey)
        return cls(f"{kind}:{value}")

    def value(self) -> BareSecretKey:
        return BareSecretKey(self._value())


class KeyPair(CryptoTyped):
    @classmethod
    def from_value(cls, kind: CryptoKind, value: BareKeyPair) -> Self:
        assert isinstance(kind, CryptoKind)
        assert isinstance(value, BareKeyPair)
        return cls(f"{kind}:{value}")

    def value(self) -> BareKeyPair:
        return BareKeyPair(self._value())

    def key(self) -> PublicKey:
        return PublicKey.from_value(kind=self.kind(), value=self.value().key())

    def secret(self) -> SecretKey:
        return SecretKey.from_value(kind=self.kind(), value=self.value().secret())

class Signature(CryptoTyped):
    @classmethod
    def from_value(cls, kind: CryptoKind, value: BareSignature) -> Self:
        assert isinstance(kind, CryptoKind)
        assert isinstance(value, BareSignature)
        return cls(f"{kind}:{value}")

    def value(self) -> BareSignature:
        return BareSignature(self._value())

class RouteId(CryptoTyped):
    @classmethod
    def from_value(cls, kind: CryptoKind, value: BareRouteId) -> Self:
        assert isinstance(kind, CryptoKind)
        assert isinstance(value, BareRouteId)
        return cls(f"{kind}:{value}")

    def value(self) -> BareRouteId:
        return BareRouteId(self._value())

class NodeId(CryptoTyped):
    @classmethod
    def from_value(cls, kind: CryptoKind, value: BareNodeId) -> Self:
        assert isinstance(kind, CryptoKind)
        assert isinstance(value, BareNodeId)
        return cls(f"{kind}:{value}")

    def value(self) -> BareNodeId:
        return BareNodeId(self._value())

class MemberId(CryptoTyped):
    @classmethod
    def from_value(cls, kind: CryptoKind, value: BareMemberId) -> Self:
        assert isinstance(kind, CryptoKind)
        assert isinstance(value, BareMemberId)
        return cls(f"{kind}:{value}")

    def value(self) -> BareMemberId:
        return BareMemberId(self._value())

class ValueSubkey(int):
    pass


class ValueSeqNum(int):
    pass

####################################################################


@total_ordering
class VeilidVersion:
    _major: int
    _minor: int
    _patch: int

    def __init__(self, major: int, minor: int, patch: int):
        self._major = major
        self._minor = minor
        self._patch = patch

    def __lt__(self, other):
        if other is None:
            return False
        if self._major < other._major:
            return True
        if self._major > other._major:
            return False
        if self._minor < other._minor:
            return True
        if self._minor > other._minor:
            return False
        if self._patch < other._patch:
            return True
        return False

    def __eq__(self, other):
        return (
            isinstance(other, VeilidVersion)
            and self._major == other._major
            and self._minor == other._minor
            and self._patch == other._patch
        )

    @property
    def major(self):
        return self._major

    @property
    def minor(self):
        return self._minor

    @property
    def patch(self):
        return self._patch


class RouteBlob:
    route_id: RouteId
    blob: bytes

    def __init__(self, route_id: RouteId, blob: bytes):
        assert isinstance(route_id, RouteId)
        assert isinstance(blob, bytes)

        self.route_id = route_id
        self.blob = blob

    @classmethod
    def from_json(cls, j: dict) -> Self:
        return cls(RouteId(j["route_id"]), urlsafe_b64decode_no_pad(j["blob"]))

    def to_json(self) -> dict:
        return self.__dict__


class DHTSchemaSMPLMember:
    m_key: BareMemberId
    m_cnt: int

    def __init__(self, m_key: BareMemberId, m_cnt: int):
        assert isinstance(m_key, BareMemberId)
        assert isinstance(m_cnt, int)

        self.m_key = m_key
        self.m_cnt = m_cnt

    @classmethod
    def from_json(cls, j: dict) -> Self:
        return cls(BareMemberId(j["m_key"]), j["m_cnt"])

    def to_json(self) -> dict:
        return self.__dict__


class DHTSchema(ABC):
    kind: DHTSchemaKind

    def __init__(self, kind: DHTSchemaKind):
        self.kind = kind

    @classmethod
    def dflt(cls, o_cnt: int) -> Self:
        return DHTSchemaDFLT(o_cnt=o_cnt) # type: ignore

    @classmethod
    def smpl(cls, o_cnt: int, members: list[DHTSchemaSMPLMember]) -> Self:
        return DHTSchemaSMPL(o_cnt=o_cnt, members=members) # type: ignore

    @classmethod
    def from_json(cls, j: dict) -> Self:
        if DHTSchemaKind(j["kind"]) == DHTSchemaKind.DFLT:
            return cls.dflt(j["o_cnt"])
        if DHTSchemaKind(j["kind"]) == DHTSchemaKind.SMPL:
            return cls.smpl(
                j["o_cnt"],
                [DHTSchemaSMPLMember.from_json(member) for member in j["members"]],
            )
        raise Exception("Unknown DHTSchema kind", j["kind"])

    def to_json(self) -> dict:
        return self.__dict__

@final
class DHTSchemaDFLT(DHTSchema):
    o_cnt: int

    def __init__(
        self,
        o_cnt: int
    ):
        super().__init__(DHTSchemaKind.DFLT)

        assert isinstance(o_cnt, int)
        self.o_cnt = o_cnt


    @classmethod
    def from_json(cls, j: dict) -> Self:
        if DHTSchemaKind(j["kind"]) == DHTSchemaKind.DFLT:
            return cls(j["o_cnt"])
        raise Exception("Invalid DHTSchemaDFLT")


@final
class DHTSchemaSMPL(DHTSchema):
    o_cnt: int
    members: list[DHTSchemaSMPLMember]

    def __init__(
        self,
        o_cnt: int,
        members: list[DHTSchemaSMPLMember]
    ):
        super().__init__(DHTSchemaKind.SMPL)

        assert isinstance(o_cnt, int)
        assert isinstance(members, list)
        for m in members:
            assert isinstance(m, DHTSchemaSMPLMember)

        self.o_cnt = o_cnt
        self.members = members

    @classmethod
    def from_json(cls, j: dict) -> Self:
        if DHTSchemaKind(j["kind"]) == DHTSchemaKind.SMPL:
            return cls(j["o_cnt"],
                [DHTSchemaSMPLMember.from_json(member) for member in j["members"]])
        raise Exception("Invalid DHTSchemaSMPL")

class DHTRecordDescriptor:
    key: RecordKey
    owner: PublicKey
    owner_secret: Optional[SecretKey]
    schema: DHTSchema

    def __init__(
        self,
        key: RecordKey,
        owner: PublicKey,
        owner_secret: Optional[SecretKey],
        schema: DHTSchema,
    ):
        self.key = key
        self.owner = owner
        self.owner_secret = owner_secret
        self.schema = schema

    def __repr__(self) -> str:
        return f"<{self.__class__.__name__}(key={self.key!r}, owner={self.owner!r}, owner_secret={self.owner_secret!r}, schema={self.schema!r})>"

    def owner_bare_key_pair(self) -> Optional[BareKeyPair]:
        if self.owner_secret is None:
            return None
        return BareKeyPair.from_parts(self.owner.value(), self.owner_secret.value())

    def owner_key_pair(self) -> Optional[KeyPair]:
        if self.owner_secret is None:
            return None
        return KeyPair.from_value(self.owner.kind(), BareKeyPair.from_parts(self.owner.value(), self.owner_secret.value()))

    @classmethod
    def from_json(cls, j: dict) -> Self:
        return cls(
            RecordKey(j["key"]),
            PublicKey(j["owner"]),
            None if j["owner_secret"] is None else SecretKey(j["owner_secret"]),
            DHTSchema.from_json(j["schema"]),
        )

    def to_json(self) -> dict:
        return self.__dict__



class DHTRecordReport:
    subkeys: list[tuple[ValueSubkey, ValueSubkey]]
    offline_subkeys: list[tuple[ValueSubkey, ValueSubkey]]
    local_seqs: list[Optional[ValueSeqNum]]
    network_seqs: list[Optional[ValueSeqNum]]

    def __init__(
        self,
        subkeys: list[tuple[ValueSubkey, ValueSubkey]],
        offline_subkeys: list[tuple[ValueSubkey, ValueSubkey]],
        local_seqs: list[Optional[ValueSeqNum]],
        network_seqs: list[Optional[ValueSeqNum]],
    ):
        self.subkeys = subkeys
        self.offline_subkeys = offline_subkeys
        self.local_seqs = local_seqs
        self.network_seqs = network_seqs

    def __repr__(self) -> str:
        return f"<{self.__class__.__name__}(subkeys={self.subkeys!r}, offline_subkeys={self.offline_subkeys!r}, local_seqs={self.local_seqs!r}, network_seqs={self.network_seqs!r})>"

    @classmethod
    def from_json(cls, j: dict) -> Self:
        return cls(
            [(p[0], p[1]) for p in j["subkeys"]],
            [(p[0], p[1]) for p in j["offline_subkeys"]],
            [(ValueSeqNum(s) if s is not None else None) for s in j["local_seqs"] ],
            [(ValueSeqNum(s) if s is not None else None) for s in j["network_seqs"] ],
        )

    def to_json(self) -> dict:
        return self.__dict__


class SetDHTValueOptions:
    writer: Optional[KeyPair]
    allow_offline: Optional[bool]

    def __init__(self, writer: Optional[KeyPair] = None, allow_offline: Optional[bool] = None):
        self.writer = writer
        self.allow_offline = allow_offline

    def __repr__(self) -> str:
        return f"<{self.__class__.__name__}(writer={self.writer!r}, allow_offline={self.allow_offline!r})>"

    @classmethod
    def from_json(cls, j: dict) -> Self:
        return cls(
            KeyPair(j["writer"]) if "writer" in j else None,
            j["allow_offline"] if "allow_offline" in j else None,
        )

    def to_json(self) -> dict:
        return self.__dict__


class DHTTransactionSetValueOptions:
    writer: Optional[KeyPair]

    def __init__(self, writer: Optional[KeyPair] = None):
        self.writer = writer

    def __repr__(self) -> str:
        return f"<{self.__class__.__name__}(writer={self.writer!r})>"

    @classmethod
    def from_json(cls, j: dict) -> Self:
        return cls(
            KeyPair(j["writer"]) if "writer" in j else None,
        )

    def to_json(self) -> dict:
        return self.__dict__


class TransactDHTRecordsOptions:
    default_signing_keypair: Optional[KeyPair]

    def __init__(self, default_signing_keypair: Optional[KeyPair] = None):
        self.default_signing_keypair = default_signing_keypair

    def __repr__(self) -> str:
        return f"<{self.__class__.__name__}(default_signing_keypair={self.default_signing_keypair!r})>"

    @classmethod
    def from_json(cls, j: dict) -> Self:
        return cls(
            KeyPair(j["default_signing_keypair"]) if "default_signing_keypair" in j else None,
        )

    def to_json(self) -> dict:
        return self.__dict__


@total_ordering
class ValueData:
    seq: ValueSeqNum
    data: bytes
    writer: PublicKey

    def __init__(self, seq: ValueSeqNum, data: bytes, writer: PublicKey):
        self.seq = seq
        self.data = data
        self.writer = writer

    def __repr__(self) -> str:
        return f"<{self.__class__.__name__}(seq={self.seq!r}, data={self.data!r}, writer={self.writer!r})>"

    def __lt__(self, other):
        if other is None:
            return True
        if self.data < other.data:
            return True
        if self.data > other.data:
            return False
        if self.seq < other.seq:
            return True
        if self.seq > other.seq:
            return False
        if self.writer < other.writer:
            return True
        return False

    def __eq__(self, other):
        return (
            isinstance(other, ValueData)
            and self.data == other.data
            and self.seq == other.seq
            and self.writer == other.writer
        )

    @classmethod
    def from_json(cls, j: dict) -> Self:
        return cls(
            ValueSeqNum(j["seq"]),
            urlsafe_b64decode_no_pad(j["data"]),
            PublicKey(j["writer"]),
        )

    def to_json(self) -> dict:
        return self.__dict__


####################################################################


class SafetySpec:
    preferred_route: Optional[RouteId]
    hop_count: int
    stability: Stability
    sequencing: Sequencing

    def __init__(
        self,
        preferred_route: Optional[RouteId],
        hop_count: int,
        stability: Stability,
        sequencing: Sequencing,
    ):
        self.preferred_route = preferred_route
        self.hop_count = hop_count
        self.stability = stability
        self.sequencing = sequencing

    @classmethod
    def from_json(cls, j: dict) -> Self:
        return cls(
            RouteId(j["preferred_route"]) if "preferred_route" in j else None,
            j["hop_count"],
            Stability(j["stability"]),
            Sequencing(j["sequencing"]),
        )

    def to_json(self) -> dict:
        return self.__dict__

class SafetySelection(ABC):

    @property
    @abstractmethod
    def kind(self) -> SafetySelectionKind:
        pass

    @classmethod
    def unsafe(cls, sequencing: Sequencing = Sequencing.PREFER_ORDERED) -> Self:
        return SafetySelectionUnsafe(sequencing=sequencing) # type: ignore

    @classmethod
    def safe(cls, safety_spec: SafetySpec) -> Self:
        return SafetySelectionSafe(safety_spec=safety_spec) # type: ignore

    @classmethod
    def from_json(cls, j: dict) -> Self:
        if "Safe" in j:
            return cls.safe(SafetySpec.from_json(j["Safe"]))
        elif "Unsafe" in j:
            return cls.unsafe(Sequencing(j["Unsafe"]))
        raise Exception("Invalid SafetySelection")

    @abstractmethod
    def to_json(self) -> dict:
        pass

@final
class SafetySelectionUnsafe(SafetySelection):
    sequencing: Sequencing

    def __init__(self, sequencing: Sequencing = Sequencing.PREFER_ORDERED):
        assert isinstance(sequencing, Sequencing)
        self.sequencing = sequencing

    @property
    def kind(self):
        return SafetySelectionKind.UNSAFE

    @classmethod
    def from_json(cls, j: dict) -> Self:
        if "Unsafe" in j:
            return cls(Sequencing(j["Unsafe"]))
        raise Exception("Invalid SafetySelectionUnsafe")

    def to_json(self) -> dict:
        return {"Unsafe": self.sequencing}

@final
class SafetySelectionSafe(SafetySelection):
    safety_spec: SafetySpec

    def __init__(self, safety_spec: SafetySpec):
        assert isinstance(safety_spec, SafetySpec)
        self.safety_spec = safety_spec

    @property
    def kind(self):
        return SafetySelectionKind.SAFE

    @classmethod
    def from_json(cls, j: dict) -> Self:
        if "Safe" in j:
            return cls(SafetySpec.from_json(j["Safe"]))
        raise Exception("Invalid SafetySelectionUnsafe")

    def to_json(self) -> dict:
        return {"Safe": self.safety_spec.to_json()}



class Target(ABC):

    @property
    @abstractmethod
    def kind(self) -> TargetKind:
        pass

    @classmethod
    def node_id(cls, node_id: NodeId) -> Self:
        return TargetNodeId(node_id=node_id) # type: ignore

    @classmethod
    def route_id(cls, route_id: RouteId) -> Self:
        return TargetRouteId(route_id=route_id) # type: ignore

    @classmethod
    def from_json(cls, j: dict) -> Self:
        if "NodeId" in j:
            return cls.node_id(NodeId(j["NodeId"]))
        elif "RouteId" in j:
            return cls.route_id(RouteId(j["Unsafe"]))
        raise Exception("Invalid Target")

    @abstractmethod
    def to_json(self) -> dict:
        pass

@final
class TargetNodeId(Target):
    id: NodeId

    def __init__(self, node_id: NodeId):
        assert isinstance(node_id, NodeId)
        self.id = node_id

    @property
    def kind(self):
        return TargetKind.NODE_ID

    @classmethod
    def from_json(cls, j: dict) -> Self:
        if "NodeId" in j:
            return cls(NodeId(j["NodeId"]))
        raise Exception("Invalid TargetNodeId")

    def to_json(self) -> dict:
        return {"NodeId": self.id}

@final
class TargetRouteId(Target):
    id: RouteId

    def __init__(self, route_id: RouteId):
        assert isinstance(route_id, RouteId)
        self.id = route_id

    @property
    def kind(self):
        return TargetKind.ROUTE_ID

    @classmethod
    def from_json(cls, j: dict) -> Self:
        if "RouteId" in j:
            return cls(RouteId(j["RouteId"]))
        raise Exception("Invalid TargetRouteId")

    def to_json(self) -> dict:
        return {"RouteId": self.id}
