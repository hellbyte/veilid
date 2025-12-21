import 'dart:async';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:veilid/veilid.dart';

final bogusKey =
    RecordKey.fromString('VLD0:qD10lHHPD1_Qr23_Qy-1JnxTht12eaWwENVG_m2v7II');

class TestDHTTransactionsFull {
  late final VeilidRoutingContext routingContext;
  final CryptoKind cryptoKind;
  final dhtRecordCount = 8;
  final dhtSubkeySize = 32768;
  final dhtSubkeyCount = 32;
  final data = <Uint8List>[];
  final dhtRecords = <RecordKey>[];

  TestDHTTransactionsFull(this.cryptoKind);

  Future<void> setUpAll() async {
    routingContext = await Veilid.instance.routingContext();

    for (var rec = 0; rec < dhtRecordCount; rec++) {
      data.add(Uint8List.fromList(List.filled(dhtSubkeySize, rec)));
    }
  }

  Future<void> tearDownAll() async {
    routingContext.close();
  }

  Future<void> setUp() async {
    dhtRecords.clear();
    for (var rec = 0; rec < dhtRecordCount; rec++) {
      final dhtRecord = await routingContext.createDHTRecord(
        cryptoKind,
        DHTSchema.dflt(oCnt: dhtSubkeyCount),
      );
      dhtRecords.add(dhtRecord.key);
    }
  }

  Future<void> tearDown() async {
    for (var rec = 0; rec < dhtRecordCount; rec++) {
      await routingContext.deleteDHTRecord(dhtRecords[rec]);
    }
    dhtRecords.clear();
  }

  Future<void> testEmptyTxInspectGetsAndCommit() async {
    // Begin
    final tx = await Veilid.instance.transactDHTRecords(dhtRecords);
    expect(tx, isInstanceOf<VeilidDHTTransaction>());

    // Inspect 1
    final inspects1 = <Future<(int, DHTRecordReport)>>[];
    for (var rec = 0; rec < dhtRecordCount; rec++) {
      inspects1.add(() async {
        final report =
            await tx.inspect(dhtRecords[rec], scope: DHTReportScope.syncGet);
        return (rec, report);
      }());
    }
    final allInspects1Res = await inspects1.wait;
    final expectedSeqs = List.filled(dhtSubkeyCount, null);
    for (final res in allInspects1Res) {
      expect(res.$2.subkeys,
          equals([ValueSubkeyRange(low: 0, high: dhtSubkeyCount - 1)]));
      expect(res.$2.localSeqs, equals(expectedSeqs));
      expect(res.$2.networkSeqs, equals(expectedSeqs));
      expect(res.$2.offlineSubkeys, equals([]));
    }
    // Gets
    for (var subkey = 0; subkey < dhtSubkeyCount; subkey++) {
      final gets = <Future<ValueData?>>[];
      for (var rec = 0; rec < dhtRecordCount; rec++) {
        gets.add(tx.get(dhtRecords[rec], subkey));
      }

      final allGetRes = await gets.wait;
      for (final res in allGetRes) {
        expect(res, isNull);
      }
    }
    await tx.commit();
  }

  Future<void> testTxFillCommitTxGetRollback() async {
    final startBegin = Veilid.instance.now();
    // Begin
    final tx = await Veilid.instance.transactDHTRecords(dhtRecords);
    expect(tx, isInstanceOf<VeilidDHTTransaction>());
    print('begin transaction: ${Veilid.instance.now().diff(startBegin)}');

    // Sets
    for (var subkey = 0; subkey < dhtSubkeyCount; subkey++) {
      final startSet = Veilid.instance.now();

      final sets = <Future<ValueData?>>[];
      for (var rec = 0; rec < dhtRecordCount; rec++) {
        sets.add(tx.set(dhtRecords[rec], subkey, data[rec]));
      }
      final allSetsRes = await sets.wait;
      for (final res in allSetsRes) {
        expect(res, isNull);
      }

      print('set subkey $subkey: ${Veilid.instance.now().diff(startSet)}');
    }

    // Commit
    final startCommit = Veilid.instance.now();
    await tx.commit();
    print('commit transaction: ${Veilid.instance.now().diff(startCommit)}');

    // Transaction 2
    final startCommit2 = Veilid.instance.now();
    final tx2 = await Veilid.instance.transactDHTRecords(dhtRecords);
    expect(tx2, isInstanceOf<VeilidDHTTransaction>());
    print('begin transaction 2: ${Veilid.instance.now().diff(startCommit2)}');

    // Gets
    for (var subkey = 0; subkey < dhtSubkeyCount; subkey++) {
      final startGet = Veilid.instance.now();

      final gets = <Future<(int, int, ValueData?)>>[];
      for (var rec = 0; rec < dhtRecordCount; rec++) {
        gets.add(() async {
          final val = await tx2.get(dhtRecords[rec], subkey);
          return (rec, subkey, val);
        }());
      }

      final allGetsRes = await gets.wait;
      for (final res in allGetsRes) {
        expect(res.$3, isNotNull);
        expect(res.$3!.data, equals(data[res.$1]));
        expect(res.$3!.seq, equals(0));
      }

      print('get subkey $subkey: ${Veilid.instance.now().diff(startGet)}');
    }
    final startRollback = Veilid.instance.now();
    await tx2.rollback();
    print(
        'rollback transaction 2: ${Veilid.instance.now().diff(startRollback)}');
  }
}
