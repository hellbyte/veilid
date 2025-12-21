import 'dart:async';
import 'dart:convert';
import 'dart:js_interop';
import 'dart:js_interop_unsafe';
import 'dart:typed_data';
import 'package:web/web.dart' as web;
import 'veilid.dart';

//////////////////////////////////////////////////////////

Veilid getVeilid() => VeilidJS();

JSObject wasm = web.window.getProperty<JSObject>('veilid_wasm'.toJS);

Uint8List convertUint8ListFromJson(dynamic json) => Uint8List.fromList(
  ((json as JSArray).dartify()! as List<Object?>)
      .map((e) => e! as int)
      .toList(),
);

dynamic convertUint8ListToJson(Uint8List data) => data.toList().jsify();

//////////////////////////////////////////////////////////////////////////////
// Direct-call binding convenience functions

T _wasmCallJson<T>(
  String method,
  T Function(dynamic) fromJson, [
  List<JSAny?>? arguments,
]) => fromJson(
  jsonDecode(wasm.callMethodVarArgs<JSString>(method.toJS, arguments).toDart),
);

String _wasmCallString(String method, [List<JSAny?>? arguments]) =>
    wasm.callMethodVarArgs<JSString>(method.toJS, arguments).toDart;

int _wasmCallInt(String method, [List<JSAny?>? arguments]) =>
    wasm.callMethodVarArgs<JSNumber>(method.toJS, arguments).toDartInt;

void _wasmCallVoid(String method, [List<JSAny?>? arguments]) =>
    wasm.callMethodVarArgs(method.toJS, arguments);

//////////////////////////////////////////////////////////////////////////////
// Promise binding convenience functions

Future<T> _wrapApiPromise<T extends JSAny?>(JSPromise<JSAny?> p) =>
    p.toDart.then((value) => value as T)
    // Any errors at all from Veilid need to be caught
    // ignore: inference_failure_on_untyped_parameter
    .catchError((e, s) {
      try {
        final ex = VeilidAPIException.fromJson(jsonDecode(e as String));
        throw ex;
      } on Exception catch (e) {
        if (e is VeilidAPIException) {
          rethrow;
        } else {
          // Wrap all other errors in VeilidAPIExceptionInternal
          throw VeilidAPIExceptionInternal('$e\nStack Trace:\n$s');
        }
      }
    });

Future<T> _wasmPromiseJson<T>(
  String method,
  T Function(dynamic) fromJson, [
  List<JSAny?>? arguments,
]) async => fromJson(
  jsonDecode(
    (await _wrapApiPromise<JSString>(
      wasm.callMethodVarArgs(method.toJS, arguments),
    )).toDart,
  ),
);

Future<T?> _wasmPromiseJsonOrNull<T>(
  String method,
  T Function(dynamic) fromJson, [
  List<JSAny?>? arguments,
]) async {
  final res = (await _wrapApiPromise<JSString>(
    wasm.callMethodVarArgs(method.toJS, arguments),
  )).toDart;
  final decodedRes = jsonDecode(res);
  if (decodedRes == null) {
    return null;
  }
  return fromJson(decodedRes);
}

Future<Uint8List> _wasmPromiseUint8List(
  String method, [
  List<JSAny?>? arguments,
]) async => base64UrlNoPadDecode(
  (await _wrapApiPromise<JSString>(
    wasm.callMethodVarArgs(method.toJS, arguments),
  )).toDart,
);

Future<Uint8List?> _wasmPromiseUint8ListOrNull(
  String method, [
  List<JSAny?>? arguments,
]) async {
  final res = await _wrapApiPromise<JSAny?>(
    wasm.callMethodVarArgs(method.toJS, arguments),
  );
  if (res == null || res.isUndefinedOrNull) {
    return null;
  }
  return base64UrlNoPadDecode((res as JSString).toDart);
}

Future<String> _wasmPromiseString(
  String method, [
  List<JSAny?>? arguments,
]) async => (await _wrapApiPromise<JSString>(
  wasm.callMethodVarArgs(method.toJS, arguments),
)).toDart;

Future<int> _wasmPromiseInt(String method, [List<JSAny?>? arguments]) async =>
    (await _wrapApiPromise<JSNumber>(
      wasm.callMethodVarArgs(method.toJS, arguments),
    )).toDartInt;

