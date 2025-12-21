import 'dart:convert';

import 'package:equatable/equatable.dart';
import 'package:flutter/foundation.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

import 'veilid_stub.dart'
    if (dart.library.io) 'veilid_ffi.dart'
    if (dart.library.js) 'veilid_js.dart';

String base64UrlNoPadEncode(List<int> bytes) {
  var x = base64Url.encode(bytes);
  while (x.endsWith('=')) {
    x = x.substring(0, x.length - 1);
  }
  return x;
}

Uint8List base64UrlNoPadDecode(String source) {
  source = base64Url.normalize(source);
  return base64Url.decode(source);
}

Uint8List base64UrlNoPadDecodeDynamic(dynamic source) =>
    base64UrlNoPadDecode(source as String);

class Uint8ListJsonConverter implements JsonConverter<Uint8List, dynamic> {
  final bool _jsIsArray;

  const Uint8ListJsonConverter() : _jsIsArray = false;

  const Uint8ListJsonConverter.jsIsArray() : _jsIsArray = true;

  @override
  Uint8List fromJson(dynamic json) => kIsWeb && _jsIsArray
      ? convertUint8ListFromJson(json)
      : base64UrlNoPadDecode(json as String);

  @override
  dynamic toJson(Uint8List data) => kIsWeb && _jsIsArray
      ? convertUint8ListToJson(data)
      : base64UrlNoPadEncode(data);
}

@immutable
sealed class EncodedString extends Equatable {
  ////////////////////////////////////////////////////////////////////////////

  final String contents;

  EncodedString._fromBytes(Uint8List bytes)
      : contents = base64UrlNoPadEncode(bytes);

  EncodedString._fromString(String s) : contents = s {
    // Ensure things can be decoded, will throw an exception if it fails
    base64UrlNoPadDecode(contents);
  }

  EncodedString._fromJson(dynamic json) : contents = json as String {
    // Ensure things can be decoded, will throw an exception if it fails
    base64UrlNoPadDecode(contents);
  }

  String toJson() => toString();

  Uint8List toBytes() => base64UrlNoPadDecode(contents);

  @override
  String toString() => contents;

  ////////////////////////////////////////////////////////////////////////////

  static T fromBytes<T extends EncodedString>(Uint8List bytes) {
    switch (T) {
      case const (BarePublicKey):
        return BarePublicKey.fromBytes(bytes) as T;
      case const (BareSignature):
        return BareSignature.fromBytes(bytes) as T;
      case const (Nonce):
        return Nonce.fromBytes(bytes) as T;
      case const (BareSecretKey):
        return BareSecretKey.fromBytes(bytes) as T;
      case const (BareHashDigest):
        return BareHashDigest.fromBytes(bytes) as T;
      case const (BareOpaqueRecordKey):
        return BareOpaqueRecordKey.fromBytes(bytes) as T;
      case const (BareSharedSecret):
        return BareSharedSecret.fromBytes(bytes) as T;
      case const (BareRouteId):
        return BareRouteId.fromBytes(bytes) as T;
      case const (BareNodeId):
        return BareNodeId.fromBytes(bytes) as T;
      case const (BareMemberId):
        return BareMemberId.fromBytes(bytes) as T;
      default:
        throw UnimplementedError();
    }
  }

  static T fromString<T extends EncodedString>(String s) {
    switch (T) {
      case const (BarePublicKey):
        return BarePublicKey.fromString(s) as T;
      case const (BareSignature):
        return BareSignature.fromString(s) as T;
      case const (Nonce):
        return Nonce.fromString(s) as T;
      case const (BareSecretKey):
        return BareSecretKey.fromString(s) as T;
      case const (BareHashDigest):
        return BareHashDigest.fromString(s) as T;
      case const (BareOpaqueRecordKey):
        return BareOpaqueRecordKey.fromString(s) as T;
      case const (BareSharedSecret):
        return BareSharedSecret.fromString(s) as T;
      case const (BareRouteId):
        return BareRouteId.fromString(s) as T;
      case const (BareNodeId):
        return BareNodeId.fromString(s) as T;
      case const (BareMemberId):
        return BareMemberId.fromString(s) as T;
      default:
        throw UnimplementedError();
    }
  }

