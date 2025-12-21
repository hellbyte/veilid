import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:veilid/veilid.dart';

Future<void> testListCryptoSystems() async {
  final cryptoKinds = Veilid.instance.validCryptoKinds();
  expect(cryptoKinds, isNotEmpty);
}

Future<void> testGetCryptoSystems() async {
  final cs = await Veilid.instance.getCryptoSystem(cryptoKindVLD0);
  expect(await cs.defaultSaltLength(), equals(16));
}

final invalidCryptoKind = CryptoKind.fromInt(cryptoKindNONE.toInt() + 1);

Future<void> testGetCryptoSystemInvalid() async {
  await expectLater(() => Veilid.instance.getCryptoSystem(invalidCryptoKind),
      throwsA(isA<VeilidAPIException>()));
}

Future<void> testHashAndVerifyPassword() async {
  for (final kind in Veilid.instance.validCryptoKinds()) {
    final cs = await Veilid.instance.getCryptoSystem(kind);
    final nonce = await cs.randomNonce();
    final salt = nonce.toBytes();

    // Password match
    final phash = await cs.hashPassword(utf8.encode('abc123'), salt);
    expect(await cs.verifyPassword(utf8.encode('abc123'), phash), isTrue);

    // Password mismatch
    await cs.hashPassword(utf8.encode('abc1234'), salt);
    expect(await cs.verifyPassword(utf8.encode('abc1235'), phash), isFalse);
  }
}

Future<void> testSignAndVerifySignature() async {
  for (final kind in Veilid.instance.validCryptoKinds()) {
    final cs = await Veilid.instance.getCryptoSystem(kind);
    final kp1 = await cs.generateKeyPair();
    final kp2 = await cs.generateKeyPair();

    // Signature match
    final sig = await cs.sign(kp1.key, kp1.secret, utf8.encode('abc123'));
    expect(await cs.verify(kp1.key, utf8.encode('abc123'), sig), isTrue);

    // Signature mismatch
    final sig2 = await cs.sign(kp1.key, kp1.secret, utf8.encode('abc1234'));
    expect(await cs.verify(kp1.key, utf8.encode('abc1234'), sig2), isTrue);
    expect(await cs.verify(kp1.key, utf8.encode('abc12345'), sig2), isFalse);
    expect(await cs.verify(kp2.key, utf8.encode('abc1234'), sig2), isFalse);
  }
}

Future<void> testSignAndVerifySignatures() async {
  final kps = <KeyPair>[];
  for (final kind in Veilid.instance.validCryptoKinds()) {
    final cs = await Veilid.instance.getCryptoSystem(kind);
    final kp = await cs.generateKeyPair();
    kps.add(kp);
  }

  // Signature match
  final sigs =
      await Veilid.instance.generateSignatures(utf8.encode('abc123'), kps);
  final pks = kps.map((kp) => kp.key).toList();
  expect(
      await Veilid.instance.verifySignatures(pks, utf8.encode('abc123'), sigs),
      equals(pks));
  // Signature mismatch
  expect(
      await Veilid.instance.verifySignatures(pks, utf8.encode('abc1234'), sigs),
      isNull);
}

Future<void> testGenerateSharedSecret() async {
  for (final kind in Veilid.instance.validCryptoKinds()) {
    final cs = await Veilid.instance.getCryptoSystem(kind);

    final kp1 = await cs.generateKeyPair();
    final kp2 = await cs.generateKeyPair();
    final kp3 = await cs.generateKeyPair();

    final ssA = await cs.generateSharedSecret(
        kp1.key, kp2.secret, utf8.encode('abc123'));
    final ssB = await cs.generateSharedSecret(
        kp2.key, kp1.secret, utf8.encode('abc123'));

    expect(ssA, equals(ssB));

    final ssC = await cs.generateSharedSecret(
        kp2.key, kp1.secret, utf8.encode('abc1234'));

    expect(ssA, isNot(equals(ssC)));

    final ssD = await cs.generateSharedSecret(
        kp3.key, kp1.secret, utf8.encode('abc123'));

    expect(ssA, isNot(equals(ssD)));
  }
}