Future<bool> _wasmPromiseBool(String method, [List<JSAny?>? arguments]) async =>
    (await _wrapApiPromise<JSBoolean>(
      wasm.callMethodVarArgs(method.toJS, arguments),
    )).toDart;

Future<void> _wasmPromiseVoid(String method, [List<JSAny?>? arguments]) =>
    _wrapApiPromise(wasm.callMethodVarArgs(method.toJS, arguments));

//////////////////////////////////////////////////////////////////////////////

class _Ctx {
  int? _id;

  final VeilidJS js;

  _Ctx(int id, this.js) : _id = id;

  int requireId() {
    if (_id == null) {
      throw VeilidAPIExceptionNotInitialized();
    }
    return _id!;
  }

  void close() {
    if (_id != null) {
      wasm.callMethodVarArgs('release_routing_context'.toJS, [_id!.toJS]);
      _id = null;
    }
  }
}

// JS implementation of VeilidRoutingContext
class VeilidRoutingContextJS extends VeilidRoutingContext {
  final _Ctx _ctx;

  static final Finalizer<_Ctx> _finalizer = Finalizer((ctx) => ctx.close());

  VeilidRoutingContextJS._(this._ctx) {
    _finalizer.attach(this, _ctx, detach: this);
  }

  @override
  void close() {
    _ctx.close();
  }

  @override
  VeilidRoutingContextJS withDefaultSafety({bool closeSelf = false}) {
    final id = _ctx.requireId();
    final newId = _wasmCallInt('routing_context_with_default_safety', [
      id.toJS,
    ]);
    final out = VeilidRoutingContextJS._(_Ctx(newId, _ctx.js));
    if (closeSelf) {
      close();
    }
    return out;
  }

  @override
  VeilidRoutingContextJS withSafety(
    SafetySelection safetySelection, {
    bool closeSelf = false,
  }) {
    final id = _ctx.requireId();
    final newId = _wasmCallInt('routing_context_with_safety', [
      id.toJS,
      jsonEncode(safetySelection).toJS,
    ]);
    final out = VeilidRoutingContextJS._(_Ctx(newId, _ctx.js));
    if (closeSelf) {
      close();
    }
    return out;
  }

  @override
  VeilidRoutingContextJS withSequencing(
    Sequencing sequencing, {
    bool closeSelf = false,
  }) {
    final id = _ctx.requireId();
    final newId = _wasmCallInt('routing_context_with_sequencing', [
      id.toJS,
      jsonEncode(sequencing).toJS,
    ]);
    final out = VeilidRoutingContextJS._(_Ctx(newId, _ctx.js));
    if (closeSelf) {
      close();
    }
    return out;
  }

  @override
  Future<SafetySelection> safety() {
    final id = _ctx.requireId();
    return _wasmPromiseJson(
      'routing_context_safety',
      SafetySelection.fromJson,
      [id.toJS],
    );
  }

  @override
  Future<Uint8List> appCall(Target target, Uint8List request) {
    final id = _ctx.requireId();
    final encodedRequest = base64UrlNoPadEncode(request);

    return _wasmPromiseUint8List('routing_context_app_call', [
      id.toJS,
      jsonEncode(target).toJS,
      encodedRequest.toJS,
    ]);
  }

  @override
  Future<void> appMessage(Target target, Uint8List message) async {
    final id = _ctx.requireId();
    final encodedMessage = base64UrlNoPadEncode(message);

    await _wasmPromiseVoid('routing_context_app_message', [
      id.toJS,
      jsonEncode(target).toJS,
      encodedMessage.toJS,
    ]);
  }

  @override
  Future<DHTRecordDescriptor> createDHTRecord(
    CryptoKind kind,
    DHTSchema schema, {
    KeyPair? owner,
  }) {
    final id = _ctx.requireId();
    return _wasmPromiseJson(
      'routing_context_create_dht_record',
      DHTRecordDescriptor.fromJson,
      [
        id.toJS,
        kind.toInt().toJS,
        jsonEncode(schema).toJS,
        if (owner != null) jsonEncode(owner).toJS else null,
      ],
    );
  }

  @override
  Future<DHTRecordDescriptor> openDHTRecord(RecordKey key, {KeyPair? writer}) {
    final id = _ctx.requireId();
    return _wasmPromiseJson(
      'routing_context_open_dht_record',
      DHTRecordDescriptor.fromJson,
      [
        id.toJS,
        jsonEncode(key).toJS,
        if (writer != null) jsonEncode(writer).toJS else null,
      ],
    );
  }

