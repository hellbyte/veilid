import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:veilid/veilid.dart';

final bogusKey =
    RecordKey.fromString('VLD0:qD10lHHPD1_Qr23_Qy-1JnxTht12eaWwENVG_m2v7II');

class TestDHT {
  late final VeilidRoutingContext routingContext;
  final CryptoKind cryptoKind;

  TestDHT(this.cryptoKind);

  Future<void> setUpAll() async {
    routingContext = await Veilid.instance.routingContext();
  }

  Future<void> tearDownAll() async {
    routingContext.close();
  }

  Future<void> testGetDHTValueUnopened() async {
    await expectLater(() => routingContext.getDHTValue(bogusKey, 0),
        throwsA(isA<VeilidAPIException>()));
  }

  Future<void> testOpenDHTRecordNonexistentNoWriter() async {
    await expectLater(() => routingContext.openDHTRecord(bogusKey),
        throwsA(isA<VeilidAPIException>()));
  }

  Future<void> testCloseDHTRecordNonexistent() async {
    await expectLater(() => routingContext.closeDHTRecord(bogusKey),
        throwsA(isA<VeilidAPIException>()));
  }

  Future<void> testDeleteDHTRecordNonexistent() async {
    await expectLater(() => routingContext.deleteDHTRecord(bogusKey),
        throwsA(isA<VeilidAPIException>()));
  }

  Future<void> testCreateDeleteDHTRecordSimple() async {
    final rec = await routingContext.createDHTRecord(
        cryptoKind, const DHTSchema.dflt(oCnt: 1));
    await routingContext.closeDHTRecord(rec.key);
    await routingContext.deleteDHTRecord(rec.key);
  }

  Future<void> testCreateDeleteDHTRecordNoClose() async {
    final rec = await routingContext.createDHTRecord(
        cryptoKind, const DHTSchema.dflt(oCnt: 1));
    await routingContext.deleteDHTRecord(rec.key);
  }

  Future<void> testGetDHTValueNonexistent() async {
    final rec = await routingContext.createDHTRecord(
        cryptoKind, const DHTSchema.dflt(oCnt: 1));
    expect(await routingContext.getDHTValue(rec.key, 0), isNull);
    await routingContext.deleteDHTRecord(rec.key);
  }

  Future<void> testSetGetDHTValue() async {
    final rec = await routingContext.createDHTRecord(
        cryptoKind, const DHTSchema.dflt(oCnt: 2));
    expect(
        await routingContext.setDHTValue(
            rec.key, 0, utf8.encode('BLAH BLAH BLAH')),
        isNull);
    final vd2 = await routingContext.getDHTValue(rec.key, 0);
    expect(vd2, isNotNull);

    final vd3 =
        await routingContext.getDHTValue(rec.key, 0, forceRefresh: true);
    expect(vd3, isNotNull);

    final vd4 = await routingContext.getDHTValue(rec.key, 1);
    expect(vd4, isNull);

    expect(vd2, equals(vd3));

    await routingContext.deleteDHTRecord(rec.key);
  }

  Future<void> testSetGetDHTValueWithOwner() async {
    final cs = await Veilid.instance.getCryptoSystem(cryptoKind);

    final ownerKeyPair = await cs.generateKeyPair();

    final rec = await routingContext.createDHTRecord(
        cryptoKind, const DHTSchema.dflt(oCnt: 2),
        owner: ownerKeyPair);
    expect(
        await routingContext.setDHTValue(
            rec.key, 0, utf8.encode('BLAH BLAH BLAH'),
            options: const SetDHTValueOptions(allowOffline: false)),
        isNull);
    final vd2 = await routingContext.getDHTValue(rec.key, 0);
    expect(vd2, isNotNull);

    final vd3 =
        await routingContext.getDHTValue(rec.key, 0, forceRefresh: true);
    expect(vd3, isNotNull);

    final vd4 = await routingContext.getDHTValue(rec.key, 1);
    expect(vd4, isNull);

    expect(vd2, equals(vd3));

    await routingContext.deleteDHTRecord(rec.key);
  }

