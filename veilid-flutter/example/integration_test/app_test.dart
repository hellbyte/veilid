@Timeout(Duration(seconds: 500))

library;

import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:veilid/veilid.dart';
import 'package:veilid_test/veilid_test.dart';

import 'test_crypto.dart';
import 'test_dht.dart';
import 'test_dht_transactions.dart';
import 'test_dht_transactions_full.dart';
import 'test_routing_context.dart';
import 'test_table_db.dart';
import 'test_veilid_config.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  final fixture =
      DefaultVeilidFixture(programName: 'veilid_flutter integration test');

  group('Unstarted Tests', () {
    test('veilid config defaults', testVeilidConfigDefaults);
  });

  group('Started Tests', () {
    setUpAll(fixture.setUp);
    tearDownAll(fixture.tearDown);

    group('Crypto Tests', () {
      test('list cryptosystems', testListCryptoSystems);
      test('get cryptosystem', testGetCryptoSystems);
      test('get cryptosystem invalid', testGetCryptoSystemInvalid);
      test('hash and verify password', testHashAndVerifyPassword);
      test('sign and verify signature', testSignAndVerifySignature);
      test('sign and verify signatures', testSignAndVerifySignatures);
    });

    group('Table DB Tests', () {
      test('delete table db nonexistent', testDeleteTableDbNonExistent);
      test('open delete table db', testOpenDeleteTableDb);
      test('open twice table db', testOpenTwiceTableDb);
      test('open twice table db store load', testOpenTwiceTableDbStoreLoad);
      test('open twice table db store delete load',
          testOpenTwiceTableDbStoreDeleteLoad);
      test('resize table db', testResizeTableDb);
    });

    group('Attached Tests', () {
      setUpAll(fixture.attach);
      tearDownAll(fixture.detach);

      group('Routing Contexts', () {
        test('routing contexts', testRoutingContexts);
        test('app message loopback',
            () => testAppMessageLoopback(fixture.updateStream));
        test('app call loopback',
            () => testAppCallLoopback(fixture.updateStream));
        test('app message loopback big packets',
            () => testAppMessageLoopbackBigPackets(fixture.updateStream));
        test('app call loopback big packets',
            () => testAppCallLoopbackBigPackets(fixture.updateStream));
      });

      for (final cryptoKind in Veilid.instance.validCryptoKinds()) {
        group('Veilid DHT $cryptoKind', () {
          final testDHT = TestDHT(cryptoKind);
          setUpAll(testDHT.setUpAll);
          tearDownAll(testDHT.tearDownAll);

          test('get dht value unopened', testDHT.testGetDHTValueUnopened);
          test('open dht record nonexistent no writer',
              testDHT.testOpenDHTRecordNonexistentNoWriter);
          test('close dht record nonexistent',
              testDHT.testCloseDHTRecordNonexistent);
          test('delete dht record nonexistent',
              testDHT.testDeleteDHTRecordNonexistent);
          test('create delete dht record simple',
              testDHT.testCreateDeleteDHTRecordSimple);
          test('create delete dht record no close',
              testDHT.testCreateDeleteDHTRecordNoClose);
          test('create delete dht record with deterministic key',
              testDHT.testCreateDHTRecordWithDeterministicKey);
          test('get dht value nonexistent', testDHT.testGetDHTValueNonexistent);
          test('set get dht value', testDHT.testSetGetDHTValue);
          test('set get dht value with owner',
              testDHT.testSetGetDHTValueWithOwner);
          test('open writer dht value', testDHT.testOpenWriterDHTValue);
          test('inspect dht record', testDHT.testInspectDHTRecord);
        });

        group('Veilid DHT Transactions $cryptoKind', () {
          final testDHTTransactions = TestDHTTransactions(cryptoKind);
          setUpAll(testDHTTransactions.setUpAll);
          tearDownAll(testDHTTransactions.tearDownAll);

          setUp(testDHTTransactions.setUp);
          tearDown(testDHTTransactions.tearDown);

          test('should create empty transaction and drop it explicitly',
              testDHTTransactions.testEmptyTxAndDrop);
          test('should create empty transaction and rollback',
              testDHTTransactions.testEmptyTxAndRollback);
          test('should create empty transaction and commit',
              testDHTTransactions.testEmptyTxAndCommit);
          test('should create transaction, add sets, and rollback',
              testDHTTransactions.testTxAddSetsAndRollback);
          test('should create transaction, add sets, and commit',
              testDHTTransactions.testTxAddSetsAndCommit);
          test('should create transaction, add sets, gets, and commit',
              testDHTTransactions.testTxAddSetsGetsAndCommit);
          test(
              'should create empty transaction, fail non-transactional sets and'
              ' then rollback',
              testDHTTransactions.testTxFailNonTxSetsAndRollback);
          test(
              'should create transaction, inspect, add sets to subkey 1, '
              ' inspect and commit. Then a new transaction inspect, and gets, '
              'and commit',
              testDHTTransactions.testTxInspectAddInspectCommit);
        });

        group('Veilid DHT Transactions Full $cryptoKind', () {
          final testDHTTransactionsFull = TestDHTTransactionsFull(cryptoKind);
          setUpAll(testDHTTransactionsFull.setUpAll);
          tearDownAll(testDHTTransactionsFull.tearDownAll);

          setUp(testDHTTransactionsFull.setUp);
          tearDown(testDHTTransactionsFull.tearDown);

          test('should create empty transaction, inspect, add gets and commit',
              testDHTTransactionsFull.testEmptyTxInspectGetsAndCommit,
              timeout: const Timeout(Duration(seconds: 120)));
          test(
              'should create transaction, fill all records, commit, and then'
              ' get all records and rollback',
              testDHTTransactionsFull.testTxFillCommitTxGetRollback,
              timeout: const Timeout(Duration(seconds: 300)));
        });
      }
    });
  });
}