  @override
  Future<void> closeDHTRecord(RecordKey key) {
    final id = _ctx.requireId();
    return _wasmPromiseVoid('routing_context_close_dht_record', [
      id.toJS,
      jsonEncode(key).toJS,
    ]);
  }

  @override
  Future<void> deleteDHTRecord(RecordKey key) {
    final id = _ctx.requireId();
    return _wasmPromiseVoid('routing_context_delete_dht_record', [
      id.toJS,
      jsonEncode(key).toJS,
    ]);
  }

  @override
  Future<ValueData?> getDHTValue(
    RecordKey key,
    int subkey, {
    bool forceRefresh = false,
  }) {
    final id = _ctx.requireId();
    return _wasmPromiseJsonOrNull(
      'routing_context_get_dht_value',
      ValueData.fromJson,
      [id.toJS, jsonEncode(key).toJS, subkey.toJS, forceRefresh.toJS],
    );
  }

  @override
  Future<ValueData?> setDHTValue(
    RecordKey key,
    int subkey,
    Uint8List data, {
    SetDHTValueOptions? options,
  }) {
    final id = _ctx.requireId();
    return _wasmPromiseJsonOrNull(
      'routing_context_set_dht_value',
      ValueData.fromJson,
      [
        id.toJS,
        jsonEncode(key).toJS,
        subkey.toJS,
        base64UrlNoPadEncode(data).toJS,
        if (options != null) jsonEncode(options).toJS else null,
      ],
    );
  }

  @override
  Future<bool> watchDHTValues(
    RecordKey key, {
    List<ValueSubkeyRange>? subkeys,
    Timestamp? expiration,
    int? count,
  }) {
    subkeys ??= [];
    expiration ??= Timestamp.zero();
    count ??= 0xFFFFFFFF;

    final id = _ctx.requireId();
    return _wasmPromiseBool('routing_context_watch_dht_values', [
      id.toJS,
      jsonEncode(key).toJS,
      jsonEncode(subkeys).toJS,
      expiration.toString().toJS,
      count.toJS,
    ]);
  }

  @override
  Future<bool> cancelDHTWatch(
    RecordKey key, {
    List<ValueSubkeyRange>? subkeys,
  }) {
    subkeys ??= [];

    final id = _ctx.requireId();
    return _wasmPromiseBool('routing_context_cancel_dht_watch', [
      id.toJS,
      jsonEncode(key).toJS,
      jsonEncode(subkeys).toJS,
    ]);
  }

  @override
  Future<DHTRecordReport> inspectDHTRecord(
    RecordKey key, {
    List<ValueSubkeyRange>? subkeys,
    DHTReportScope scope = DHTReportScope.local,
  }) {
    final id = _ctx.requireId();
    return _wasmPromiseJson(
      'routing_context_inspect_dht_record',
      DHTRecordReport.fromJson,
      [
        id.toJS,
        jsonEncode(key).toJS,
        if (subkeys != null) jsonEncode(subkeys).toJS else null,
        jsonEncode(scope).toJS,
      ],
    );
  }
}

// JS implementation of VeilidCryptoSystem
class VeilidCryptoSystemJS extends VeilidCryptoSystem {
  final int _kind;

  // Keep the reference
  // ignore: unused_field
  final VeilidJS _js;

  VeilidCryptoSystemJS._(this._js, this._kind);

  @override
  CryptoKind kind() => CryptoKind.fromInt(_kind);

  @override
  Future<SharedSecret> cachedDH(PublicKey key, SecretKey secret) =>
      _wasmPromiseJson('crypto_cached_dh', SharedSecret.fromJson, [
        _kind.toJS,
        jsonEncode(key).toJS,
        jsonEncode(secret).toJS,
      ]);

  @override
  Future<SharedSecret> computeDH(PublicKey key, SecretKey secret) =>
      _wasmPromiseJson('crypto_compute_dh', SharedSecret.fromJson, [
        _kind.toJS,
        jsonEncode(key).toJS,
        jsonEncode(secret).toJS,
      ]);

  @override
  Future<SharedSecret> generateSharedSecret(
    PublicKey key,
    SecretKey secret,
    Uint8List domain,
  ) =>
      _wasmPromiseJson('crypto_generate_shared_secret', SharedSecret.fromJson, [
        _kind.toJS,
        jsonEncode(key).toJS,
        jsonEncode(secret).toJS,
        base64UrlNoPadEncode(domain).toJS,
      ]);

