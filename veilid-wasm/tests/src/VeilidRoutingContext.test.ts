import { expect } from '@wdio/globals';

import {
  DEBUGGING,
  veilidCoreInitConfig,
  veilidCoreStartupConfig,
} from './utils/veilid-config';

import {
  DHTRecordDescriptor,
  DHTRecordReport,
  KeyPair,
  RecordKey,
  ValueData,
  VeilidDHTTransaction,
  VeilidRoutingContext,
  veilidClient,
  veilidCrypto
} from 'veilid-wasm';
import { textEncoder, textDecoder } from './utils/marshalling-utils';
import { asyncCallWithTimeout, waitForPublicAttachment, waitForOfflineSubkeyWrite } from './utils/wait-utils';

describe('VeilidRoutingContext', () => {
  before('veilid startup', async () => {
    await veilidClient.initializeCore(veilidCoreInitConfig);
    await veilidClient.startupCore((_update) => {
      if (DEBUGGING) {
        if (_update.kind === 'Log') {
          console.log(_update.message);
        }
      }
    }, veilidCoreStartupConfig);
    await veilidClient.attach();
    await asyncCallWithTimeout(waitForPublicAttachment(), 30_000);
    //console.log("---Started Up---");
  });

  after('veilid shutdown', async () => {
    //console.log("---Shutting Down---");
    await veilidClient.detach();
    await veilidClient.shutdownCore();
  });

  describe('constructors', () => {
    it('should create using .create()', async () => {
      const routingContext = VeilidRoutingContext.create();
      await expect(routingContext instanceof VeilidRoutingContext).toBe(true);
    });

    it('should create using new', () => {
      const routingContext = new VeilidRoutingContext();
      void expect(routingContext instanceof VeilidRoutingContext).toBe(true);
    });

    it('should create with default safety', async () => {
      const routingContext = VeilidRoutingContext.create().withDefaultSafety();
      await expect(routingContext instanceof VeilidRoutingContext).toBe(true);
    });

    it('should create with safety', async () => {
      const routingContext = VeilidRoutingContext.create().withSafety({
        Safe: {
          hopCount: 2,
          sequencing: 'EnsureOrdered',
          stability: 'Reliable',
        },
      });
      await expect(routingContext instanceof VeilidRoutingContext).toBe(true);
    });

    it('should create with sequencing', async () => {
      const routingContext =
        VeilidRoutingContext.create().withSequencing('EnsureOrdered');
      await expect(routingContext instanceof VeilidRoutingContext).toBe(true);
    });

    it('should error if unsafe is used', async () => {
      await expect(() => {
        VeilidRoutingContext.create().withSafety({
          Unsafe: 'EnsureOrdered',
        });
      }).toThrow();
    });
  });

  describe('operations', () => {
    let routingContext: VeilidRoutingContext;

    before('create routing context', () => {
      routingContext = VeilidRoutingContext.create();
    });

    for (const cryptoKind of veilidCrypto.VALID_CRYPTO_KINDS) {

      describe(`createDhtRecord for ${cryptoKind}`, () => {
        it('should create dht record with default schema', async () => {
          const dhtRecord = await routingContext.createDHTRecord(cryptoKind, { 'DFLT': { oCnt: 1 } });
          await expect(dhtRecord.key).toBeInstanceOf(RecordKey);
          await expect(dhtRecord.owner).toBeInstanceOf(KeyPair);
          await expect(dhtRecord.schema).toEqual({ 'DFLT': { oCnt: 1 } });
        });

        it('should create dht record with default schema, no owner for', async () => {
          const dhtRecord = await routingContext.createDHTRecord(cryptoKind, { 'DFLT': { oCnt: 1 } });
          await expect(dhtRecord.key).toBeInstanceOf(RecordKey);
          await expect(dhtRecord.owner).toBeInstanceOf(KeyPair);
          await expect(dhtRecord.schema).toEqual({ 'DFLT': { oCnt: 1 } });
        });

        it('should create dht record with default schema, with owner, and a deterministic key', async () => {
          const vcrypto = veilidClient.getCrypto(cryptoKind)
          const ownerKeyPair = vcrypto.generateKeyPair();
          const owner = ownerKeyPair.key
          const dhtRecord = await routingContext.createDHTRecord(cryptoKind, { 'DFLT': { oCnt: 1 } }, ownerKeyPair);
          const dhtRecordKey = await veilidClient.getDHTRecordKey({ 'DFLT': { oCnt: 1 } }, owner, dhtRecord.key.encryptionKey);
          await expect(dhtRecord.key).toBeInstanceOf(RecordKey);
          await expect(dhtRecord.key.isEqual(dhtRecordKey)).toEqual(true);
          await expect(dhtRecord.owner).toBeInstanceOf(KeyPair);
          await expect((dhtRecord.owner as KeyPair).isEqual(ownerKeyPair)).toEqual(true);
          await expect(dhtRecord.schema).toEqual({ 'DFLT': { oCnt: 1 } });
        });
      });

      describe(`DHT kitchen sink for ${cryptoKind}`, () => {
        let dhtRecord: DHTRecordDescriptor;
        const data = '🚀 This example DHT data with unicode a Ā 𐀀 文 🚀';

        beforeEach('create dht record', async () => {
          dhtRecord = await routingContext.createDHTRecord(
            cryptoKind,
            { 'DFLT': { oCnt: 1 } },
          );

          await expect(dhtRecord.key).toBeInstanceOf(RecordKey);
          await expect(dhtRecord.owner).toBeInstanceOf(KeyPair);
          await expect(dhtRecord.schema).toEqual({ 'DFLT': { oCnt: 1 } });
        });

        afterEach('free dht record', async () => {
          await routingContext.deleteDHTRecord(dhtRecord.key);
        });

        it('should set value', async () => {
          const setValueRes = await routingContext.setDHTValue(
            dhtRecord.key,
            0,
            textEncoder.encode(data)
          );
          await expect(setValueRes).toBeUndefined();
        });

        it('should get value with force refresh', async () => {

          const setValueRes = await routingContext.setDHTValue(
            dhtRecord.key,
            0,
            textEncoder.encode(data)
          );
          await expect(setValueRes).toBeUndefined();

          // Wait for synchronization
          await waitForOfflineSubkeyWrite(routingContext, dhtRecord.key);

          const getValueRes = await routingContext.getDHTValue(
            dhtRecord.key,
            0,
            true
          );

          await expect(getValueRes?.data).toBeDefined();
          await expect(textDecoder.decode(getValueRes?.data)).toBe(data);

          await expect(getValueRes?.writer.isEqual((dhtRecord.owner as KeyPair).key)).toEqual(true);
          await expect(getValueRes?.seq).toBe(0);
        });

        it('should open readonly record', async () => {
          await routingContext.closeDHTRecord(dhtRecord.key);

          const readonlyDhtRecord = await routingContext.openDHTRecord(
            dhtRecord.key
          );
          await expect(readonlyDhtRecord).toBeDefined();

          const setValueRes = routingContext.setDHTValue(
            dhtRecord.key,
            0,
            textEncoder.encode(data)
          );
          await expect(setValueRes).rejects.toEqual({
            kind: 'Generic',
            message: 'value is not writable',
          });
        });

        it('should open writable record', async () => {
          await routingContext.closeDHTRecord(dhtRecord.key);

          const writeableDhtRecord = await routingContext.openDHTRecord(
            dhtRecord.key,
            dhtRecord.owner as KeyPair,
          );
          await expect(writeableDhtRecord).toBeDefined();
          const setValueRes = await routingContext.setDHTValue(
            dhtRecord.key,
            0,
            textEncoder.encode(`${data}👋`)
          );
          await expect(setValueRes).toBeUndefined();
        });

        it('should open readonly record and specify writer during the set', async () => {
          await routingContext.closeDHTRecord(dhtRecord.key);

          const writeableDhtRecord = await routingContext.openDHTRecord(
            dhtRecord.key,
          );
          await expect(writeableDhtRecord).toBeDefined();
          const setValueResFail = routingContext.setDHTValue(
            dhtRecord.key,
            0,
            textEncoder.encode(`${data}👋`),
          );
          await expect(setValueResFail).rejects.toEqual({
            kind: 'Generic',
            message: 'value is not writable',
          });
          const setValueRes = await routingContext.setDHTValue(
            dhtRecord.key,
            0,
            textEncoder.encode(`${data}👋`),
            {
              writer: dhtRecord.owner as KeyPair,
              allowOffline: undefined
            }
          );
          await expect(setValueRes).toBeUndefined();
        });

        it('should watch value and cancel watch', async () => {
          const setValueRes = await routingContext.setDHTValue(
            dhtRecord.key,
            0,
            textEncoder.encode(data)
          );
          await expect(setValueRes).toBeUndefined();

          // With typical values
          const watchValueRes = await routingContext.watchDhtValues(
            dhtRecord.key,
            [[0, 0]],
            "0",
            0xFFFFFFFF,
          );
          await expect(watchValueRes).toEqual(true);

          const cancelValueRes = await routingContext.cancelDHTWatch(
            dhtRecord.key,
            [],
          )

          await expect(cancelValueRes).toEqual(false);

        });

        it('should watch value and cancel watch with default values', async () => {
          const setValueRes = await routingContext.setDHTValue(
            dhtRecord.key,
            0,
            textEncoder.encode(data)
          );
          await expect(setValueRes).toBeUndefined();

          // Again with default values
          const watchValueRes = await routingContext.watchDhtValues(
            dhtRecord.key,
          );
          await expect(watchValueRes).toEqual(true);

          const cancelValueRes = await routingContext.cancelDHTWatch(
            dhtRecord.key,
          )
          await expect(cancelValueRes).toEqual(false);
        });

        it('should set a value and inspect it', async () => {
          const setValueRes = await routingContext.setDHTValue(
            dhtRecord.key,
            0,
            textEncoder.encode(data)
          );
          await expect(setValueRes).toBeUndefined();

          // Inspect locally
          const inspectRes = await routingContext.inspectDHTRecord(
            dhtRecord.key,
            [[0, 0]],
            "Local",
          );
          await expect(inspectRes).toBeDefined();
          await expect(inspectRes.subkeys).toEqual([[0, 0]]);
          await expect(inspectRes.localSeqs).toEqual([0]);
          await expect(inspectRes.networkSeqs).toEqual([undefined]);

          // Wait for synchronization
          await waitForOfflineSubkeyWrite(routingContext, dhtRecord.key);

          // Inspect network
          const inspectRes2 = await routingContext.inspectDHTRecord(
            dhtRecord.key,
            [[0, 0]],
            "SyncGet",
          );
          await expect(inspectRes2).toBeDefined();
          await expect(inspectRes2.subkeys).toEqual([[0, 0]]);
          await expect(inspectRes2.offlineSubkeys).toEqual([]);
          await expect(inspectRes2.localSeqs).toEqual([0]);
          await expect(inspectRes2.networkSeqs).toEqual([0]);
        });

        it('should set a value and inspect it with defaults', async () => {
          const setValueRes = await routingContext.setDHTValue(
            dhtRecord.key,
            0,
            textEncoder.encode(data)
          );
          await expect(setValueRes).toBeUndefined();

          // Wait for synchronization
          await waitForOfflineSubkeyWrite(routingContext, dhtRecord.key);

          // Inspect locally
          const inspectRes = await routingContext.inspectDHTRecord(
            dhtRecord.key,
          );
          await expect(inspectRes).toBeDefined();
          await expect(inspectRes.offlineSubkeys).toEqual([]);
          await expect(inspectRes.localSeqs).toEqual([0]);
          await expect(inspectRes.networkSeqs).toEqual([undefined]);
        });
      });

      describe(`DHT transactions simple tests for ${cryptoKind}`, () => {
        const DHT_RECORD_COUNT = 4
        const DHT_SUBKEY_SIZE = 16
        const DHT_SUBKEY_COUNT = 2
        const data: Uint8Array[] = [];
        for (let rec = 0; rec < DHT_RECORD_COUNT; rec++) {
          data.push(new Uint8Array(DHT_SUBKEY_SIZE).fill(rec));
        }

        let dhtRecords: RecordKey[];

        beforeEach('create dht records', async () => {
          dhtRecords = [];
          for (let rec = 0; rec < DHT_RECORD_COUNT; rec++) {
            const dhtRecord = await routingContext.createDHTRecord(
              cryptoKind,
              { 'DFLT': { oCnt: DHT_SUBKEY_COUNT } },
            );
            await expect(dhtRecord.key).toBeInstanceOf(RecordKey);
            await expect(dhtRecord.owner).toBeInstanceOf(KeyPair);
            await expect(dhtRecord.schema).toEqual({ 'DFLT': { oCnt: DHT_SUBKEY_COUNT } });
            dhtRecords.push(dhtRecord.key);
          }
        });

        afterEach('free dht records', async () => {
          for (let rec = 0; rec < DHT_RECORD_COUNT; rec++) {
            await routingContext.deleteDHTRecord(dhtRecords[rec]);
          }
          dhtRecords = [];
        });

        it('should create empty transaction and drop it explicitly', async () => {
          const tx = await veilidClient.transactDHTRecords(dhtRecords);
          await expect(tx).toBeInstanceOf(VeilidDHTTransaction);
          tx.free();
        });

        it('should create empty transaction and drop it lazily', async () => {
          const tx = await veilidClient.transactDHTRecords(dhtRecords);
          await expect(tx).toBeInstanceOf(VeilidDHTTransaction);
        });

        it('should create empty transaction and rollback', async () => {
          const tx = await veilidClient.transactDHTRecords(dhtRecords);
          await expect(tx).toBeInstanceOf(VeilidDHTTransaction);
          await tx.rollback();
        });

        it('should create empty transaction and commit', async () => {
          const tx = await veilidClient.transactDHTRecords(dhtRecords);
          await expect(tx).toBeInstanceOf(VeilidDHTTransaction);
          await tx.commit();
        });

        it('should create transaction, add sets, and rollback', async () => {
          const tx = await veilidClient.transactDHTRecords(dhtRecords);
          await expect(tx).toBeInstanceOf(VeilidDHTTransaction);

          for (let rec = 0; rec < DHT_RECORD_COUNT; rec++) {
            const res = await tx.set(dhtRecords[rec], 0, data[rec]);
            await expect(res).toBeUndefined();
          }

          await tx.rollback();
        });

        it('should create transaction, add sets, and commit', async () => {
          const tx = await veilidClient.transactDHTRecords(dhtRecords);
          await expect(tx).toBeInstanceOf(VeilidDHTTransaction);

          for (let rec = 0; rec < DHT_RECORD_COUNT; rec++) {
            const res = await tx.set(dhtRecords[rec], 0, data[rec]);
            await expect(res).toBeUndefined();
          }

          await tx.commit();
        });

        it('should create transaction, add sets, gets, and commit', async () => {
          const tx = await veilidClient.transactDHTRecords(dhtRecords);
          await expect(tx).toBeInstanceOf(VeilidDHTTransaction);

          const sets = [];
          for (let rec = 0; rec < DHT_RECORD_COUNT; rec++) {
            sets.push(tx.set(dhtRecords[rec], 0, data[rec]));
          }

          const allSetRes = await Promise.all(sets)
          for (const res of allSetRes) {
            await expect(res).toBeUndefined();
          }

          const gets = [];
          for (let rec = 0; rec < DHT_RECORD_COUNT; rec++) {
            gets.push(tx.get(dhtRecords[rec], 0));
          }

          const allGetRes = await Promise.all(gets)
          for (const res of allGetRes) {
            await expect(res).toBeUndefined();
          }

          await tx.commit();
        });

        it('should create empty transaction, fail non-transactional sets and then rollback', async () => {
          const tx = await veilidClient.transactDHTRecords(dhtRecords);
          await expect(tx).toBeInstanceOf(VeilidDHTTransaction);

          const sets: Promise<ValueData | undefined>[] = [];
          for (let subkey = 0; subkey < DHT_SUBKEY_COUNT; subkey++) {
            for (let rec = 0; rec < DHT_RECORD_COUNT; rec++) {
              sets.push(routingContext.setDHTValue(dhtRecords[rec], subkey, data[rec]));
            }
          }

          try {
            await Promise.all(sets);
          } catch (error) {
            await expect(error).toMatchObject({ kind: "TryAgain" });
          }
          await tx.rollback();

        });

        it('should create transaction, inspect, add sets to subkey 1, inspect, and commit. Then a new transaction inspect, and gets, and commit', async () => {
          // Begin
          const tx = await veilidClient.transactDHTRecords(dhtRecords);
          await expect(tx).toBeInstanceOf(VeilidDHTTransaction);

          // Inspect 1
          const inspects1: Promise<{ rec: number, report: DHTRecordReport }>[] = [];
          for (let rec = 0; rec < DHT_RECORD_COUNT; rec++) {
            inspects1.push((async () => {
              const report = await tx.inspect(dhtRecords[rec], null, "SyncGet");
              return { rec: rec, report: report };
            })());
          }
          const allInspects1Res = await Promise.all(inspects1)
          for (const res of allInspects1Res) {
            //console.log("res1", res);
            await expect(res.report.subkeys).toEqual([[0, 1]]);
            await expect(res.report.localSeqs).toEqual([undefined, undefined]);
            await expect(res.report.networkSeqs).toEqual([undefined, undefined]);
            await expect(res.report.offlineSubkeys).toEqual([]);
          }

          // Sets
          const sets: Promise<ValueData | undefined>[] = [];
          for (let rec = 0; rec < DHT_RECORD_COUNT; rec++) {
            sets.push(tx.set(dhtRecords[rec], 1, data[rec]));
          }

          const allSetRes = await Promise.all(sets)
          for (const res of allSetRes) {
            await expect(res).toBeUndefined();
          }

          // Inspect 2 (should be the same because we haven't committed yet)
          const inspects2: Promise<{ rec: number, report: DHTRecordReport }>[] = [];
          for (let rec = 0; rec < DHT_RECORD_COUNT; rec++) {
            inspects2.push((async () => {
              const report = await tx.inspect(dhtRecords[rec], null, "SyncGet");
              return { rec: rec, report: report };
            })());
          }
          const allInspects2Res = await Promise.all(inspects2)
          for (const res of allInspects2Res) {
            //console.log("res2", res);
            await expect(res.report.subkeys).toEqual([[0, 1]]);
            await expect(res.report.localSeqs).toEqual([undefined, undefined]);
            await expect(res.report.networkSeqs).toEqual([undefined, undefined]);
            await expect(res.report.offlineSubkeys).toEqual([]);
          }

          // Commit
          await tx.commit();

          // Transaction 2
          const tx2 = await veilidClient.transactDHTRecords(dhtRecords);
          await expect(tx2).toBeInstanceOf(VeilidDHTTransaction);

          // Inspect 3 (should be updated post-commit)
          const inspects3: Promise<{ rec: number, report: DHTRecordReport }>[] = [];
          for (let rec = 0; rec < DHT_RECORD_COUNT; rec++) {
            inspects3.push((async () => {
              const report = await tx2.inspect(dhtRecords[rec], null, "SyncGet");
              return { rec: rec, report: report };
            })());
          }
          const allInspects3Res = await Promise.all(inspects3)
          for (const res of allInspects3Res) {
            //console.log("res3", res);
            await expect(res.report.subkeys).toEqual([[0, 1]]);
            await expect(res.report.localSeqs).toEqual([undefined, 0]);
            await expect(res.report.networkSeqs).toEqual([undefined, 0]);
            await expect(res.report.offlineSubkeys).toEqual([]);
          }

          // Gets should match inspect
          const expected = [undefined, 0];
          for (let subkey = 0; subkey < DHT_SUBKEY_COUNT; subkey++) {
            const gets: Promise<{ rec: number, subkey: number, val: ValueData | undefined }>[] = [];
            for (let rec = 0; rec < DHT_RECORD_COUNT; rec++) {
              gets.push((async () => {
                const val = await tx2.get(dhtRecords[rec], subkey);
                return { rec: rec, subkey: subkey, val: val };
              })());
            }

            const allGetRes = await Promise.all(gets)
            for (const res of allGetRes) {
              if (expected[res.subkey] == null) {
                await expect(res.val).toBeUndefined();
              } else {
                await expect(res.val).toBeDefined();
                await expect(res.val!.data).toEqual(data[res.rec]);
                await expect(res.val!.seq).toEqual(expected[res.subkey]);
              }
            }
          }
          await tx2.commit();
        });
      });

      describe(`DHT transactions full records tests for ${cryptoKind}`, () => {
        const DHT_RECORD_COUNT = 8
        const DHT_SUBKEY_SIZE = 32768
        const DHT_SUBKEY_COUNT = 32
        const data: Uint8Array[] = [];
        for (let rec = 0; rec < DHT_RECORD_COUNT; rec++) {
          data.push(new Uint8Array(DHT_SUBKEY_SIZE).fill(rec));
        }

        let dhtRecords: RecordKey[];

        beforeEach('create dht records', async () => {
          dhtRecords = [];
          for (let rec = 0; rec < DHT_RECORD_COUNT; rec++) {
            const dhtRecord = await routingContext.createDHTRecord(
              cryptoKind,
              { 'DFLT': { oCnt: DHT_SUBKEY_COUNT } },
            );
            await expect(dhtRecord.key).toBeInstanceOf(RecordKey);
            await expect(dhtRecord.owner).toBeInstanceOf(KeyPair);
            await expect(dhtRecord.schema).toEqual({ 'DFLT': { oCnt: DHT_SUBKEY_COUNT } });
            dhtRecords.push(dhtRecord.key);
          }
        });

        afterEach('free dht records', async () => {
          for (let rec = 0; rec < DHT_RECORD_COUNT; rec++) {
            await routingContext.deleteDHTRecord(dhtRecords[rec]);
          }
          dhtRecords = [];
        });


        it('should create empty transaction, inspect, add gets and commit', async () => {
          const tx = await veilidClient.transactDHTRecords(dhtRecords);
          await expect(tx).toBeInstanceOf(VeilidDHTTransaction);

          // Inspect 1
          const inspects1: Promise<{ rec: number, report: DHTRecordReport }>[] = [];
          for (let rec = 0; rec < DHT_RECORD_COUNT; rec++) {
            inspects1.push((async () => {
              const report = await tx.inspect(dhtRecords[rec], null, "SyncGet");
              return { rec: rec, report: report };
            })());
          }
          const allInspects1Res = await Promise.all(inspects1)
          const expectedSeqs = Array(DHT_SUBKEY_COUNT).fill(undefined);
          for (const res of allInspects1Res) {
            await expect(res.report.subkeys).toEqual([[0, DHT_SUBKEY_COUNT - 1]]);
            await expect(res.report.localSeqs).toEqual(expectedSeqs);
            await expect(res.report.networkSeqs).toEqual(expectedSeqs);
            await expect(res.report.offlineSubkeys).toEqual([]);
          }

          for (let subkey = 0; subkey < DHT_SUBKEY_COUNT; subkey++) {
            const gets: Promise<ValueData | undefined>[] = [];
            for (let rec = 0; rec < DHT_RECORD_COUNT; rec++) {
              gets.push(tx.get(dhtRecords[rec], subkey));
            }
            const allGetRes = await Promise.all(gets)
            for (const res of allGetRes) {
              await expect(res).toBeUndefined();
            }
          }

          await tx.commit();
        });


        it('should create transaction, fill all records, commit, and then get all records and rollback', async () => {
          const startBegin = performance.now();
          const tx = await veilidClient.transactDHTRecords(dhtRecords);
          await expect(tx).toBeInstanceOf(VeilidDHTTransaction);
          console.log(`begin transaction: ${performance.now() - startBegin}ms`)

          // Sets
          for (let subkey = 0; subkey < DHT_SUBKEY_COUNT; subkey++) {
            const startSet = performance.now();

            const sets: Promise<ValueData | undefined>[] = [];
            for (let rec = 0; rec < DHT_RECORD_COUNT; rec++) {
              sets.push(tx.set(dhtRecords[rec], subkey, data[rec]));
            }

            const allSetRes = await Promise.all(sets)
            for (const res of allSetRes) {
              await expect(res).toBeUndefined();
            }

            console.log(`set subkey ${subkey}: ${performance.now() - startSet}ms`)
          }

          const startCommit = performance.now();
          await tx.commit();
          console.log(`commit transaction: ${performance.now() - startCommit}ms`)

          const startBegin2 = performance.now();
          const tx2 = await veilidClient.transactDHTRecords(dhtRecords);
          await expect(tx2).toBeInstanceOf(VeilidDHTTransaction);
          console.log(`begin transaction 2: ${performance.now() - startBegin2}ms`)

          // Gets
          for (let subkey = 0; subkey < DHT_SUBKEY_COUNT; subkey++) {
            const startGet = performance.now();

            const gets: Promise<{ rec: number, subkey: number, val: ValueData | undefined }>[] = [];
            for (let rec = 0; rec < DHT_RECORD_COUNT; rec++) {
              gets.push((async () => {
                const val = await tx2.get(dhtRecords[rec], subkey);
                return { rec: rec, subkey: subkey, val: val };
              })());
            }

            const allGetRes = await Promise.all(gets)
            for (const res of allGetRes) {
              await expect(res.val).toBeDefined();
              await expect(res.val!.data).toEqual(data[res.rec]);
              await expect(res.val!.seq).toEqual(0);
            }

            console.log(`get subkey ${subkey}: ${performance.now() - startGet}ms`)
          }

          const startRollback = performance.now();
          await tx2.rollback();
          console.log(`rollback transaction 2: ${performance.now() - startRollback}ms`)
        });
      });
    }
  });
});
