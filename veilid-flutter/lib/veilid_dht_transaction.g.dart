// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'veilid_dht_transaction.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_DHTTransactionSetValueOptions _$DHTTransactionSetValueOptionsFromJson(
  Map<String, dynamic> json,
) => _DHTTransactionSetValueOptions(
  writer: json['writer'] == null ? null : KeyPair.fromJson(json['writer']),
);

Map<String, dynamic> _$DHTTransactionSetValueOptionsToJson(
  _DHTTransactionSetValueOptions instance,
) => <String, dynamic>{'writer': instance.writer?.toJson()};