  @override
  Future<Uint8List> randomBytes(int len) =>
      _wasmPromiseUint8List('crypto_random_bytes', [_kind.toJS, len.toJS]);

  @override
  Future<int> sharedSecretLength() =>
      _wasmPromiseInt('crypto_shared_secret_length', [_kind.toJS]);

  @override
  Future<int> nonceLength() =>
      _wasmPromiseInt('crypto_nonce_length', [_kind.toJS]);

  @override
  Future<int> hashDigestLength() =>
      _wasmPromiseInt('crypto_hash_digest_length', [_kind.toJS]);

  @override
  Future<int> publicKeyLength() =>
      _wasmPromiseInt('crypto_public_key_length', [_kind.toJS]);

  @override
  Future<int> secretKeyLength() =>
      _wasmPromiseInt('crypto_secret_key_length', [_kind.toJS]);

  @override
  Future<int> signatureLength() =>
      _wasmPromiseInt('crypto_signature_length', [_kind.toJS]);

  @override
  Future<int> defaultSaltLength() =>
      _wasmPromiseInt('crypto_default_salt_length', [_kind.toJS]);

  @override
  Future<int> aeadOverhead() =>
      _wasmPromiseInt('crypto_aead_overhead', [_kind.toJS]);

  @override
  Future<void> checkSharedSecret(SharedSecret secret) => _wasmPromiseVoid(
    'crypto_check_shared_secret',
    [_kind.toJS, jsonEncode(secret).toJS],
  );

  @override
  Future<void> checkNonce(Nonce nonce) => _wasmPromiseVoid(
    'crypto_check_nonce',
    [_kind.toJS, jsonEncode(nonce).toJS],
  );

  @override
  Future<void> checkHashDigest(HashDigest digest) => _wasmPromiseVoid(
    'crypto_check_hash_digest',
    [_kind.toJS, jsonEncode(digest).toJS],
  );

  @override
  Future<void> checkPublicKey(PublicKey key) => _wasmPromiseVoid(
    'crypto_check_public_key',
    [_kind.toJS, jsonEncode(key).toJS],
  );

  @override
  Future<void> checkSecretKey(SecretKey key) => _wasmPromiseVoid(
    'crypto_check_secret_key',
    [_kind.toJS, jsonEncode(key).toJS],
  );

  @override
  Future<void> checkSignature(Signature signature) => _wasmPromiseVoid(
    'crypto_check_signature',
    [_kind.toJS, jsonEncode(signature).toJS],
  );

  @override
  Future<String> hashPassword(Uint8List password, Uint8List salt) =>
      _wasmPromiseString('crypto_hash_password', [
        _kind.toJS,
        base64UrlNoPadEncode(password).toJS,
        base64UrlNoPadEncode(salt).toJS,
      ]);

  @override
  Future<bool> verifyPassword(Uint8List password, String passwordHash) =>
      _wasmPromiseBool('crypto_verify_password', [
        _kind.toJS,
        base64UrlNoPadEncode(password).toJS,
        passwordHash.toJS,
      ]);

  @override
  Future<SharedSecret> deriveSharedSecret(Uint8List password, Uint8List salt) =>
      _wasmPromiseJson('crypto_derive_shared_secret', SharedSecret.fromJson, [
        _kind.toJS,
        base64UrlNoPadEncode(password).toJS,
        base64UrlNoPadEncode(salt).toJS,
      ]);

  @override
  Future<Nonce> randomNonce() =>
      _wasmPromiseJson('crypto_random_nonce', Nonce.fromJson, [_kind.toJS]);

  @override
  Future<SharedSecret> randomSharedSecret() => _wasmPromiseJson(
    'crypto_random_shared_secret',
    SharedSecret.fromJson,
    [_kind.toJS],
  );

  @override
  Future<KeyPair> generateKeyPair() => _wasmPromiseJson(
    'crypto_generate_key_pair',
    KeyPair.fromJson,
    [_kind.toJS],
  );

  @override
  Future<HashDigest> generateHash(Uint8List data) => _wasmPromiseJson(
    'crypto_generate_hash',
    HashDigest.fromJson,
    [_kind.toJS, base64UrlNoPadEncode(data).toJS],
  );

