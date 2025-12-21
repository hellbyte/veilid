import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:charcode/charcode.dart';
import 'package:equatable/equatable.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

import 'veilid.dart';

//////////////////////////////////////
/// CryptoKind

@immutable
class CryptoKind extends Equatable {
  final int kind;
  const CryptoKind.fromInt(this.kind);
  // Allow const
  // ignore: prefer_constructors_over_static_methods
  static CryptoKind fromBytes(Uint8List b) =>
      CryptoKind.fromInt(ByteData.sublistView(b).getUint32(0));

  // Allow const
  // ignore: prefer_constructors_over_static_methods
  static CryptoKind fromString(String s) {
    if (s.codeUnits.length != 4) {
      throw const FormatException('malformed string');
    }
    return CryptoKind.fromInt(
      ByteData.sublistView(Uint8List.fromList(s.codeUnits)).getUint32(0),
    );
  }

  @override
  String toString() {
    final b = Uint8List(4);
    ByteData.sublistView(b).setUint32(0, kind);
    return b.map(String.fromCharCode).join();
  }

  int toInt() => kind;

  Uint8List toBytes() {
    final b = Uint8List(4);
    ByteData.sublistView(b).setUint32(0, kind);
    return b;
  }

  @override
  List<Object?> get props => [kind];
}

const cryptoKindVLD0 = CryptoKind.fromInt(
  $V << 24 | $L << 16 | $D << 8 | $0 << 0,
); // "VLD0"
// final cryptoKindVLD1 = CryptoKind.fromInt(
//     $V << 24 | $L << 16 | $D << 8 | $1 << 0); // "VLD1"
const cryptoKindNONE = CryptoKind.fromInt(
  $N << 24 | $O << 16 | $N << 8 | $E << 0,
); // "NONE"

const bestCryptoKind = cryptoKindVLD0;

//////////////////////////////////////
/// Types

abstract interface class TypedCryptoKey<V> {
  CryptoKind get kind;
  V get value;
}

@immutable
class Typed<V extends EncodedString> extends Equatable
    implements TypedCryptoKey<V> {
  @override
  final CryptoKind kind;

  @override
  final V value;

  const Typed({required this.kind, required this.value});

  factory Typed.fromString(String s) {
    final parts = s.split(':');
    if (parts.length < 2 || parts[0].codeUnits.length != 4) {
      throw const FormatException('malformed string');
    }
    final kind = CryptoKind.fromString(parts[0]);
    final value = EncodedString.fromString<V>(parts.sublist(1).join(':'));
    return Typed(kind: kind, value: value);
  }

  factory Typed.fromBytes(Uint8List b) {
    final kind = CryptoKind.fromBytes(b);
    final value = EncodedString.fromBytes<V>(b.sublist(4));
    return Typed(kind: kind, value: value);
  }

  factory Typed.fromJson(dynamic json) => Typed.fromString(json as String);

  @override
  List<Object> get props => [kind, value];

  @override
  String toString() => '$kind:$value';

  Uint8List toBytes() {
    final b = BytesBuilder()
      ..add(kind.toBytes())
      ..add(value.toBytes());
    return b.toBytes();
  }

  String toJson() => toString();
}

@immutable
class BareKeyPair extends Equatable {
  final BarePublicKey key;

  final BareSecretKey secret;

  const BareKeyPair({required this.key, required this.secret});

  factory BareKeyPair.fromString(String s) {
    final parts = s.split(':');
    if (parts.length != 2) {
      throw const FormatException('malformed string');
    }
    final key = BarePublicKey.fromString(parts[0]);
    final secret = BareSecretKey.fromString(parts[1]);
    return BareKeyPair(key: key, secret: secret);
  }

  factory BareKeyPair.fromJson(dynamic json) =>
      BareKeyPair.fromString(json as String);

  @override
  List<Object> get props => [key, secret];

  @override
  String toString() => '$key:$secret';

  String toJson() => toString();
}

@immutable
class KeyPair extends Equatable implements TypedCryptoKey<BareKeyPair> {
  final PublicKey key;

  final SecretKey secret;

  KeyPair({required this.key, required this.secret})
    : assert(key.kind == secret.kind, 'keypair parts must have same kind');

