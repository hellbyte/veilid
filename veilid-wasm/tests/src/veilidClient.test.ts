import { expect } from '@wdio/globals';

import {
  DEBUGGING,
  veilidCoreInitConfig,
  veilidCoreStartupConfig,
} from './utils/veilid-config';

import { VeilidState, veilidClient, veilidCrypto, KeyPair, Signature } from 'veilid-wasm';
import { asyncCallWithTimeout, waitForDetached, waitForPublicAttachment, waitForShutdown } from './utils/wait-utils';
import { textEncoder } from './utils/marshalling-utils';

describe('veilidClient', () => {
  before('veilid startup', async () => {
    // console.log("---Init---");
    await veilidClient.initializeCore(veilidCoreInitConfig);
    // console.log("---Startup---");
    // console.log("config: ", veilidCoreStartupConfig);
    await veilidClient.startupCore((_update) => {
      if (DEBUGGING) {
        if (_update.kind === 'Log') {
          console.log(_update.message);
        }
      }
    }, veilidCoreStartupConfig);
    // console.log("---Started Up---");
  });

  after('veilid shutdown', async () => {
    // console.log("---Shutting Down---");
    await veilidClient.shutdownCore();
    await asyncCallWithTimeout(waitForShutdown(), 10000);
  });

  it('should print version', async () => {
    const version = veilidClient.versionString();
    await expect(typeof version).toBe('string');
    await expect(version.length).toBeGreaterThan(0);
  });

  it('should print features', async () => {
    const features = veilidClient.features();
    await expect(Array.isArray(features)).toBe(true);
    await expect(features.length).toBeGreaterThan(0);
  });

  it('should get config', async () => {
    const defaultConfig = veilidClient.defaultConfig();
    await expect(typeof defaultConfig).toBe('object');

    await expect(defaultConfig).toHaveProperty('programName');
    await expect(defaultConfig).toHaveProperty('namespace');
    await expect(defaultConfig).toHaveProperty('capabilities');
    await expect(defaultConfig).toHaveProperty('protectedStore');
    await expect(defaultConfig).toHaveProperty('tableStore');
    await expect(defaultConfig).toHaveProperty('blockStore');
    await expect(defaultConfig).toHaveProperty('network');
  });

  it('should attach and detach', async () => {
    await veilidClient.attach();
    await asyncCallWithTimeout(waitForPublicAttachment(), 10000);
    await veilidClient.detach();
    await asyncCallWithTimeout(waitForDetached(), 10000);
  });

  describe('kitchen sink', () => {
    before('attach', async () => {
      await veilidClient.attach();
      await asyncCallWithTimeout(waitForPublicAttachment(), 10000);
    });
    after('detach', async () => {
      await veilidClient.detach();
      await asyncCallWithTimeout(waitForDetached(), 10000);
    });

    let state: VeilidState;

    it('should get state', async () => {
      state = await veilidClient.getState();
      await expect(state.attachment).toBeDefined();
      await expect(state.config.config).toBeDefined();
      await expect(state.network).toBeDefined();
    });

    it('should call debug command', async () => {
      const response = await veilidClient.debug('help');
      await expect(response).toBeDefined();
      await expect(response.length).toBeGreaterThan(0);
    });
  });

  describe('global crypto functions', () => {

    it(`should sign and verify for all crypto kinds`, async () => {

      const keypairs: KeyPair[] = []
      for (const cryptoKind of veilidCrypto.VALID_CRYPTO_KINDS) {
        const vcrypto = veilidClient.getCrypto(cryptoKind)
        const keypair = vcrypto.generateKeyPair();
        await expect(typeof keypair).toBe('object');

        keypairs.push(keypair);
      }

      const data = textEncoder.encode(
        'This is some data I am signing with my key 🔑'
      );

      let signatures: Signature[];
      await expect(async () => {
        signatures = veilidClient.generateSignatures(data, keypairs);
        await expect(typeof signatures).toBe('object');
      }).not.toThrow();

      const publicKeys = keypairs.map((kp) => kp.key)

      await expect(async () => {
        const res = veilidClient.verifySignatures(publicKeys, data, signatures);
        await expect(res).not.toBeUndefined();
        await expect(res!.length).toEqual(publicKeys.length);
      }).not.toThrow();

      signatures = []
      await expect(async () => {
        const res = veilidClient.verifySignatures(publicKeys, data, signatures);
        await expect(res).not.toBeUndefined();
        await expect(res!.length).toEqual(0);
      }).not.toThrow();

    });
  })
});