  @override
  Future<bool> validateKeyPair(PublicKey key, SecretKey secret) =>
      _wasmPromiseBool('crypto_validate_key_pair', [
        _kind.toJS,
        jsonEncode(key).toJS,
        jsonEncode(secret).toJS,
      ]);

  @override
  Future<bool> validateHash(Uint8List data, HashDigest hash) =>
      _wasmPromiseBool('crypto_validate_hash', [
        _kind.toJS,
        base64UrlNoPadEncode(data).toJS,
        jsonEncode(hash).toJS,
      ]);

  @override
  Future<Signature> sign(PublicKey key, SecretKey secret, Uint8List data) =>
      _wasmPromiseJson('crypto_sign', Signature.fromJson, [
        _kind.toJS,
        jsonEncode(key).toJS,
        jsonEncode(secret).toJS,
        base64UrlNoPadEncode(data).toJS,
      ]);

  @override
  Future<bool> verify(PublicKey key, Uint8List data, Signature signature) =>
      _wasmPromiseBool('crypto_verify', [
        _kind.toJS,
        jsonEncode(key).toJS,
        base64UrlNoPadEncode(data).toJS,
        jsonEncode(signature).toJS,
      ]);

  @override
  Future<Uint8List> decryptAead(
    Uint8List body,
    Nonce nonce,
    SharedSecret sharedSecret,
    Uint8List? associatedData,
  ) => _wasmPromiseUint8List('crypto_decrypt_aead', [
    _kind.toJS,
    base64UrlNoPadEncode(body).toJS,
    jsonEncode(nonce).toJS,
    jsonEncode(sharedSecret).toJS,
    if (associatedData != null)
      base64UrlNoPadEncode(associatedData).toJS
    else
      null,
  ]);

  @override
  Future<Uint8List> encryptAead(
    Uint8List body,
    Nonce nonce,
    SharedSecret sharedSecret,
    Uint8List? associatedData,
  ) => _wasmPromiseUint8List('crypto_encrypt_aead', [
    _kind.toJS,
    base64UrlNoPadEncode(body).toJS,
    jsonEncode(nonce).toJS,
    jsonEncode(sharedSecret).toJS,
    if (associatedData != null)
      base64UrlNoPadEncode(associatedData).toJS
    else
      null,
  ]);

  @override
  Future<Uint8List> cryptNoAuth(
    Uint8List body,
    Nonce nonce,
    SharedSecret sharedSecret,
  ) => _wasmPromiseUint8List('crypto_crypt_no_auth', [
    _kind.toJS,
    base64UrlNoPadEncode(body).toJS,
    jsonEncode(nonce).toJS,
    jsonEncode(sharedSecret).toJS,
  ]);
}

class _TDBT {
  int? id;

  final VeilidTableDBJS tdbjs;

  final VeilidJS js;

  _TDBT(this.id, this.tdbjs, this.js);

  void ensureValid() {
    if (id == null) {
      throw VeilidAPIExceptionNotInitialized();
    }
  }

  void close() {
    if (id != null) {
      _wasmCallVoid('release_table_db_transaction', [id!.toJS]);
      id = null;
    }
  }
}

// JS implementation of VeilidTableDBTransaction
class VeilidTableDBTransactionJS extends VeilidTableDBTransaction {
  final _TDBT _tdbt;

  static final Finalizer<_TDBT> _finalizer = Finalizer((tdbt) => tdbt.close());

  VeilidTableDBTransactionJS._(this._tdbt) {
    _finalizer.attach(this, _tdbt, detach: this);
  }

  @override
  bool get isDone => _tdbt.id == null;

  @override
  Future<void> commit() async {
    _tdbt.ensureValid();
    final id = _tdbt.id!;
    await _wasmPromiseVoid('table_db_transaction_commit', [id.toJS]);
    _tdbt.close();
  }

  @override
  Future<void> rollback() async {
    _tdbt.ensureValid();
    final id = _tdbt.id!;
    await _wasmPromiseVoid('table_db_transaction_rollback', [id.toJS]);
    _tdbt.close();
  }

  @override
  Future<void> store(int col, Uint8List key, Uint8List value) {
    _tdbt.ensureValid();
    final id = _tdbt.id!;
    final encodedKey = base64UrlNoPadEncode(key);
    final encodedValue = base64UrlNoPadEncode(value);

    return _wasmPromiseVoid('table_db_transaction_store', [
      id.toJS,
      col.toJS,
      encodedKey.toJS,
      encodedValue.toJS,
    ]);
  }