  Future<void> testCreateDHTRecordWithDeterministicKey() async {
    final cs = await Veilid.instance.getCryptoSystem(cryptoKind);
    final ownerKeyPair = await cs.generateKeyPair();
    final owner = ownerKeyPair.key;
    final secret = ownerKeyPair.secret;
    const schema = DHTSchema.dflt(oCnt: 1);
    final dhtRecord = await routingContext.createDHTRecord(
        cs.kind(), const DHTSchema.dflt(oCnt: 1),
        owner: ownerKeyPair);
    final encryptionKey = dhtRecord.key.encryptionKey;
    final dhtRecordKey =
        await Veilid.instance.getDHTRecordKey(schema, owner, encryptionKey);
    expect(dhtRecord.key, equals(dhtRecordKey));
    expect(dhtRecord.owner, equals(owner));
    expect(dhtRecord.ownerSecret, equals(secret));
    expect(dhtRecord.schema, equals(schema));
    await routingContext.closeDHTRecord(dhtRecord.key);
    await routingContext.deleteDHTRecord(dhtRecord.key);
  }

  Future<void> testOpenWriterDHTValue() async {
    final cs = await Veilid.instance.getCryptoSystem(cryptoKind);

    var rec = await routingContext.createDHTRecord(
        cs.kind(), const DHTSchema.dflt(oCnt: 2));
    final key = rec.key;
    final owner = rec.owner;
    final secret = rec.ownerSecret!;

    expect(await cs.validateKeyPair(owner, secret), isTrue);
    final otherKeyPair = await cs.generateKeyPair();

    final va = utf8.encode('Qwertyuiop Asdfghjkl Zxcvbnm');
    final vb = utf8.encode('1234567890');
    final vc = utf8.encode(r'!@#$%^&*()');

    // Test subkey writes
    expect(await routingContext.setDHTValue(key, 1, va), isNull);

    var vdtemp = await routingContext.getDHTValue(key, 1);
    expect(vdtemp, isNotNull);
    expect(vdtemp!.data, equals(va));
    expect(vdtemp.seq, equals(0));
    expect(vdtemp.writer, equals(owner));

    expect(await routingContext.getDHTValue(key, 0), isNull);

    expect(await routingContext.setDHTValue(key, 0, vb), isNull);

    expect(
        await routingContext.getDHTValue(key, 0, forceRefresh: true),
        equals(ValueData(
          data: vb,
          seq: 0,
          writer: owner,
        )));

    expect(
        await routingContext.getDHTValue(key, 1, forceRefresh: true),
        equals(ValueData(
          data: va,
          seq: 0,
          writer: owner,
        )));

    // Equal value should not trigger sequence number update
    expect(await routingContext.setDHTValue(key, 1, va), isNull);

    // Different value should trigger sequence number update
    expect(await routingContext.setDHTValue(key, 1, vb), isNull);

    await settle(key, 0);
    await settle(key, 1);

    // Now that we initialized some subkeys
    // and verified they stored correctly
    // Delete things locally and reopen and see if we can write
    // with the same writer key
    //

    await routingContext.closeDHTRecord(key);
    await routingContext.deleteDHTRecord(key);

    rec = await routingContext.openDHTRecord(key,
        writer: KeyPair(key: owner, secret: secret));
    expect(rec, isNotNull);
    expect(rec.key, equals(key));
    expect(rec.owner, equals(owner));
    expect(rec.ownerSecret, equals(secret));
    expect(rec.schema, isA<DHTSchemaDFLT>());
    expect(rec.schema.oCnt, equals(2));

    // Verify subkey 1 can be set before it is get but newer is available online
    vdtemp = await routingContext.setDHTValue(key, 1, vc);
    expect(vdtemp, isNotNull);
    expect(vdtemp!.data, equals(vb));
    expect(vdtemp.seq, equals(1));
    expect(vdtemp.writer, equals(owner));

    // Verify subkey 1 can be set a second time
    // and it updates because seq is newer
    expect(await routingContext.setDHTValue(key, 1, vc), isNull);

    // Verify the network got the subkey update with a refresh check
    vdtemp = await routingContext.getDHTValue(key, 1, forceRefresh: true);
    expect(vdtemp, isNotNull);
    expect(vdtemp!.data, equals(vc));
    expect(vdtemp.seq, equals(2));
    expect(vdtemp.writer, equals(owner));

    // Delete things locally and reopen and see if we can write
    // with a different writer key (should fail)
    await routingContext.closeDHTRecord(key);
    await routingContext.deleteDHTRecord(key);

    rec = await routingContext.openDHTRecord(key, writer: otherKeyPair);
    expect(rec, isNotNull);
    expect(rec.key, equals(key));
    expect(rec.owner, equals(owner));
    expect(rec.ownerSecret, isNull);
    expect(rec.schema, isA<DHTSchemaDFLT>());
    expect(rec.schema.oCnt, equals(2));

    // Verify subkey 1 can NOT be set because we have the wrong writer
    await expectLater(() => routingContext.setDHTValue(key, 1, va),
        throwsA(isA<VeilidAPIException>()));

    // Verify subkey 0 can NOT be set because we have the wrong writer
    await expectLater(() => routingContext.setDHTValue(key, 0, va),
        throwsA(isA<VeilidAPIException>()));

    // Verify subkey 0 can be set because override with the right writer
    // Should have prior sequence number as its returned value because it
    // exists online at seq 0
    vdtemp = await routingContext.setDHTValue(key, 0, va,
        options:
            SetDHTValueOptions(writer: KeyPair(key: owner, secret: secret)));
    expect(vdtemp, isNotNull);
    expect(vdtemp!.data, equals(vb));
    expect(vdtemp.seq, equals(0));
    expect(vdtemp.writer, equals(owner));

    // Should update the second time to seq 1
    vdtemp = await routingContext.setDHTValue(key, 0, va,
        options: SetDHTValueOptions(
            writer: KeyPair(key: owner, secret: secret), allowOffline: false));
    expect(vdtemp, isNull);

    // Clean up
    await routingContext.closeDHTRecord(key);
    await routingContext.deleteDHTRecord(key);
  }