  factory KeyPair.fromString(String s) {
    final parts = s.split(':');
    if (parts.length != 3 || parts[0].codeUnits.length != 4) {
      throw VeilidAPIExceptionInvalidArgument('malformed string', 's', s);
    }
    final kind = CryptoKind.fromString(parts[0]);
    final key = PublicKey(
      kind: kind,
      value: BarePublicKey.fromString(parts[1]),
    );
    final secret = SecretKey(
      kind: kind,
      value: BareSecretKey.fromString(parts[2]),
    );
    return KeyPair(key: key, secret: secret);
  }

  factory KeyPair.fromJson(dynamic json) => KeyPair.fromString(json as String);

  factory KeyPair.fromBareKeyPair(CryptoKind kind, BareKeyPair keyPair) =>
      KeyPair(
        key: PublicKey(kind: kind, value: keyPair.key),
        secret: SecretKey(kind: kind, value: keyPair.secret),
      );

  factory KeyPair.fromPublicAndBareSecret(
    PublicKey key,
    BareSecretKey secret,
  ) => KeyPair(
    key: key,
    secret: SecretKey(kind: key.kind, value: secret),
  );

  @override
  CryptoKind get kind => key.kind;

  @override
  BareKeyPair get value => BareKeyPair(key: key.value, secret: secret.value);

  @override
  List<Object> get props => [key, secret];

  @override
  String toString() => '${key.kind}:${key.value}:${secret.value}';

  String toJson() => toString();

  BareKeyPair toBareKeyPair() =>
      BareKeyPair(key: key.value, secret: secret.value);
}

@immutable
class BareRecordKey extends Equatable {
  final BareOpaqueRecordKey key;

  final BareSharedSecret? encryptionKey;

  const BareRecordKey({required this.key, required this.encryptionKey});

  factory BareRecordKey.fromString(String s) {
    final parts = s.split(':');
    if (parts.length > 2 || parts.isEmpty) {
      throw const FormatException('malformed string');
    }
    if (parts.length == 2) {
      final key = BareOpaqueRecordKey.fromString(parts[0]);
      final encryptionKey = BareSharedSecret.fromString(parts[1]);
      return BareRecordKey(key: key, encryptionKey: encryptionKey);
    }
    final key = BareOpaqueRecordKey.fromString(parts[0]);
    return BareRecordKey(key: key, encryptionKey: null);
  }

  factory BareRecordKey.fromJson(dynamic json) =>
      BareRecordKey.fromString(json as String);

  @override
  List<Object?> get props => [key, encryptionKey];

  @override
  String toString() => encryptionKey != null ? '$key:$encryptionKey' : '$key';

  String toJson() => toString();
}

@immutable
class RecordKey extends Equatable implements TypedCryptoKey<BareRecordKey> {
  final OpaqueRecordKey opaque;

  final SharedSecret? encryptionKey;

  RecordKey({required this.opaque, required this.encryptionKey})
    : assert(
        encryptionKey == null || opaque.kind == encryptionKey.kind,
        'recordkey parts must have same kind',
      );

  factory RecordKey.fromString(String s) {
    final parts = s.split(':');
    if (parts.length < 2 ||
        parts.length > 3 ||
        parts[0].codeUnits.length != 4) {
      throw VeilidAPIExceptionInvalidArgument('malformed string', 's', s);
    }
    final kind = CryptoKind.fromString(parts[0]);
    final key = OpaqueRecordKey(
      kind: kind,
      value: BareOpaqueRecordKey.fromString(parts[1]),
    );
    if (parts.length == 3) {
      final encryptionKey = SharedSecret(
        kind: kind,
        value: BareSharedSecret.fromString(parts[2]),
      );
      return RecordKey(opaque: key, encryptionKey: encryptionKey);
    }
    return RecordKey(opaque: key, encryptionKey: null);
  }

  factory RecordKey.fromJson(dynamic json) =>
      RecordKey.fromString(json as String);

  factory RecordKey.fromBareRecordKey(
    CryptoKind kind,
    BareRecordKey bareRecordKey,
  ) => RecordKey(
    opaque: OpaqueRecordKey(kind: kind, value: bareRecordKey.key),
    encryptionKey: bareRecordKey.encryptionKey == null
        ? null
        : SharedSecret(kind: kind, value: bareRecordKey.encryptionKey!),
  );