  @override
  Future<void> delete(int col, Uint8List key) {
    _tdbt.ensureValid();
    final id = _tdbt.id!;
    final encodedKey = base64UrlNoPadEncode(key);

    return _wasmPromiseVoid('table_db_transaction_delete', [
      id.toJS,
      col.toJS,
      encodedKey.toJS,
    ]);
  }
}

class _TDB {
  int? _id;

  final VeilidJS js;

  _TDB(int id, this.js) : _id = id;

  int requireId() {
    if (_id == null) {
      throw VeilidAPIExceptionNotInitialized();
    }
    return _id!;
  }

  void close() {
    if (_id != null) {
      _wasmCallVoid('release_table_db', [_id!.toJS]);
      _id = null;
    }
  }
}

// JS implementation of VeilidDHTTransaction
class VeilidDHTTransactionJS extends VeilidDHTTransaction {
  final _DTX _dtx;

  static final Finalizer<_DTX> _finalizer = Finalizer((dtx) => dtx.close());

  VeilidDHTTransactionJS._(this._dtx) {
    _finalizer.attach(this, _dtx, detach: this);
  }

  @override
  bool get isDone => _dtx.id == null;

  @override
  Future<void> commit() async {
    final id = _dtx.requireId();
    await _wasmPromiseVoid('dht_transaction_commit', [id.toJS]);
    _dtx.close();
  }

  @override
  Future<void> rollback() async {
    final id = _dtx.requireId();
    await _wasmPromiseVoid('dht_transaction_rollback', [id.toJS]);
    _dtx.close();
  }

  @override
  Future<ValueData?> get(RecordKey key, int subkey) async {
    final id = _dtx.requireId();
    return _wasmPromiseJsonOrNull('dht_transaction_get', ValueData.fromJson, [
      id.toJS,
      jsonEncode(key).toJS,
      subkey.toJS,
    ]);
  }

  @override
  Future<ValueData?> set(
    RecordKey key,
    int subkey,
    Uint8List data, {
    DHTTransactionSetValueOptions? options,
  }) {
    final id = _dtx.requireId();
    return _wasmPromiseJsonOrNull('dht_transaction_set', ValueData.fromJson, [
      id.toJS,
      jsonEncode(key).toJS,
      subkey.toJS,
      base64UrlNoPadEncode(data).toJS,
      if (options != null) jsonEncode(options).toJS else null,
    ]);
  }

  @override
  Future<DHTRecordReport> inspect(
    RecordKey key, {
    List<ValueSubkeyRange>? subkeys,
    DHTReportScope scope = DHTReportScope.local,
  }) {
    final id = _dtx.requireId();
    return _wasmPromiseJson(
      'dht_transaction_inspect',
      DHTRecordReport.fromJson,
      [
        id.toJS,
        jsonEncode(key).toJS,
        if (subkeys != null) jsonEncode(subkeys).toJS else null,
        jsonEncode(scope).toJS,
      ],
    );
  }
}

class _DTX {
  int? id;

  final VeilidJS js;

  _DTX(int this.id, this.js);

  int requireId() {
    if (id == null) {
      throw VeilidAPIExceptionNotInitialized();
    }
    return id!;
  }

  void close() {
    if (id != null) {
      _wasmCallVoid('release_dht_transaction', [id!.toJS]);
      id = null;
    }
  }
}

// JS implementation of VeilidTableDB
class VeilidTableDBJS extends VeilidTableDB {
  final _TDB _tdb;

  static final Finalizer<_TDB> _finalizer = Finalizer((tdb) => tdb.close());

  VeilidTableDBJS._(this._tdb) {
    _finalizer.attach(this, _tdb, detach: this);
  }

  @override
  void close() {
    _tdb.close();
  }

  @override
  int get columnCount {
    final id = _tdb.requireId();
    return _wasmCallInt('table_db_get_column_count', [id.toJS]);
  }

  @override
  Future<List<Uint8List>> getKeys(int col) async {
    final id = _tdb.requireId();
    return await _wasmCallJson(
      'table_db_get_keys',
      jsonListConstructor(base64UrlNoPadDecodeDynamic),
      [id.toJS, col.toJS],
    );
  }

