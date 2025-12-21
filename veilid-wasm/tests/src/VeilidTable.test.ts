import { expect } from '@wdio/globals';

import {
  DEBUGGING,
  veilidCoreInitConfig,
  veilidCoreStartupConfig,
} from './utils/veilid-config';

import { VeilidTableDB, veilidClient } from 'veilid-wasm';
import { textEncoder, textDecoder } from './utils/marshalling-utils';

const TABLE_NAME = 'some-table';
const TABLE_COLS = 1;

describe('VeilidTable', () => {
  before('veilid startup', async () => {
    await veilidClient.initializeCore(veilidCoreInitConfig);
    await veilidClient.startupCore((_update) => {
      if (DEBUGGING) {
        if (_update.kind === 'Log') {
          console.log(_update.message);
        }
      }
    }, veilidCoreStartupConfig);
  });

  after('veilid shutdown', async () => {
    await veilidClient.shutdownCore();
  });

  it('should open and close a table', async () => {
    const table = new VeilidTableDB(TABLE_NAME, TABLE_COLS);
    await table.openTable();

    const keys = await table.getKeys(0);
    await expect(keys.length).toBe(0);
  });

  describe('table operations', () => {
    let table: VeilidTableDB;

    before('create table', async () => {
      table = new VeilidTableDB(TABLE_NAME, TABLE_COLS);
      await table.openTable();
    });

    it('should have no keys', async () => {
      const keys = await table.getKeys(0);
      await expect(keys.length).toBe(0);
    });

    describe('store/load', () => {
      const key = 'test-key with unicode 🚀';
      const value = 'test value with unicode 🚀';

      it('should store value', async () => {
        await table.store(
          0,
          textEncoder.encode(key),
          textEncoder.encode(value)
        );
      });

      it('should load value', async () => {
        const storedValue = await table.load(0, textEncoder.encode(key));
        await expect(storedValue).toBeDefined();
        await expect(textDecoder.decode(storedValue)).toBe(value);
      });

      it('should have key in list of keys', async () => {
        const keys = await table.getKeys(0);
        const decodedKeys = keys.map((key) => textDecoder.decode(key));
        await expect(decodedKeys).toEqual([key]);
      });
    });

    describe('transactions', () => {
      it('should commit a transaction', async () => {
        const transaction = await table.createTransaction();

        const key = 'tranaction-key🔥';
        const first = 'first🅱';
        const second = 'second✔';
        const third = 'third📢';

        await transaction.store(
          0,
          textEncoder.encode(key),
          textEncoder.encode(first)
        );
        await transaction.store(
          0,
          textEncoder.encode(key),
          textEncoder.encode(second)
        );
        await transaction.store(
          0,
          textEncoder.encode(key),
          textEncoder.encode(third)
        );

        await transaction.commit();

        const storedValue = await table.load(0, textEncoder.encode(key));
        await expect(storedValue).toBeDefined();
        await expect(textDecoder.decode(storedValue)).toBe(third);
      });
    });
  });
});