  factory RecordKey.fromBytes(Uint8List bytes) {
    final keyLength = ByteData.sublistView(bytes).getUint8(0);
    final keyBytes = bytes.sublist(1, 1 + keyLength);
    final key = OpaqueRecordKey.fromBytes(keyBytes);
    SharedSecret? encryptionKey;
    if (bytes.length > 1 + keyLength) {
      final ekBytes = bytes.sublist(1 + keyLength, bytes.length);
      encryptionKey = SharedSecret(
        kind: key.kind,
        value: BareSharedSecret.fromBytes(ekBytes),
      );
    }
    return RecordKey(opaque: key, encryptionKey: encryptionKey);
  }

  @override
  List<Object?> get props => [opaque, encryptionKey];

  @override
  CryptoKind get kind => opaque.kind;

  @override
  BareRecordKey get value =>
      BareRecordKey(key: opaque.value, encryptionKey: encryptionKey?.value);

  @override
  String toString() => encryptionKey != null
      ? '${opaque.kind}:${opaque.value}:${encryptionKey!.value}'
      : '${opaque.kind}:${opaque.value}';

  String toJson() => toString();

  Uint8List toBytes() {
    final keyBytes = opaque.toBytes();
    final b = BytesBuilder()
      ..addByte(keyBytes.lengthInBytes)
      ..add(keyBytes);
    final ek = encryptionKey;
    if (ek != null) {
      b.add(ek.value.toBytes());
    }
    return b.toBytes();
  }
}

typedef PublicKey = Typed<BarePublicKey>;
typedef Signature = Typed<BareSignature>;
typedef SecretKey = Typed<BareSecretKey>;
typedef HashDigest = Typed<BareHashDigest>;
typedef SharedSecret = Typed<BareSharedSecret>;
typedef RouteId = Typed<BareRouteId>;
typedef NodeId = Typed<BareNodeId>;
typedef MemberId = Typed<BareMemberId>;
typedef OpaqueRecordKey = Typed<BareOpaqueRecordKey>;

//////////////////////////////////////
/// VeilidCryptoSystem

abstract class VeilidCryptoSystem {
  CryptoKind kind();

  // Cached Operations

  Future<SharedSecret> cachedDH(PublicKey key, SecretKey secret);

  // Generation

  Future<Uint8List> randomBytes(int len);
  Future<String> hashPassword(Uint8List password, Uint8List salt);
  Future<bool> verifyPassword(Uint8List password, String passwordHash);
  Future<SharedSecret> deriveSharedSecret(Uint8List password, Uint8List salt);
  Future<Nonce> randomNonce();
  Future<SharedSecret> randomSharedSecret();
  Future<SharedSecret> computeDH(PublicKey key, SecretKey secret);
  Future<SharedSecret> generateSharedSecret(
    PublicKey key,
    SecretKey secret,
    Uint8List domain,
  );
  Future<KeyPair> generateKeyPair();
  Future<HashDigest> generateHash(Uint8List data);
  //Future<HashDigest> generateHashReader(Stream<List<int>> reader);

  // Validation

  Future<int> sharedSecretLength();
  Future<int> nonceLength();
  Future<int> hashDigestLength();
  Future<int> publicKeyLength();
  Future<int> secretKeyLength();
  Future<int> signatureLength();
  Future<int> aeadOverhead();
  Future<int> defaultSaltLength();

  Future<void> checkSharedSecret(SharedSecret secret);
  Future<void> checkNonce(Nonce nonce);
  Future<void> checkHashDigest(HashDigest digest);
  Future<void> checkPublicKey(PublicKey key);
  Future<void> checkSecretKey(SecretKey key);
  Future<void> checkSignature(Signature signature);

  Future<bool> validateKeyPair(PublicKey key, SecretKey secret);
  Future<bool> validateKeyPairWithKeyPair(KeyPair keyPair) =>
      validateKeyPair(keyPair.key, keyPair.secret);

  Future<bool> validateHash(Uint8List data, HashDigest hash);
  //Future<bool> validateHashReader(Stream<List<int>> reader, HashDigest hash);

  // Authentication

  Future<Signature> sign(PublicKey key, SecretKey secret, Uint8List data);
  Future<Signature> signWithKeyPair(KeyPair keyPair, Uint8List data) =>
      sign(keyPair.key, keyPair.secret, data);

  Future<bool> verify(PublicKey key, Uint8List data, Signature signature);

  // AEAD Encrypt/Decrypt

