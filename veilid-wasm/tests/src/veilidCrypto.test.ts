import { expect } from '@wdio/globals';

import {
  DEBUGGING,
  veilidCoreInitConfig,
  veilidCoreStartupConfig,
} from './utils/veilid-config';

import { veilidClient, veilidCrypto } from 'veilid-wasm';
import { textEncoder, unmarshallBytes } from './utils/marshalling-utils';

describe('veilidCrypto', () => {
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

  it('should list crypto kinds', async () => {
    const kinds = veilidCrypto.VALID_CRYPTO_KINDS;
    await expect(kinds.length).toBeGreaterThan(0)
  });

  for (const cryptoKind of veilidCrypto.VALID_CRYPTO_KINDS) {
    it(`should generate key pair for ${cryptoKind}`, async () => {
      const vcrypto = veilidClient.getCrypto(cryptoKind)
      const keypair = vcrypto.generateKeyPair();
      await expect(typeof keypair).toBe('object');

      const keyPairKind = keypair.kind;
      const barePublicKey = keypair.value.key
      const bareSecretKey = keypair.value.secret
      await expect(keyPairKind).toEqual(cryptoKind);
      await expect(unmarshallBytes(barePublicKey.toString()).length).toBe(vcrypto.publicKeyLength());
      await expect(unmarshallBytes(bareSecretKey.toString()).length).toBe(vcrypto.secretKeyLength());

      const isValid = vcrypto.validateKeyPair(
        keypair.key,
        keypair.secret,
      );
      await expect(isValid).toBe(true);

    });
  }

  for (const cryptoKind of veilidCrypto.VALID_CRYPTO_KINDS) {
    it(`should generate random bytes for ${cryptoKind}`, async () => {
      const vcrypto = veilidClient.getCrypto(cryptoKind)
      const bytes = vcrypto.randomBytes(64);
      await expect(bytes instanceof Uint8Array).toBe(true);
      await expect(bytes.length).toBe(64);

    });
  }

  for (const cryptoKind of veilidCrypto.VALID_CRYPTO_KINDS) {
    it(`should hash data and validate hash for ${cryptoKind}`, async () => {
      const vcrypto = veilidClient.getCrypto(cryptoKind)
      const data = textEncoder.encode('this is my data🚀');
      const hash = vcrypto.generateHash(data);

      await expect(hash).toBeDefined();
      await expect(typeof hash).toBe('object');

      const isValid = vcrypto.validateHash(data, hash);
      await expect(isValid).toBe(true);
    });
  }

  for (const cryptoKind of veilidCrypto.VALID_CRYPTO_KINDS) {
    it(`should hash and validate password for ${cryptoKind}`, async () => {
      const vcrypto = veilidClient.getCrypto(cryptoKind)

      const password = textEncoder.encode('this is my data🚀');
      const saltLength = vcrypto.defaultSaltLength();
      await expect(saltLength).toBeGreaterThan(0);

      const salt = vcrypto.randomBytes(saltLength);
      await expect(salt instanceof Uint8Array).toBe(true);
      await expect(salt.length).toBe(saltLength);

      const hash = vcrypto.hashPassword(password, salt);
      await expect(hash).toBeDefined();
      await expect(typeof hash).toBe('string');

      const isValid = vcrypto.verifyPassword(password, hash);
      await expect(isValid).toBe(true);
    });
  }

  for (const cryptoKind of veilidCrypto.VALID_CRYPTO_KINDS) {
    it(`should aead encrypt and decrypt for ${cryptoKind}`, async () => {
      const vcrypto = veilidClient.getCrypto(cryptoKind)
      const body = textEncoder.encode(
        'This is an encoded body with my secret data in it🔥'
      );
      const ad = textEncoder.encode(
        'This is data associated with my secret data👋'
      );

      const nonce = vcrypto.randomNonce();
      await expect(typeof nonce).toBe('object');

      const sharedSecred = vcrypto.randomSharedSecret();
      await expect(typeof sharedSecred).toBe('object');

      const encBody = vcrypto.encryptAead(
        body,
        nonce,
        sharedSecred,
        ad
      );
      await expect(encBody instanceof Uint8Array).toBe(true);

      const overhead = vcrypto.aeadOverhead();
      await expect(encBody.length - body.length).toBe(overhead);

      const decBody = vcrypto.decryptAead(
        encBody,
        nonce,
        sharedSecred,
        ad
      );
      await expect(decBody instanceof Uint8Array).toBe(true);
      await expect(body).toEqual(decBody);
    });
  }
  for (const cryptoKind of veilidCrypto.VALID_CRYPTO_KINDS) {
    it(`should sign and verify for ${cryptoKind}`, async () => {
      const vcrypto = veilidClient.getCrypto(cryptoKind)
      const keypair = vcrypto.generateKeyPair();
      const data = textEncoder.encode(
        'This is some data I am signing with my key 🔑'
      );
      await expect(typeof keypair).toBe('object');

      const publicKey = keypair.key;
      const secretKey = keypair.secret;

      const sig = vcrypto.sign(publicKey, secretKey, data);
      await expect(typeof sig).toBe('object');

      await expect(async () => {
        const res = vcrypto.verify(publicKey, data, sig);
        await expect(res).toBe(true);
      }).not.toThrow();
    });
  }

});
