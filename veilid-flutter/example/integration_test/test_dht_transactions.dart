import 'dart:async';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:veilid/veilid.dart';

final bogusKey =
    RecordKey.fromString('VLD0:qD10lHHPD1_Qr23_Qy-1JnxTht12eaWwENVG_m2v7II');

class TestDHTTransactions {
  late final VeilidRoutingContext routingContext;
  final CryptoKind cryptoKind;
  final dhtRecordCount = 4;
  final dhtSubkeySize = 16;
  final dhtSubkeyCount = 2;
  final data = <Uint8List>[];
  final dhtRecords = <RecordKey>[];

  TestDHTTransactions(this.cryptoKind);

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

  Future<void> testEmptyTxAndDrop() async {
    final tx = await Veilid.instance.transactDHTRecords(dhtRecords);
    expect(tx, isInstanceOf<VeilidDHTTransaction>());
  }

  Future<void> testEmptyTxAndRollback() async {
    final tx = await Veilid.instance.transactDHTRecords(dhtRecords);
    expect(tx, isInstanceOf<VeilidDHTTransaction>());
    await tx.rollback();
  }

  Future<void> testEmptyTxAndCommit() async {
    final tx = await Veilid.instance.transactDHTRecords(dhtRecords);
    expect(tx, isInstanceOf<VeilidDHTTransaction>());
    await tx.commit();
  }

  Future<void> testTxAddSetsAndRollback() async {
    final tx = await Veilid.instance.transactDHTRecords(dhtRecords);
    expect(tx, isInstanceOf<VeilidDHTTransaction>());
    for (var rec = 0; rec < dhtRecordCount; rec++) {
      final res = await tx.set(dhtRecords[rec], 0, data[rec]);
      expect(res, isNull);
    }

    await tx.commit();
  }

  Future<void> testTxAddSetsAndCommit() async {
    final tx = await Veilid.instance.transactDHTRecords(dhtRecords);
    expect(tx, isInstanceOf<VeilidDHTTransaction>());
    for (var rec = 0; rec < dhtRecordCount; rec++) {
      final res = await tx.set(dhtRecords[rec], 0, data[rec]);
      expect(res, isNull);
    }

    await tx.commit();
  }

  Future<void> testTxAddSetsGetsAndCommit() async {
    final tx = await Veilid.instance.transactDHTRecords(dhtRecords);
    expect(tx, isInstanceOf<VeilidDHTTransaction>());

    final sets = <Future<ValueData?>>[];
    for (var rec = 0; rec < dhtRecordCount; rec++) {
      sets.add(tx.set(dhtRecords[rec], 0, data[rec]));
    }
    final allSetRes = await sets.wait;
    for (final res in allSetRes) {
      expect(res, isNull);
    }

    final gets = <Future<ValueData?>>[];
    for (var rec = 0; rec < dhtRecordCount; rec++) {
      gets.add(tx.get(dhtRecords[rec], 0));
    }

    final allGetRes = await gets.wait;
    for (final res in allGetRes) {
      expect(res, isNull);
    }

    await tx.commit();
  }

  Future<void> testTxFailNonTxSetsAndRollback() async {
    final tx = await Veilid.instance.transactDHTRecords(dhtRecords);
    expect(tx, isInstanceOf<VeilidDHTTransaction>());

    final sets = <Future<ValueData?>>[];
    for (var subkey = 0; subkey < dhtSubkeyCount; subkey++) {
      for (var rec = 0; rec < dhtRecordCount; rec++) {
        sets.add(
            routingContext.setDHTValue(dhtRecords[rec], subkey, data[rec]));
      }
    }
    try {
      await sets.wait;
      // Must catch error for test
      // ignore: avoid_catching_errors
    } on ParallelWaitError<List<ValueData?>, List<AsyncError?>> catch (pwerr) {
      for (final ex in pwerr.errors) {
        expect(ex?.error, isA<VeilidAPIExceptionTryAgain>());
      }
    }

    await tx.rollback();
  }

  Future<void> testTxInspectAddInspectCommit() async {
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
    for (final res in allInspects1Res) {
      expect(res.$2.subkeys, equals([const ValueSubkeyRange(low: 0, high: 1)]));
      expect(res.$2.localSeqs, equals([null, null]));
      expect(res.$2.networkSeqs, equals([null, null]));
      expect(res.$2.offlineSubkeys, equals([]));
    }

    // Sets
    final sets = <Future<ValueData?>>[];
    for (var rec = 0; rec < dhtRecordCount; rec++) {
      sets.add(tx.set(dhtRecords[rec], 1, data[rec]));
    }
    final allSetRes = await sets.wait;
    for (final res in allSetRes) {
      expect(res, isNull);
    }

    // Inspect 2 (should be the same because we haven't committed yet)
    final inspects2 = <Future<(int, DHTRecordReport)>>[];
    for (var rec = 0; rec < dhtRecordCount; rec++) {
      inspects2.add(() async {
        final report =
            await tx.inspect(dhtRecords[rec], scope: DHTReportScope.syncGet);
        return (rec, report);
      }());
    }
    final allInspects2Res = await inspects2.wait;
    for (final res in allInspects2Res) {
      expect(res.$2.subkeys, equals([const ValueSubkeyRange(low: 0, high: 1)]));
      expect(res.$2.localSeqs, equals([null, null]));
      expect(res.$2.networkSeqs, equals([null, null]));
      expect(res.$2.offlineSubkeys, equals([]));
    }

    // Commit
    await tx.commit();

    // Transaction 2
    final tx2 = await Veilid.instance.transactDHTRecords(dhtRecords);
    expect(tx2, isInstanceOf<VeilidDHTTransaction>());

    // Inspect 3 (should be updated post-commit)
    final inspects3 = <Future<(int, DHTRecordReport)>>[];
    for (var rec = 0; rec < dhtRecordCount; rec++) {
      inspects3.add(() async {
        final report =
            await tx2.inspect(dhtRecords[rec], scope: DHTReportScope.syncGet);
        return (rec, report);
      }());
    }
    final allInspects3Res = await inspects3.wait;
    for (final res in allInspects3Res) {
      expect(res.$2.subkeys, equals([const ValueSubkeyRange(low: 0, high: 1)]));
      expect(res.$2.localSeqs, equals([null, 0]));
      expect(res.$2.networkSeqs, equals([null, 0]));
      expect(res.$2.offlineSubkeys, equals([]));
    }

    // Gets should match inspect
    const expected = [null, 0];
    for (var subkey = 0; subkey < dhtSubkeyCount; subkey++) {
      final gets = <Future<(int, int, ValueData?)>>[];
      for (var rec = 0; rec < dhtRecordCount; rec++) {
        gets.add(() async {
          final val = await tx2.get(dhtRecords[rec], subkey);
          return (rec, subkey, val);
        }());
      }

      final allGetRes = await gets.wait;
      for (final res in allGetRes) {
        if (expected[res.$2] == null) {
          expect(res.$3, isNull);
        } else {
          expect(res.$3, isNotNull);
          expect(res.$3!.data, equals(data[res.$1]));
          expect(res.$3!.seq, equals(expected[res.$2]));
        }
      }
    }
    await tx2.commit();
  }
}