  Future<void> settle(RecordKey key, int subkey) async {
    // Wait for set to settle
    do {
      await Future<void>.delayed(const Duration(milliseconds: 100));
    } while ((await routingContext.inspectDHTRecord(key))
        .offlineSubkeys
        .containsSubkey(subkey));
  }

  Future<void> testInspectDHTRecord() async {
    final rec = await routingContext.createDHTRecord(
        cryptoKind, const DHTSchema.dflt(oCnt: 2));

    expect(
        await routingContext.setDHTValue(
            rec.key, 0, utf8.encode('BLAH BLAH BLAH'),
            options: const SetDHTValueOptions(allowOffline: false)),
        isNull);

    final rr = await routingContext.inspectDHTRecord(rec.key);
    expect(rr.subkeys, equals([ValueSubkeyRange.make(0, 1)]));
    expect(rr.localSeqs, equals([0, null]));
    expect(rr.networkSeqs, equals([null, null]));

    final rr2 = await routingContext.inspectDHTRecord(rec.key,
        scope: DHTReportScope.syncGet);
    expect(rr2.subkeys, equals([ValueSubkeyRange.make(0, 1)]));
    expect(rr2.localSeqs, equals([0, null]));
    expect(rr2.networkSeqs, equals([0, null]));

    await routingContext.closeDHTRecord(rec.key);
    await routingContext.deleteDHTRecord(rec.key);
  }
}