  @override
  VeilidTableDBTransaction transact() {
    final id = _tdb.requireId();
    final xid = _wasmCallInt('table_db_transact', [id.toJS]);

    return VeilidTableDBTransactionJS._(_TDBT(xid, this, _tdb.js));
  }

  @override
  Future<void> store(int col, Uint8List key, Uint8List value) {
    final id = _tdb.requireId();
    final encodedKey = base64UrlNoPadEncode(key);
    final encodedValue = base64UrlNoPadEncode(value);

    return _wasmPromiseVoid('table_db_store', [
      id.toJS,
      col.toJS,
      encodedKey.toJS,
      encodedValue.toJS,
    ]);
  }

  @override
  Future<Uint8List?> load(int col, Uint8List key) async {
    final id = _tdb.requireId();
    final encodedKey = base64UrlNoPadEncode(key);

    return _wasmPromiseUint8ListOrNull('table_db_load', [
      id.toJS,
      col.toJS,
      encodedKey.toJS,
    ]);
  }

  @override
  Future<Uint8List?> delete(int col, Uint8List key) async {
    final id = _tdb.requireId();
    final encodedKey = base64UrlNoPadEncode(key);

    return _wasmPromiseUint8ListOrNull('table_db_delete', [
      id.toJS,
      col.toJS,
      encodedKey.toJS,
    ]);
  }
}

// JS implementation of high level Veilid API

class VeilidJS extends Veilid {
  @override
  void initializeVeilidCore(Map<String, dynamic> platformConfigJson) {
    final platformConfigJsonString = jsonEncode(platformConfigJson);
    _wasmCallVoid('initialize_veilid_core', [platformConfigJsonString.toJS]);
  }

  @override
  void changeLogLevel(String layer, VeilidConfigLogLevel logLevel) {
    final logLevelJsonString = jsonEncode(logLevel);
    _wasmCallVoid('change_log_level', [layer.toJS, logLevelJsonString.toJS]);
  }

  @override
  void changeLogIgnore(String layer, List<String> changes) {
    final changesJsonString = jsonEncode(changes.join(','));
    _wasmCallVoid('change_log_ignore', [layer.toJS, changesJsonString.toJS]);
  }

  @override
  Future<Stream<VeilidUpdate>> startupVeilidCore(VeilidConfig config) async {
    final streamController = StreamController<VeilidUpdate>();
    void updateCallback(String update) {
      final updateJson = jsonDecode(update) as Map<String, dynamic>;
      if (updateJson['kind'] == 'Shutdown') {
        unawaited(streamController.close());
      } else {
        final update = VeilidUpdate.fromJson(updateJson);
        streamController.add(update);
      }
    }

    await _wasmPromiseVoid('startup_veilid_core', [
      updateCallback.toJS,
      jsonEncode(config).toJS,
    ]);

    return streamController.stream;
  }

  @override
  Future<VeilidState> getVeilidState() =>
      _wasmPromiseJson('get_veilid_state', VeilidState.fromJson, []);

  @override
  Future<bool> isShutdown() => _wasmPromiseBool('is_shutdown', []);

  @override
  Future<void> attach() => _wasmPromiseVoid('attach', []);

  @override
  Future<void> detach() => _wasmPromiseVoid('detach', []);

  @override
  Future<void> shutdownVeilidCore() =>
      _wasmPromiseVoid('shutdown_veilid_core', []);

  @override
  List<CryptoKind> validCryptoKinds() => _wasmCallJson(
    'valid_crypto_kinds',
    (x) =>
        (x as List<dynamic>).map((x) => CryptoKind.fromInt(x as int)).toList(),
    [],
  );

  @override
  Future<VeilidCryptoSystem> getCryptoSystem(CryptoKind kind) async {
    final vck = validCryptoKinds();
    if (!vck.contains(kind)) {
      throw const VeilidAPIExceptionGeneric('unsupported cryptosystem');
    }
    return VeilidCryptoSystemJS._(this, kind.toInt());
  }

  @override
  Future<List<PublicKey>?> verifySignatures(
    List<PublicKey> nodeIds,
    Uint8List data,
    List<Signature> signatures,
  ) => _wasmPromiseJson(
    'verify_signatures',
    optJsonListConstructor(PublicKey.fromJson),
    [
      jsonEncode(nodeIds).toJS,
      base64UrlNoPadEncode(data).toJS,
      jsonEncode(signatures).toJS,
    ],
  );

