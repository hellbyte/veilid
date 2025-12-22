import 'dart:async';
import 'dart:typed_data';

import 'package:freezed_annotation/freezed_annotation.dart';

import 'veilid.dart';

part 'veilid_dht_transaction.freezed.dart';
part 'veilid_dht_transaction.g.dart';

//////////////////////////////////////
/// DHTTransactionSetValueOptions

@freezed
sealed class DHTTransactionSetValueOptions
    with _$DHTTransactionSetValueOptions {
  const factory DHTTransactionSetValueOptions({KeyPair? writer}) =
      _DHTTransactionSetValueOptions;

  factory DHTTransactionSetValueOptions.fromJson(dynamic json) =>
      _$DHTTransactionSetValueOptionsFromJson(json as Map<String, dynamic>);

  @override
  Map<String, dynamic> toJson() => {'writer': writer};
}

//////////////////////////////////////
/// VeilidDHTTransaction

abstract class VeilidDHTTransaction {
  bool get isDone;

  Future<void> commit();
  Future<void> rollback();
  Future<ValueData?> get(RecordKey key, int subkey);
  Future<ValueData?> set(
    RecordKey key,
    int subkey,
    Uint8List data, {
    DHTTransactionSetValueOptions? options,
  });
  Future<DHTRecordReport> inspect(
    RecordKey key, {
    List<ValueSubkeyRange>? subkeys,
    DHTReportScope scope = DHTReportScope.local,
  });
}
