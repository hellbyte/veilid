use super::*;
use crate::tests::*;

async fn startup() -> VeilidAPI {
    info!("test_table_store: starting");
    let (update_callback, config) = fixture_veilid_core();
    api_startup(update_callback, config)
        .await
        .expect("startup failed")
}

async fn shutdown(api: VeilidAPI) {
    info!("test_table_store: shutting down");
    api.shutdown().await;
    info!("test_table_store: finished");
}

async fn test_delete_open_delete(ts: &TableStore) {
    info!("test_delete_open_delete");

    let _ = ts.delete("test").await;
    let db = ts.open("test", 3).await.expect("should have opened");
    assert!(
        ts.delete("test").await.is_err(),
        "should fail because file is opened"
    );
    drop(db);
    assert!(
        ts.delete("test").await.is_ok(),
        "should succeed because file is closed"
    );
    let db = ts.open("test", 3).await.expect("should have opened");
    assert!(
        ts.delete("test").await.is_err(),
        "should fail because file is opened"
    );
    drop(db);
    let db = ts.open("test", 3).await.expect("should have opened");
    assert!(
        ts.delete("test").await.is_err(),
        "should fail because file is opened"
    );
    drop(db);
    assert!(
        ts.delete("test").await.is_ok(),
        "should succeed because file is closed"
    );
}

async fn test_store_delete_load(ts: &TableStore) {
    info!("test_store_delete_load");

    let _ = ts.delete("test").await;
    let db = ts.open("test", 3).await.expect("should have opened");
    assert!(
        ts.delete("test").await.is_err(),
        "should fail because file is opened"
    );

    assert_eq!(
        db.load(0, b"foo").await.unwrap(),
        None,
        "should not load missing key"
    );
    assert!(
        db.store(1, b"foo", b"1234567890").await.is_ok(),
        "should store new key"
    );
    assert_eq!(
        db.load(0, b"foo").await.unwrap(),
        None,
        "should not load missing key"
    );
    assert_eq!(
        db.load(1, b"foo").await.unwrap(),
        Some(b"1234567890".to_vec())
    );

    assert!(
        db.store(1, b"bar", b"FNORD").await.is_ok(),
        "should store new key"
    );
    assert!(
        db.store(0, b"bar", b"ABCDEFGHIJKLMNOPQRSTUVWXYZ")
            .await
            .is_ok(),
        "should store new key"
    );
    assert!(
        db.store(2, b"bar", b"FNORD").await.is_ok(),
        "should store new key"
    );
    assert!(
        db.store(2, b"baz", b"QWERTY").await.is_ok(),
        "should store new key"
    );
    assert!(
        db.store(2, b"bar", b"QWERTYUIOP").await.is_ok(),
        "should store new key"
    );

    assert_eq!(db.load(1, b"bar").await.unwrap(), Some(b"FNORD".to_vec()));
    assert_eq!(
        db.load(0, b"bar").await.unwrap(),
        Some(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_vec())
    );
    assert_eq!(
        db.load(2, b"bar").await.unwrap(),
        Some(b"QWERTYUIOP".to_vec())
    );
    assert_eq!(db.load(2, b"baz").await.unwrap(), Some(b"QWERTY".to_vec()));

    assert_eq!(db.delete(1, b"bar").await.unwrap(), Some(b"FNORD".to_vec()));
    assert_eq!(db.delete(1, b"bar").await.unwrap(), None);
    assert!(
        db.delete(4, b"bar").await.is_err(),
        "can't delete from column that doesn't exist"
    );

    drop(db);
    let db = ts.open("test", 3).await.expect("should have opened");

    assert_eq!(db.load(1, b"bar").await.unwrap(), None);
    assert_eq!(
        db.load(0, b"bar").await.unwrap(),
        Some(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_vec())
    );
    assert_eq!(
        db.load(2, b"bar").await.unwrap(),
        Some(b"QWERTYUIOP".to_vec())
    );
    assert_eq!(db.load(2, b"baz").await.unwrap(), Some(b"QWERTY".to_vec()));
}

async fn test_transaction(ts: &TableStore) {
    info!("test_transaction");

    let _ = ts.delete("test").await;
    let db = ts.open("test", 3).await.expect("should have opened");
    assert!(
        ts.delete("test").await.is_err(),
        "should fail because file is opened"
    );

    let tx = db.transact();
    assert!(tx.store(0, b"aaa", b"a-value").await.is_ok());
    assert!(tx
        .store_json(1, b"bbb", &"b-value".to_owned())
        .await
        .is_ok());
    assert!(tx.store(3, b"ddd", b"d-value").await.is_err());
    assert!(tx.store(0, b"ddd", b"d-value").await.is_ok());
    assert!(tx.delete(0, b"ddd").await.is_ok());
    assert!(tx.commit().await.is_ok());

    let tx = db.transact();
    assert!(tx.delete(2, b"ccc").await.is_ok());
    tx.rollback();

    assert_eq!(db.load(0, b"aaa").await, Ok(Some(b"a-value".to_vec())));
    assert_eq!(
        db.load_json::<String>(1, b"bbb").await,
        Ok(Some("b-value".to_owned()))
    );
    assert_eq!(db.load(0, b"ddd").await, Ok(None));
}