  @override
  Future<List<Signature>> generateSignatures(
    Uint8List data,
    List<KeyPair> keyPairs,
  ) => _wasmPromiseJson(
    'generate_signatures',
    jsonListConstructor(Signature.fromJson),
    [base64UrlNoPadEncode(data).toJS, jsonEncode(keyPairs).toJS],
  );

  @override
  Future<KeyPair> generateKeyPair(CryptoKind kind) => _wasmPromiseJson(
    'generate_key_pair',
    KeyPair.fromJson,
    [kind.toInt().toJS],
  );

  @override
  Future<VeilidRoutingContext> routingContext() async {
    final rcid = await _wasmPromiseInt('routing_context', []);
    return VeilidRoutingContextJS._(_Ctx(rcid, this));
  }

  @override
  Future<MemberId> generateMemberId(PublicKey writerKey) => _wasmPromiseJson(
    'generate_member_id',
    MemberId.fromJson,
    [jsonEncode(writerKey).toJS],
  );

  @override
  Future<VeilidDHTTransaction> transactDHTRecords(
    List<RecordKey> recordKeys, {
    TransactDHTRecordsOptions? options,
  }) async {
    final xid = await _wasmPromiseInt('transact_dht_records', [
      jsonEncode(recordKeys).toJS,
      if (options != null) jsonEncode(options).toJS else null,
    ]);
    return VeilidDHTTransactionJS._(_DTX(xid, this));
  }

  @override
  Future<RecordKey> getDHTRecordKey(
    DHTSchema schema,
    PublicKey owner,
    SharedSecret? encryptionKey,
  ) => _wasmPromiseJson('get_dht_record_key', RecordKey.fromJson, [
    jsonEncode(schema).toJS,
    jsonEncode(owner).toJS,
    if (encryptionKey != null) jsonEncode(encryptionKey).toJS else null,
  ]);

  @override
  Future<RouteBlob> newPrivateRoute() =>
      _wasmPromiseJson('new_private_route', RouteBlob.fromJson, []);

  @override
  Future<RouteBlob> newCustomPrivateRoute(
    Stability stability,
    Sequencing sequencing,
  ) {
    final stabilityString = jsonEncode(stability);
    final sequencingString = jsonEncode(sequencing);

    return _wasmPromiseJson('new_private_route', RouteBlob.fromJson, [
      stabilityString.toJS,
      sequencingString.toJS,
    ]);
  }

  @override
  Future<RouteId> importRemotePrivateRoute(Uint8List blob) {
    final encodedBlob = base64UrlNoPadEncode(blob);
    return _wasmPromiseJson('import_remote_private_route', RouteId.fromJson, [
      encodedBlob.toJS,
    ]);
  }

  @override
  Future<void> releasePrivateRoute(RouteId routeId) =>
      _wasmPromiseVoid('release_private_route', [jsonEncode(routeId).toJS]);

  @override
  Future<void> appCallReply(String callId, Uint8List message) {
    final encodedMessage = base64UrlNoPadEncode(message);
    return _wasmPromiseVoid('app_call_reply', [
      callId.toJS,
      encodedMessage.toJS,
    ]);
  }

  @override
  Future<VeilidTableDB> openTableDB(String name, int columnCount) async {
    final dbid = await _wasmPromiseInt('open_table_db', [
      name.toJS,
      columnCount.toJS,
    ]);
    return VeilidTableDBJS._(_TDB(dbid, this));
  }

  @override
  Future<bool> deleteTableDB(String name) =>
      _wasmPromiseBool('delete_table_db', [name.toJS]);

  @override
  Timestamp now() => Timestamp.fromString(_wasmCallString('now', []));

  @override
  Future<String> debug(String command) =>
      _wasmPromiseString('debug', [command.toJS]);

  @override
  String veilidVersionString() => _wasmCallString('veilid_version_string');

  @override
  VeilidVersion veilidVersion() {
    final jsonVersion = _wasmCallJson(
      'veilid_version',
      (x) => x as Map<String, dynamic>,
      [],
    );
    return VeilidVersion(
      jsonVersion['major'] as int,
      jsonVersion['minor'] as int,
      jsonVersion['patch'] as int,
    );
  }

  @override
  List<String> veilidFeatures() =>
      _wasmCallJson('veilid_features', (x) => x as List<String>, []);

  @override
  String defaultVeilidConfig() => _wasmCallString('default_veilid_config', []);
}