  Future<Uint8List> decryptAead(
    Uint8List body,
    Nonce nonce,
    SharedSecret sharedSecret,
    Uint8List? associatedData,
  );
  Future<Uint8List> encryptAead(
    Uint8List body,
    Nonce nonce,
    SharedSecret sharedSecret,
    Uint8List? associatedData,
  );
  Future<Uint8List> cryptNoAuth(
    Uint8List body,
    Nonce nonce,
    SharedSecret sharedSecret,
  );

  Future<Uint8List> encryptAeadWithNonce(
    Uint8List body,
    SharedSecret secret,
  ) async {
    // generate nonce
    final nonce = await randomNonce();
    // crypt and append nonce
    final b = BytesBuilder()
      ..add(await encryptAead(body, nonce, secret, null))
      ..add(nonce.toBytes());
    return b.toBytes();
  }

  Future<Uint8List> decryptAeadWithNonce(
    Uint8List body,
    SharedSecret secret,
  ) async {
    final nlen = await nonceLength();
    if (body.length < nlen) {
      throw const FormatException('not enough data to decrypt');
    }
    final nonce = Nonce.fromBytes(body.sublist(body.length - nlen));
    final encryptedData = body.sublist(0, body.length - nlen);
    // decrypt
    return decryptAead(encryptedData, nonce, secret, null);
  }

  Future<Uint8List> encryptAeadWithPassword(
    Uint8List body,
    String password,
  ) async {
    final ekbytes = Uint8List.fromList(utf8.encode(password));
    final nonce = await randomNonce();
    final saltBytes = nonce.toBytes();
    final sharedSecret = await deriveSharedSecret(ekbytes, saltBytes);
    return Uint8List.fromList(
      (await encryptAead(body, nonce, sharedSecret, null)) + saltBytes,
    );
  }

  Future<Uint8List> decryptAeadWithPassword(
    Uint8List body,
    String password,
  ) async {
    final nlen = await nonceLength();
    if (body.length < nlen) {
      throw const FormatException('not enough data to decrypt');
    }
    final ekbytes = Uint8List.fromList(utf8.encode(password));
    final bodyBytes = body.sublist(0, body.length - nlen);
    final saltBytes = body.sublist(body.length - nlen);
    final nonce = Nonce.fromBytes(saltBytes);
    final sharedSecret = await deriveSharedSecret(ekbytes, saltBytes);
    return decryptAead(bodyBytes, nonce, sharedSecret, null);
  }

  // NoAuth Encrypt/Decrypt

  Future<Uint8List> encryptNoAuthWithNonce(
    Uint8List body,
    SharedSecret secret,
  ) async {
    // generate nonce
    final nonce = await randomNonce();
    // crypt and append nonce
    final b = BytesBuilder()
      ..add(await cryptNoAuth(body, nonce, secret))
      ..add(nonce.toBytes());
    return b.toBytes();
  }

  Future<Uint8List> decryptNoAuthWithNonce(
    Uint8List body,
    SharedSecret secret,
  ) async {
    final nlen = await nonceLength();
    if (body.length < nlen) {
      throw const FormatException('not enough data to decrypt');
    }
    final nonce = Nonce.fromBytes(body.sublist(body.length - nlen));
    final encryptedData = body.sublist(0, body.length - nlen);
    // decrypt
    return cryptNoAuth(encryptedData, nonce, secret);
  }

  Future<Uint8List> encryptNoAuthWithPassword(
    Uint8List body,
    String password,
  ) async {
    final ekbytes = Uint8List.fromList(utf8.encode(password));
    final nonce = await randomNonce();
    final saltBytes = nonce.toBytes();
    final sharedSecret = await deriveSharedSecret(ekbytes, saltBytes);
    return Uint8List.fromList(
      (await cryptNoAuth(body, nonce, sharedSecret)) + saltBytes,
    );
  }

  Future<Uint8List> decryptNoAuthWithPassword(
    Uint8List body,
    String password,
  ) async {
    final nlen = await nonceLength();
    if (body.length < nlen) {
      throw const FormatException('not enough data to decrypt');
    }
    final ekbytes = Uint8List.fromList(utf8.encode(password));
    final bodyBytes = body.sublist(0, body.length - nlen);
    final saltBytes = body.sublist(body.length - nlen);
    final nonce = Nonce.fromBytes(saltBytes);
    final sharedSecret = await deriveSharedSecret(ekbytes, saltBytes);
    return cryptNoAuth(bodyBytes, nonce, sharedSecret);
  }
}