async fn test_json(vcrypto: &AsyncCryptoSystemGuard<'_>, ts: &TableStore) {
    info!("test_json");

    let _ = ts.delete("test").await;
    let db = ts.open("test", 3).await.expect("should have opened");
    let keypair = vcrypto.generate_keypair().await;

    assert!(db.store_json(0, b"asdf", &keypair).await.is_ok());

    assert_eq!(db.load_json::<KeyPair>(0, b"qwer").await.unwrap(), None);

    let d = match db.load_json::<KeyPair>(0, b"asdf").await {
        Ok(x) => x,
        Err(e) => {
            panic!("couldn't decode: {}", e);
        }
    };
    assert_eq!(d, Some(keypair.clone()), "keys should be equal");

    let d = match db.delete_json::<KeyPair>(0, b"asdf").await {
        Ok(x) => x,
        Err(e) => {
            panic!("couldn't decode: {}", e);
        }
    };
    assert_eq!(d, Some(keypair.clone()), "keys should be equal");

    assert!(
        db.store(1, b"foo", b"1234567890").await.is_ok(),
        "should store new key"
    );

    assert!(
        db.load_json::<PublicKey>(1, b"foo").await.is_err(),
        "should fail to unfreeze"
    );
}

async fn test_protect_unprotect(vcrypto: &AsyncCryptoSystemGuard<'_>, ts: &TableStore) {
    info!("test_protect_unprotect");

    let dek1 = SharedSecret::new(
        vcrypto.kind(),
        BareSharedSecret::new(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]),
    );

    let dek2 = SharedSecret::new(
        vcrypto.kind(),
        BareSharedSecret::new(&[
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0xFF,
        ]),
    );

    let dek3 = SharedSecret::new(
        vcrypto.kind(),
        BareSharedSecret::new(&[0x80u8; VLD0_SHARED_SECRET_LENGTH]),
    );

    let deks = [dek1, dek2, dek3];
    let passwords = [
        "",
        " ",
        "  ",
        "12345678",
        "|/\\!@#$%^&*()_+",
        "Ⓜ️",
        "🔥🔥♾️",
    ];

    for dek in deks {
        for password in passwords {
            info!("testing dek {} with password {}", dek, password);
            let dek_bytes = ts
                .maybe_protect_device_encryption_key(dek.clone(), password)
                .await
                .unwrap_or_else(|_| panic!("protect: dek: '{}' pw: '{}'", dek, password));

            let unprotected = ts
                .maybe_unprotect_device_encryption_key(&dek_bytes, password)
                .await
                .unwrap_or_else(|_| panic!("unprotect: dek: '{}' pw: '{}'", dek, password));
            assert_eq!(unprotected, dek);
            let invalid_password = format!("{}x", password);
            let _ = ts
                .maybe_unprotect_device_encryption_key(&dek_bytes, &invalid_password)
                .await
                .expect_err(&format!(
                    "invalid_password: dek: '{}' pw: '{}'",
                    dek, &invalid_password
                ));
            if !password.is_empty() {
                let _ = ts
                    .maybe_unprotect_device_encryption_key(&dek_bytes, "")
                    .await
                    .expect_err(&format!("empty_password: dek: '{}' pw: ''", dek));
            }
        }
    }
}

async fn test_store_load_json_many(ts: &TableStore) {
    info!("test_json");

    let _ = ts.delete("test").await;
    let db = ts.open("test", 3).await.expect("should have opened");

    let rows = 16;
    let valuesize = 32768;
    let parallel = 10;

    let value = vec!["ABCD".to_string(); valuesize];

    let mut unord = FuturesUnordered::new();

    let mut r = 0;
    let start_ts = Timestamp::now();
    let mut keys = HashSet::new();
    loop {
        while r < rows && unord.len() < parallel {
            let key = format!("key_{}", r);
            r += 1;

            unord.push(Box::pin(async {
                let key = key;

                db.store_json(0, key.as_bytes(), &value)
                    .await
                    .expect("should store");
                let value2 = db
                    .load_json::<Vec<String>>(0, key.as_bytes())
                    .await
                    .expect("should load")
                    .expect("should exist");
                assert_eq!(value, value2);

                key.as_bytes().to_vec()
            }));
        }
        if let Some(res) = unord.next().await {
            keys.insert(res);
        } else {
            break;
        }
    }

    let stored_keys = db.get_keys(0).await.expect("should get keys");
    let stored_keys_set = stored_keys.into_iter().collect::<HashSet<_>>();
    assert_eq!(stored_keys_set, keys, "should have same keys");

    let end_ts = Timestamp::now();
    trace!(
        "test_store_load_json_many duration={}",
        end_ts.duration_since(start_ts)
    );
}

pub async fn test_all() {
    let api = startup().await;
    let crypto = api.crypto().unwrap();
    let ts = api.table_store().unwrap();

    test_store_load_json_many(&ts).await;

    for ck in VALID_CRYPTO_KINDS {
        let vcrypto = crypto.get_async(ck).unwrap();
        test_protect_unprotect(&vcrypto, &ts).await;
        test_delete_open_delete(&ts).await;
        test_store_delete_load(&ts).await;
        test_transaction(&ts).await;
        test_json(&vcrypto, &ts).await;
        let _ = ts.delete("test").await;
    }

    shutdown(api).await;
}