  static T fromJson<T extends EncodedString>(dynamic json) {
    switch (T) {
      case const (BarePublicKey):
        return BarePublicKey.fromJson(json) as T;
      case const (BareSignature):
        return BareSignature.fromJson(json) as T;
      case const (Nonce):
        return Nonce.fromJson(json) as T;
      case const (BareSecretKey):
        return BareSecretKey.fromJson(json) as T;
      case const (BareHashDigest):
        return BareHashDigest.fromJson(json) as T;
      case const (BareOpaqueRecordKey):
        return BareOpaqueRecordKey.fromJson(json) as T;
      case const (BareSharedSecret):
        return BareSharedSecret.fromJson(json) as T;
      case const (BareRouteId):
        return BareRouteId.fromJson(json) as T;
      case const (BareNodeId):
        return BareNodeId.fromJson(json) as T;
      case const (BareMemberId):
        return BareMemberId.fromJson(json) as T;
      default:
        throw UnimplementedError();
    }
  }

  @override
  List<Object> get props => [contents];
}

class BarePublicKey extends EncodedString {
  BarePublicKey.fromBytes(super.bytes) : super._fromBytes();
  BarePublicKey.fromString(super.s) : super._fromString();
  BarePublicKey.fromJson(super.json) : super._fromJson();
}

class BareSignature extends EncodedString {
  BareSignature.fromBytes(super.bytes) : super._fromBytes();
  BareSignature.fromString(super.s) : super._fromString();
  BareSignature.fromJson(super.json) : super._fromJson();
}

class Nonce extends EncodedString {
  Nonce.fromBytes(super.bytes) : super._fromBytes();
  Nonce.fromString(super.s) : super._fromString();
  Nonce.fromJson(super.json) : super._fromJson();
}

class BareSecretKey extends EncodedString {
  BareSecretKey.fromBytes(super.bytes) : super._fromBytes();
  BareSecretKey.fromString(super.s) : super._fromString();
  BareSecretKey.fromJson(super.json) : super._fromJson();
}

class BareHashDigest extends EncodedString {
  BareHashDigest.fromBytes(super.bytes) : super._fromBytes();
  BareHashDigest.fromString(super.s) : super._fromString();
  BareHashDigest.fromJson(super.json) : super._fromJson();
}

class BareOpaqueRecordKey extends EncodedString {
  BareOpaqueRecordKey.fromBytes(super.bytes) : super._fromBytes();
  BareOpaqueRecordKey.fromString(super.s) : super._fromString();
  BareOpaqueRecordKey.fromJson(super.json) : super._fromJson();
}

class BareSharedSecret extends EncodedString {
  BareSharedSecret.fromBytes(super.bytes) : super._fromBytes();
  BareSharedSecret.fromString(super.s) : super._fromString();
  BareSharedSecret.fromJson(super.json) : super._fromJson();
}

class BareRouteId extends EncodedString {
  BareRouteId.fromBytes(super.bytes) : super._fromBytes();
  BareRouteId.fromString(super.s) : super._fromString();
  BareRouteId.fromJson(super.json) : super._fromJson();
}

class BareNodeId extends EncodedString {
  BareNodeId.fromBytes(super.bytes) : super._fromBytes();
  BareNodeId.fromString(super.s) : super._fromString();
  BareNodeId.fromJson(super.json) : super._fromJson();
}

class BareMemberId extends EncodedString {
  BareMemberId.fromBytes(super.bytes) : super._fromBytes();
  BareMemberId.fromString(super.s) : super._fromString();
  BareMemberId.fromJson(super.json) : super._fromJson();
}
