// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'veilid.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_TransactDHTRecordsOptions _$TransactDHTRecordsOptionsFromJson(
  Map<String, dynamic> json,
) => _TransactDHTRecordsOptions(
  defaultSigningKeyPair: json['defaultSigningKeyPair'] == null
      ? null
      : KeyPair.fromJson(json['defaultSigningKeyPair']),
);

Map<String, dynamic> _$TransactDHTRecordsOptionsToJson(
  _TransactDHTRecordsOptions instance,
) => <String, dynamic>{
  'defaultSigningKeyPair': instance.defaultSigningKeyPair?.toJson(),
};
