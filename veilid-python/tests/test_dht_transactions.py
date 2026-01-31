# DHT Transaction Veilid Tests
from typing import Any, Awaitable, Callable, Optional, Coroutine

import pytest
import asyncio
import time
import os

import veilid
from veilid import *
from veilid.types import *

##################################################################
BOGUS_KEY = RecordKey.from_value(
    CryptoKind.CRYPTO_KIND_VLD0, BareRecordKey.from_parts(BareOpaqueRecordKey.from_bytes(b'                                '), None))

TEST_MESSAGE_1 = b"BLAH BLAH BLAH"
TEST_MESSAGE_2 = b"blah blah blah blah"

@pytest.mark.asyncio
async def test_transact_dht_records_empty(api_connection: VeilidAPI):
    with pytest.raises(VeilidAPIError):
        await api_connection.transact_dht_records([], None)

@pytest.mark.asyncio
async def test_transact_dht_records_unopened(api_connection: VeilidAPI):
    with pytest.raises(VeilidAPIError):
        await api_connection.transact_dht_records([BOGUS_KEY], None)

@pytest.mark.asyncio
async def test_transact_dht_records_duplicate(api_connection: VeilidAPI):
    with pytest.raises(VeilidAPIError):
        await api_connection.transact_dht_records([BOGUS_KEY, BOGUS_KEY], None)

@pytest.mark.asyncio
async def test_transact_dht_records_nonexistent_with_options(api_connection: VeilidAPI):
    for kind in await api_connection.valid_crypto_kinds():
        cs = await api_connection.get_crypto_system(kind)
        async with cs:
            default_signing_keypair = await cs.generate_key_pair()

        with pytest.raises(VeilidAPIError):
            await api_connection.transact_dht_records([BOGUS_KEY], TransactDHTRecordsOptions(default_signing_keypair=default_signing_keypair))


@pytest.mark.asyncio
async def test_transact_dht_records_close_out_of_order_one_of_one(api_connection: VeilidAPI):
    rc = await api_connection.new_routing_context()
    async with rc:
        for kind in await api_connection.valid_crypto_kinds():
            rec = await rc.create_dht_record(kind, DHTSchema.dflt(1))

            rec_tx = await api_connection.transact_dht_records([rec.key], None)
            async with rec_tx:
                await rc.close_dht_record(rec.key)
                await rc.delete_dht_record(rec.key)

@pytest.mark.asyncio
async def test_transact_dht_records_close_out_of_order_one_of_two(api_connection: VeilidAPI):
    rc = await api_connection.new_routing_context()
    async with rc:
        for kind in await api_connection.valid_crypto_kinds():
            rec1 = await rc.create_dht_record(kind, DHTSchema.dflt(1))
            rec2 = await rc.create_dht_record(kind, DHTSchema.dflt(1))

            rec_tx = await api_connection.transact_dht_records([rec1.key, rec2.key], None)
            async with rec_tx:
                await rc.close_dht_record(rec1.key)
                await rc.delete_dht_record(rec1.key)

            await rc.close_dht_record(rec2.key)
            await rc.delete_dht_record(rec2.key)


@pytest.mark.asyncio
async def test_transact_dht_records_close_out_of_order_two_of_two(api_connection: VeilidAPI):
    rc = await api_connection.new_routing_context()
    async with rc:
        for kind in await api_connection.valid_crypto_kinds():
            rec1 = await rc.create_dht_record(kind, DHTSchema.dflt(1))
            rec2 = await rc.create_dht_record(kind, DHTSchema.dflt(1))

            rec_tx = await api_connection.transact_dht_records([rec1.key, rec2.key], None)
            async with rec_tx:
                await rc.close_dht_record(rec1.key)
                await rc.close_dht_record(rec2.key)

            await rc.delete_dht_record(rec1.key)
            await rc.delete_dht_record(rec2.key)



@pytest.mark.asyncio
async def test_transact_dht_records_get_nonexistent(api_connection: VeilidAPI):
    rc = await api_connection.new_routing_context()
    async with rc:
        for kind in await api_connection.valid_crypto_kinds():
            rec = await rc.create_dht_record(kind, DHTSchema.dflt(1))

            rec_tx = await api_connection.transact_dht_records([rec.key], None)
            async with rec_tx:
                assert await rec_tx.get(rec.key, ValueSubkey(0)) is None

            await rc.close_dht_record(rec.key)
            await rc.delete_dht_record(rec.key)


@pytest.mark.asyncio
async def test_transact_dht_records_set_commit_get(api_connection: VeilidAPI):
    rc = await api_connection.new_routing_context()
    async with rc:
        for kind in await api_connection.valid_crypto_kinds():
            rec = await rc.create_dht_record(kind, DHTSchema.dflt(2))

            rec_tx = await api_connection.transact_dht_records([rec.key], None)
            async with rec_tx:
                vd = await rec_tx.set(rec.key, ValueSubkey(0), TEST_MESSAGE_1)
                assert vd is None
                await rec_tx.commit()

            vd2 = await rc.get_dht_value(rec.key, ValueSubkey(0), True)
            assert vd2 is not None

            assert vd2.data == TEST_MESSAGE_1

            await rc.close_dht_record(rec.key)
            await rc.delete_dht_record(rec.key)



@pytest.mark.asyncio
async def test_transact_dht_records_set_commit_delete_get(api_connection: VeilidAPI):
    rc = await api_connection.new_routing_context()
    async with rc:
        for kind in await api_connection.valid_crypto_kinds():
            rec = await rc.create_dht_record(kind, DHTSchema.dflt(2))

            rec_tx = await api_connection.transact_dht_records([rec.key], None)
            async with rec_tx:
                vd = await rec_tx.set(rec.key, ValueSubkey(0), TEST_MESSAGE_1)
                assert vd is None
                await rec_tx.commit()

            await rc.close_dht_record(rec.key)
            await rc.delete_dht_record(rec.key)
            # Reopen the record readonly
            rec = await rc.open_dht_record(rec.key)

            vd2 = await rc.get_dht_value(rec.key, ValueSubkey(0), True)
            assert vd2 is not None

            assert vd2.data == TEST_MESSAGE_1

            await rc.close_dht_record(rec.key)
            await rc.delete_dht_record(rec.key)




@pytest.mark.asyncio
async def test_transact_dht_records_set_rollback_get(api_connection: VeilidAPI):
    rc = await api_connection.new_routing_context()
    async with rc:
        for kind in await api_connection.valid_crypto_kinds():
            rec = await rc.create_dht_record(kind, DHTSchema.dflt(2))

            rec_tx = await api_connection.transact_dht_records([rec.key], None)
            async with rec_tx:
                vd = await rec_tx.set(rec.key, ValueSubkey(0), TEST_MESSAGE_1)
                assert vd is None
                await rec_tx.rollback()

            vd2 = await rc.get_dht_value(rec.key, ValueSubkey(0), True)
            assert vd2 is None

            await rc.close_dht_record(rec.key)
            await rc.delete_dht_record(rec.key)


@pytest.mark.asyncio
async def test_transact_dht_records_set_drop_get(api_connection: VeilidAPI):
    rc = await api_connection.new_routing_context()
    async with rc:
        for kind in await api_connection.valid_crypto_kinds():
            rec = await rc.create_dht_record(kind, DHTSchema.dflt(2))

            rec_tx = await api_connection.transact_dht_records([rec.key], None)
            async with rec_tx:
                vd = await rec_tx.set(rec.key, ValueSubkey(0), TEST_MESSAGE_1)
                assert vd is None
                # Drop rec_tx

            vd2 = await rc.get_dht_value(rec.key, ValueSubkey(0), True)
            assert vd2 is None

            await rc.close_dht_record(rec.key)
            await rc.delete_dht_record(rec.key)

@pytest.mark.asyncio
async def test_transact_dht_records_set_drop_use_dead(api_connection: VeilidAPI):
    rc = await api_connection.new_routing_context()
    async with rc:
        for kind in await api_connection.valid_crypto_kinds():
            rec = await rc.create_dht_record(kind, DHTSchema.dflt(2))

            rec_tx = await api_connection.transact_dht_records([rec.key], None)
            async with rec_tx:
                vd = await rec_tx.set(rec.key, ValueSubkey(0), TEST_MESSAGE_1)
                assert vd is None

            vd2 = await rc.get_dht_value(rec.key, ValueSubkey(0), True)
            assert vd2 is None

            await rc.close_dht_record(rec.key)
            await rc.delete_dht_record(rec.key)


@pytest.mark.asyncio
async def test_transact_dht_records_wrong_set(api_connection: VeilidAPI):
    rc = await api_connection.new_routing_context()
    async with rc:
        for kind in await api_connection.valid_crypto_kinds():
            rec = await rc.create_dht_record(kind, DHTSchema.dflt(2))

            rec_tx = await api_connection.transact_dht_records([rec.key], None)
            async with rec_tx:
                with pytest.raises(VeilidAPIError):
                    vd = await rc.set_dht_value(rec.key, ValueSubkey(0), TEST_MESSAGE_1)

            await rc.close_dht_record(rec.key)
            await rc.delete_dht_record(rec.key)


@pytest.mark.asyncio
async def test_transact_dht_records_set_commit_get_commit(api_connection: VeilidAPI):
    rc = await api_connection.new_routing_context()
    async with rc:
        for kind in await api_connection.valid_crypto_kinds():
            rec = await rc.create_dht_record(kind, DHTSchema.dflt(2))

            rec_tx = await api_connection.transact_dht_records([rec.key], None)
            async with rec_tx:
                vd = await rec_tx.set(rec.key, ValueSubkey(0), TEST_MESSAGE_1)
                assert vd is None
                await rec_tx.commit()

            rec_tx = await api_connection.transact_dht_records([rec.key], None)
            async with rec_tx:
                vd2 = await rec_tx.get(rec.key, ValueSubkey(0))
                assert vd2 is not None
                await rec_tx.commit()

            assert vd2.data == TEST_MESSAGE_1

            vd3 = await rc.get_dht_value(rec.key, ValueSubkey(0), False)
            assert vd3 is not None and vd3.data == TEST_MESSAGE_1

            await rc.close_dht_record(rec.key)
            await rc.delete_dht_record(rec.key)


@pytest.mark.asyncio
async def test_transact_dht_records_set_commit_delete_get_rollback(api_connection: VeilidAPI):
    rc = await api_connection.new_routing_context()
    async with rc:
        for kind in await api_connection.valid_crypto_kinds():
            rec = await rc.create_dht_record(kind, DHTSchema.dflt(2))

            # Set value transactionally
            rec_tx = await api_connection.transact_dht_records([rec.key], None)
            async with rec_tx:
                vd = await rec_tx.set(rec.key, ValueSubkey(0), TEST_MESSAGE_1)
                assert vd is None
                await rec_tx.commit()

            # Delete it locally
            await rc.close_dht_record(rec.key)
            await rc.delete_dht_record(rec.key)

            # Reopen the record readonly
            rec = await rc.open_dht_record(rec.key)

            # Get the value transactionally but do not commit locally
            rec_tx = await api_connection.transact_dht_records([rec.key], None)
            async with rec_tx:
                vd2 = await rec_tx.get(rec.key, ValueSubkey(0))
                assert vd2 is not None and vd2.data == TEST_MESSAGE_1
                await rec_tx.rollback()

            # Should not have committed the get result locally due to rollback
            report1 = await rc.inspect_dht_record(rec.key, [], DHTReportScope.LOCAL)
            assert report1.local_seqs == [None, None]

            await rc.close_dht_record(rec.key)
            await rc.delete_dht_record(rec.key)

            # Reopen the record readonly
            rec = await rc.open_dht_record(rec.key)

            # Should get transactionally set value from online
            vd3 = await rc.get_dht_value(rec.key, ValueSubkey(0))
            assert vd3 is not None and vd3.data == TEST_MESSAGE_1

            await rc.close_dht_record(rec.key)
            await rc.delete_dht_record(rec.key)


@pytest.mark.skipif(os.getenv("INTEGRATION") != "1", reason="integration test requires two servers running")
@pytest.mark.asyncio
async def test_dht_transaction_integration_writer_reader_fail_begin():

    async def null_update_callback(update: veilid.VeilidUpdate):
        pass

    try:
        api0 = await veilid.api_connector(null_update_callback, 0)
    except veilid.VeilidConnectionError:
        pytest.skip("Unable to connect to veilid-server 0.")
        return

    try:
        api1 = await veilid.api_connector(null_update_callback, 1)
    except veilid.VeilidConnectionError:
        pytest.skip("Unable to connect to veilid-server 1.")
        return

    async with api0, api1:
        # purge local and remote record stores to ensure we start fresh
        await api0.debug("record purge local")
        await api0.debug("record purge remote")
        await api1.debug("record purge local")
        await api1.debug("record purge remote")

        # make routing contexts
        rc0 = await api0.new_routing_context()
        rc1 = await api1.new_routing_context()
        async with rc0, rc1:
            for kind in await api0.valid_crypto_kinds():

                # create a record on server 0
                rec0 = await rc0.create_dht_record(kind, DHTSchema.dflt(2))

                # start dht record transaction on server 0
                rec0_tx = await api0.transact_dht_records([rec0.key], None)
                async with rec0_tx:
                    # write subkey 0
                    vd = await rec0_tx.set(rec0.key, ValueSubkey(0), b"AAA")
                    assert vd is None

                    # commit
                    await rec0_tx.commit()

                # start another dht record transaction on server 0
                rec0_tx = await api0.transact_dht_records([rec0.key], None)
                async with rec0_tx:

                    # write subkey 0 on server 0
                    vd = await rec0_tx.set(rec0.key, ValueSubkey(0), b"BBB")
                    assert vd is None

                    # open dht record on server 1
                    rec1 = await rc1.open_dht_record(rec0.key, rec0.owner_key_pair())

                    # Try to transact with the same member keypair a second time and it will fail
                    with pytest.raises(VeilidAPIError):
                        await api1.transact_dht_records([rec1.key], None)

                    # commit on server 0
                    await rec0_tx.commit()

                # start dht record transaction on server 0
                rec0_tx = await api0.transact_dht_records([rec0.key], None)
                async with rec0_tx:
                    # read subkey 0
                    vd = await rec0_tx.get(rec0.key, ValueSubkey(0))
                    assert vd is not None and vd.data == b"BBB"

                    # commit
                    await rec0_tx.rollback()

                await rc0.close_dht_record(rec0.key)
                await rc0.delete_dht_record(rec0.key)
                await rc1.close_dht_record(rec1.key)
                await rc1.delete_dht_record(rec1.key)

@pytest.mark.skipif(os.getenv("INTEGRATION") != "1", reason="integration test requires two servers running")
@pytest.mark.asyncio
async def test_dht_transaction_integration_writer_reader_fail_commit():

    async def null_update_callback(update: veilid.VeilidUpdate):
        pass

    try:
        api0 = await veilid.api_connector(null_update_callback, 0)
    except veilid.VeilidConnectionError:
        pytest.skip("Unable to connect to veilid-server 0.")
        return

    try:
        api1 = await veilid.api_connector(null_update_callback, 1)
    except veilid.VeilidConnectionError:
        pytest.skip("Unable to connect to veilid-server 1.")
        return

    async with api0, api1:
        # purge local and remote record stores to ensure we start fresh
        await api0.debug("record purge local")
        await api0.debug("record purge remote")
        await api1.debug("record purge local")
        await api1.debug("record purge remote")

        # make routing contexts
        rc0 = await api0.new_routing_context()
        rc1 = await api1.new_routing_context()
        async with rc0, rc1:
            for kind in await api0.valid_crypto_kinds():

                # create two keypairs
                cs = await api0.get_crypto_system(kind)
                async with cs:
                    writer0 = await cs.generate_key_pair()
                    writer1 = await cs.generate_key_pair()

                # create a DHT schema with the two members
                member0 = await api0.generate_member_id(writer0.key())
                member1 = await api0.generate_member_id(writer1.key())
                schema = DHTSchema.smpl(0, [DHTSchemaSMPLMember(member0.value(), 1), DHTSchemaSMPLMember(member1.value(), 1)])

                # create a record on server 0 and reopen with writer
                rec0 = await rc0.create_dht_record(kind, schema)
                rec0 = await rc0.open_dht_record(rec0.key, writer0)

                # start dht record transaction on server 0
                rec0_tx = await api0.transact_dht_records([rec0.key], None)
                async with rec0_tx:
                    # write subkey 0
                    vd = await rec0_tx.set(rec0.key, ValueSubkey(0), b"AAA")
                    assert vd is None

                    # commit
                    await rec0_tx.commit()

                # start another dht record transaction on server 0
                rec0_tx = await api0.transact_dht_records([rec0.key], None)
                async with rec0_tx:

                    # write subkey 0 on server 0
                    vd = await rec0_tx.set(rec0.key, ValueSubkey(0), b"BBB")
                    assert vd is None

                    # open dht record on server 1
                    rec1 = await rc1.open_dht_record(rec0.key, writer1)

                    # start transaction on server 1 using second member
                    rec1_tx = await api1.transact_dht_records([rec1.key], None)
                    async with rec1_tx:

                        # write subkey 1 on server 1
                        vd = await rec1_tx.set(rec1.key, ValueSubkey(1), b"CCC")
                        assert vd is None

                        # commit on server 1
                        await rec1_tx.commit()

                    # commit on server 0 should fail because snapshots no longer match
                    with pytest.raises(VeilidAPIError):
                        await rec0_tx.commit()

                # start dht record transaction on server 0
                rec0_tx = await api0.transact_dht_records([rec0.key], None)
                async with rec0_tx:
                    # read subkey 1
                    vd = await rec0_tx.get(rec0.key, ValueSubkey(1))
                    assert vd is not None and vd.data == b"CCC"

                    # commit
                    await rec0_tx.rollback()

                await rc0.close_dht_record(rec0.key)
                await rc0.delete_dht_record(rec0.key)
                await rc1.close_dht_record(rec1.key)
                await rc1.delete_dht_record(rec1.key)


@pytest.mark.skipif(os.getenv("STRESS") != "1", reason="stress test takes a long time")
@pytest.mark.asyncio
async def test_dht_transaction_write_read_full_subkeys():

    async def null_update_callback(update: VeilidUpdate):
        pass

    try:
        api0 = await api_connector(null_update_callback, 0)
    except VeilidConnectionError:
        pytest.skip("Unable to connect to veilid-server 0.")
        return

    async with api0:
        # purge local and remote record stores to ensure we start fresh
        await api0.debug("record purge local")
        await api0.debug("record purge remote")

        # make routing contexts
        rc0 = await (await api0.new_routing_context()).with_sequencing(Sequencing.ENSURE_ORDERED)
        async with rc0:

            for kind in await api0.valid_crypto_kinds():
                print(f"kind: {kind}")
                cs = await api0.get_crypto_system(kind)
                async with cs:

                    # Number of records
                    COUNT = 8
                    # Number of subkeys per record
                    SUBKEY_COUNT = 32
                    # BareNonce to encrypt test data
                    NONCE = Nonce.from_bytes(b"A"*await cs.nonce_length())
                    # Secret to encrypt test data
                    SECRET = SharedSecret.from_value(await cs.kind(), BareSharedSecret.from_bytes(b"A"*await cs.shared_secret_length()))
                    # Max subkey size
                    MAX_SUBKEY_SIZE = min(32768, 1024*1024//SUBKEY_COUNT)
                    # MAX_SUBKEY_SIZE = 256

                    # write dht records on server 0
                    records : list[DHTRecordDescriptor] = []
                    subkey_data_list : list[bytes] = []
                    schema = DHTSchema.dflt(SUBKEY_COUNT)
                    print(f'writing {COUNT} records with full subkeys')
                    for n in range(COUNT):
                        desc = await rc0.create_dht_record(kind, schema)
                        print(f'  {n}: {desc.key} {desc.owner}:{desc.owner_secret}')
                        records.append(desc)

                        # Make encrypted data that is consistent and hard to compress
                        subkey_data = bytes(chr(ord("A")+n%32)*MAX_SUBKEY_SIZE, 'ascii')
                        subkey_data = await cs.crypt_no_auth(subkey_data, NONCE, SECRET)
                        subkey_data_list.append(subkey_data)

                    start = time.time()
                    transaction = await api0.transact_dht_records([x.key for x in records], None)
                    print(f'transaction begin: {time.time()-start}')

                    for i in range(SUBKEY_COUNT):
                        start = time.time()

                        init_set_futures : set[Coroutine[Any, Any, ValueData | None]] = set()

                        for n in range(COUNT):
                            key = records[n].key
                            subkey_data = subkey_data_list[n]
                            init_set_futures.add(transaction.set(key, ValueSubkey(i), subkey_data))

                        # Update each subkey for each record in parallel
                        # This ensures that each record gets its own expiration update
                        await asyncio.gather(*init_set_futures)

                        print(f'transaction set subkey {i}: {time.time()-start}')


                    start = time.time()
                    await transaction.commit()
                    print(f'transaction commit: {time.time()-start}')

                    for desc in records:
                        await rc0.close_dht_record(desc.key)

                    await api0.debug("record purge local")
                    await api0.debug("record purge remote")

                    # read dht records on server 0
                    print(f'reading {COUNT} records')

                    for desc in records:
                        await rc0.open_dht_record(desc.key)

                    start = time.time()
                    transaction = await api0.transact_dht_records([x.key for x in records], None)
                    print(f'transaction begin: {time.time()-start}')

                    for i in range(SUBKEY_COUNT):
                        start = time.time()
                        subkey = ValueSubkey(i)

                        init_get_futures : set[Coroutine[Any, Any, tuple[RecordKey, ValueSubkey, bytes, ValueData | None]]] = set()

                        for n in range(COUNT):
                            key = records[n].key
                            subkey_data = subkey_data_list[n]

                            async def getter(key: RecordKey, subkey: ValueSubkey, check_data: bytes):
                                return (key, subkey, check_data, await transaction.get(key, subkey))

                            init_get_futures.add(getter(key, subkey, subkey_data))

                        # Get each subkey for each record in parallel
                        # This ensures that each record gets its own expiration update
                        get_results = await asyncio.gather(*init_get_futures)
                        for key, sk, check_data, vd in get_results:
                            assert vd is not None and vd.data == check_data

                        print(f'transaction get subkey {i}: {time.time()-start}')

                    await transaction.rollback()

                    for desc in records:
                        await rc0.close_dht_record(desc.key)


@pytest.mark.skipif(os.getenv("STRESS") != "1", reason="stress test takes a long time")
@pytest.mark.asyncio
async def test_dht_transaction_write_read_full_records_serial():

    async def null_update_callback(update: VeilidUpdate):
        pass

    try:
        api0 = await api_connector(null_update_callback, 0)
    except VeilidConnectionError:
        pytest.skip("Unable to connect to veilid-server 0.")
        return

    async with api0:
        # purge local and remote record stores to ensure we start fresh
        await api0.debug("record purge local")
        await api0.debug("record purge remote")

        # make routing contexts
        rc0 = await (await api0.new_routing_context()).with_sequencing(Sequencing.ENSURE_ORDERED)
        async with rc0:

            for kind in await api0.valid_crypto_kinds():
                print(f"kind: {kind}")
                cs = await api0.get_crypto_system(kind)
                async with cs:

                    # Number of records
                    COUNT = 8
                    # Number of subkeys per record
                    SUBKEY_COUNT = 32
                    # BareNonce to encrypt test data
                    NONCE = Nonce.from_bytes(b"A"*await cs.nonce_length())
                    # Secret to encrypt test data
                    SECRET = SharedSecret.from_value(await cs.kind(), BareSharedSecret.from_bytes(b"A"*await cs.shared_secret_length()))
                    # Max subkey size
                    MAX_SUBKEY_SIZE = min(32768, 1024*1024//SUBKEY_COUNT)
                    # MAX_SUBKEY_SIZE = 256
                    # Concurrency limit for subkeys within a transaction
                    CONCURRENCY_LIMIT = 8

                    # write dht records on server 0
                    records : list[DHTRecordDescriptor] = []
                    subkey_data_list : list[bytes] = []
                    schema = DHTSchema.dflt(SUBKEY_COUNT)
                    print(f'writing {COUNT} records with full subkeys')
                    for n in range(COUNT):
                        desc = await rc0.create_dht_record(kind, schema)
                        print(f'  {n}: {desc.key} {desc.owner}:{desc.owner_secret}')
                        records.append(desc)

                        # Make encrypted data that is consistent and hard to compress
                        subkey_data = bytes(chr(ord("A")+n%32)*MAX_SUBKEY_SIZE, 'ascii')
                        subkey_data = await cs.crypt_no_auth(subkey_data, NONCE, SECRET)
                        subkey_data_list.append(subkey_data)

                    for n in range(COUNT):
                        start = time.time()
                        transaction = await api0.transact_dht_records([records[n].key], None)
                        print(f'transaction {n} begin: {time.time()-start}')

                        semaphore = asyncio.Semaphore(CONCURRENCY_LIMIT)

                        key = records[n].key
                        subkey_data = subkey_data_list[n]

                        init_set_futures : set[Coroutine[Any, Any, ValueData | None]] = set()

                        async def setter(key: RecordKey, subkey: ValueSubkey, subkey_data: bytes):
                            async with semaphore:
                                subkey_start = time.time()
                                print(f'subkey {subkey} start time offset: {subkey_start-start}')

                                cnt = 0
                                while True:
                                    try:
                                        res = await transaction.set(key, subkey, subkey_data)
                                        break
                                    except veilid.VeilidAPIErrorTryAgain:
                                        cnt += 1
                                        print(f'  retry #{cnt} setting {key} #{subkey}')
                                        continue
                                
                                subkey_finish = time.time()
                                print(f'subkey {subkey} finish time offset: {subkey_finish-start}, duration: {subkey_finish-subkey_start}')
                                return res

                        for i in range(SUBKEY_COUNT):
                            start = time.time()
                            init_set_futures.add(setter(key, ValueSubkey(i), subkey_data))

                        # Update each subkey for each record serially
                        # This stress tests record keepalives
                        await asyncio.gather(*init_set_futures)

                        print(f'transaction set record {n}: {time.time()-start}')

                        start = time.time()
                        await transaction.commit()
                        print(f'transaction commit: {time.time()-start}')

                    for desc in records:
                        await rc0.close_dht_record(desc.key)

                    await api0.debug("record purge local")
                    await api0.debug("record purge remote")

                    # read dht records on server 0
                    print(f'reading {COUNT} records')

                    for desc in records:
                        await rc0.open_dht_record(desc.key)

                    start = time.time()
                    print(f'transaction begin: {time.time()-start}')

                    for n in range(COUNT):
                        key = records[n].key
                        transaction = await api0.transact_dht_records([records[n].key], None)
                        subkey_data = subkey_data_list[n]

                        init_get_futures : set[Coroutine[Any, Any, tuple[RecordKey, ValueSubkey, bytes, ValueData | None]]] = set()
                        semaphore = asyncio.Semaphore(CONCURRENCY_LIMIT)

                        for i in range(SUBKEY_COUNT):
                            start = time.time()
                            subkey = ValueSubkey(i)

                            async def getter(key: RecordKey, subkey: ValueSubkey, check_data: bytes):
                                async with semaphore:
                                    subkey_start = time.time()
                                    print(f'subkey {subkey} start time offset: {subkey_start-start}')

                                    cnt = 0
                                    while True:
                                        try:
                                            res = await transaction.get(key, subkey)
                                            break
                                        except veilid.VeilidAPIErrorTryAgain:
                                            cnt += 1
                                            print(f'  retry #{cnt} setting {key} #{subkey}')
                                            continue

                                    subkey_finish = time.time()
                                    print(f'subkey {subkey} finish time offset: {subkey_finish-start}, duration: {subkey_finish-subkey_start}')
                                    return (key, subkey, check_data, res)

                            init_get_futures.add(getter(key, subkey, subkey_data))

                        # Get each subkey for each record serially
                        # This stress tests record keepalives
                        get_results = await asyncio.gather(*init_get_futures)
                        for key, sk, check_data, vd in get_results:
                            assert vd is not None and vd.data == check_data

                        print(f'transaction get record {n}: {time.time()-start}')

                        start = time.time()
                        await transaction.rollback()
                        print(f'transaction rollback: {time.time()-start}')

                    for desc in records:
                        await rc0.close_dht_record(desc.key)



@pytest.mark.skipif(os.getenv("STRESS") != "1", reason="stress test takes a long time")
@pytest.mark.asyncio
async def test_dht_transaction_write_read_full_records_parallel():

    async def null_update_callback(update: VeilidUpdate):
        pass

    try:
        api0 = await api_connector(null_update_callback, 0)
    except VeilidConnectionError:
        pytest.skip("Unable to connect to veilid-server 0.")
        return

    async with api0:
        # purge local and remote record stores to ensure we start fresh
        await api0.debug("record purge local")
        await api0.debug("record purge remote")

        # make routing contexts
        rc0 = await (await api0.new_routing_context()).with_sequencing(Sequencing.ENSURE_ORDERED)
        async with rc0:

            for kind in await api0.valid_crypto_kinds():
                print(f"kind: {kind}")
                cs = await api0.get_crypto_system(kind)
                async with cs:

                    # Number of records
                    COUNT = 8
                    # Number of subkeys per record
                    SUBKEY_COUNT = 32
                    # BareNonce to encrypt test data
                    NONCE = Nonce.from_bytes(b"A"*await cs.nonce_length())
                    # Secret to encrypt test data
                    SECRET = SharedSecret.from_value(await cs.kind(), BareSharedSecret.from_bytes(b"A"*await cs.shared_secret_length()))
                    # Max subkey size
                    MAX_SUBKEY_SIZE = min(32768, 1024*1024//SUBKEY_COUNT)
                    # MAX_SUBKEY_SIZE = 256

                    # write dht records on server 0
                    records : list[DHTRecordDescriptor] = []
                    subkey_data_list : list[bytes] = []
                    schema = DHTSchema.dflt(SUBKEY_COUNT)
                    print(f'writing {COUNT} records with full subkeys')
                    for n in range(COUNT):
                        desc = await rc0.create_dht_record(kind, schema)
                        print(f'  {n}: {desc.key} {desc.owner}:{desc.owner_secret}')
                        records.append(desc)

                        # Make encrypted data that is consistent and hard to compress
                        subkey_data = bytes(chr(ord("A")+n%32)*MAX_SUBKEY_SIZE, 'ascii')
                        subkey_data = await cs.crypt_no_auth(subkey_data, NONCE, SECRET)
                        subkey_data_list.append(subkey_data)

                    start = time.time()
                    transaction = await api0.transact_dht_records([x.key for x in records], None)
                    print(f'transaction begin: {time.time()-start}')

                    for n in range(COUNT):
                        key = records[n].key
                        subkey_data = subkey_data_list[n]

                        init_set_futures : set[Coroutine[Any, Any, ValueData | None]] = set()

                        for i in range(SUBKEY_COUNT):
                            start = time.time()

                            init_set_futures.add(transaction.set(key, ValueSubkey(i), subkey_data))

                        # Update each subkey for each record serially
                        # This stress tests record keepalives
                        await asyncio.gather(*init_set_futures)

                        print(f'transaction set record {n}: {time.time()-start}')


                    start = time.time()
                    await transaction.commit()
                    print(f'transaction commit: {time.time()-start}')

                    for desc in records:
                        await rc0.close_dht_record(desc.key)

                    await api0.debug("record purge local")
                    await api0.debug("record purge remote")

                    # read dht records on server 0
                    print(f'reading {COUNT} records')

                    for desc in records:
                        await rc0.open_dht_record(desc.key)

                    start = time.time()
                    transaction = await api0.transact_dht_records([x.key for x in records], None)
                    print(f'transaction begin: {time.time()-start}')

                    for n in range(COUNT):
                        key = records[n].key
                        subkey_data = subkey_data_list[n]

                        init_get_futures : set[Coroutine[Any, Any, tuple[RecordKey, ValueSubkey, bytes, ValueData | None]]] = set()

                        for i in range(SUBKEY_COUNT):
                            start = time.time()
                            subkey = ValueSubkey(i)

                            async def getter(key: RecordKey, subkey: ValueSubkey, check_data: bytes):
                                return (key, subkey, check_data, await transaction.get(key, subkey))

                            init_get_futures.add(getter(key, subkey, subkey_data))

                        # Get each subkey for each record serially
                        # This stress tests record keepalives
                        get_results = await asyncio.gather(*init_get_futures)
                        for key, sk, check_data, vd in get_results:
                            assert vd is not None and vd.data == check_data

                        print(f'transaction get record {n}: {time.time()-start}')

                    await transaction.rollback()

                    for desc in records:
                        await rc0.close_dht_record(desc.key)



@pytest.mark.skipif(os.getenv("STRESS") != "1", reason="stress test takes a long time")
@pytest.mark.asyncio
async def test_dht_transaction_write_read_full_parallel():

    async def null_update_callback(update: VeilidUpdate):
        pass

    try:
        api0 = await api_connector(null_update_callback, 0)
    except VeilidConnectionError:
        pytest.skip("Unable to connect to veilid-server 0.")
        return

    async with api0:
        # purge local and remote record stores to ensure we start fresh
        await api0.debug("record purge local")
        await api0.debug("record purge remote")

        # make routing contexts
        rc0 = await (await api0.new_routing_context()).with_sequencing(Sequencing.ENSURE_ORDERED)
        async with rc0:

            for kind in await api0.valid_crypto_kinds():
                print(f"kind: {kind}")
                cs = await api0.get_crypto_system(kind)
                async with cs:

                    # Number of records
                    COUNT = 32
                    # Number of subkeys per record
                    SUBKEY_COUNT = 32
                    # Number of subkeys to batch
                    SUBKEY_BATCH = int(os.getenv("SUBKEY_BATCH") or "4")
                    # BareNonce to encrypt test data
                    NONCE = Nonce.from_bytes(b"A"*await cs.nonce_length())
                    # Secret to encrypt test data
                    SECRET = SharedSecret.from_value(await cs.kind(), BareSharedSecret.from_bytes(b"A"*await cs.shared_secret_length()))
                    # Max subkey size
                    MAX_SUBKEY_SIZE = min(32768, 1024*1024//SUBKEY_COUNT)
                    # MAX_SUBKEY_SIZE = 256

                    # write dht records on server 0
                    records : list[DHTRecordDescriptor] = []
                    subkey_data_list : list[bytes] = []
                    schema = DHTSchema.dflt(SUBKEY_COUNT)

                    print(f'writing {COUNT} records with full subkeys')
                    for n in range(COUNT):
                        desc = await rc0.create_dht_record(kind, schema)
                        print(f'  {n}: {desc.key} {desc.owner}:{desc.owner_secret}')
                        records.append(desc)

                        # Make encrypted data that is consistent and hard to compress
                        subkey_data = bytes(chr(ord("A")+n%32)*MAX_SUBKEY_SIZE, 'ascii')
                        subkey_data = await cs.crypt_no_auth(subkey_data, NONCE, SECRET)
                        subkey_data_list.append(subkey_data)

                    t1start = time.time()
                    transaction = await api0.transact_dht_records([x.key for x in records], None)
                    print(f'transaction begin: {time.time()-t1start}')

                    for i in range(0,SUBKEY_COUNT, SUBKEY_BATCH):
                        init_set_futures : set[Coroutine[Any, Any, ValueData | None]] = set()
                        for j in range(SUBKEY_BATCH):
                            subkey = ValueSubkey(i+j)
                            for n in range(COUNT):
                                key = records[n].key
                                subkey_data = subkey_data_list[n]

                                async def setter(key: RecordKey, subkey: ValueSubkey, data: bytes):
                                    # start = time.time()
                                    cnt = 0
                                    while True:
                                        try:
                                            await transaction.set(key, subkey, data)
                                            break
                                        except veilid.VeilidAPIErrorTryAgain:
                                            cnt += 1
                                            print(f'  retry #{cnt} setting {key} #{subkey}')
                                            continue
                                    # print(f'set {key} #{subkey}: {time.time()-start}')

                                init_set_futures.add(setter(key, subkey, subkey_data))

                        # Update all subkeys for all records simultaneously
                        start = time.time()
                        await asyncio.gather(*init_set_futures)
                        print(f'transaction set subkeys {i}-{i+SUBKEY_BATCH-1}: {time.time()-start}')

                    start = time.time()
                    await transaction.commit()
                    print(f'transaction commit: {time.time()-start}')

                    for desc in records:
                        await rc0.close_dht_record(desc.key)

                    await api0.debug("record purge local")
                    await api0.debug("record purge remote")

                    # read dht records on server 0
                    print(f'reading {COUNT} records')

                    for desc in records:
                        await rc0.open_dht_record(desc.key)

                    t2start = time.time()
                    transaction = await api0.transact_dht_records([x.key for x in records], None)
                    print(f'transaction begin: {time.time()-t2start}')

                    for i in range(0,SUBKEY_COUNT, SUBKEY_BATCH):
                        init_get_futures : set[Coroutine[Any, Any, tuple[RecordKey, ValueSubkey, bytes, ValueData | None]]] = set()
                        for j in range(SUBKEY_BATCH):
                            subkey = ValueSubkey(i+j)

                            for n in range(COUNT):
                                key = records[n].key
                                subkey_data = subkey_data_list[n]

                                async def getter(key: RecordKey, subkey: ValueSubkey, check_data: bytes):
                                    #start = time.time()
                                    out = (key, subkey, check_data, await transaction.get(key, subkey))
                                    # print(f'get {key} #{subkey}: {time.time()-start}')
                                    return out

                                init_get_futures.add(getter(key, subkey, subkey_data))

                        # Update each subkey for each record in parallel
                        # This ensures that each record gets its own expiration update
                        start = time.time()
                        get_results = await asyncio.gather(*init_get_futures)
                        for key, sk, check_data, vd in get_results:
                            assert vd is not None and vd.data == check_data
                        print(f'transaction get subkeys {i}-{i+SUBKEY_BATCH-1}: {time.time()-start}')

                    await transaction.rollback()
                    print(f'done: {time.time()-t1start}')

                    for desc in records:
                        await rc0.close_dht_record(desc.key)



@pytest.mark.skipif(os.getenv("FILLDHT") is None, reason="fill disk test disabled")
@pytest.mark.asyncio
async def test_dht_fill_dht_transact():

    async def null_update_callback(update: VeilidUpdate):
        pass

    try:
        api0 = await api_connector(null_update_callback, 0)
    except VeilidConnectionError:
        pytest.skip("Unable to connect to veilid-server 0.")
        return

    async with api0:

        # make routing contexts
        rc0 = await (await api0.new_routing_context()).with_sequencing(Sequencing.ENSURE_ORDERED)
        async with rc0:

            kind = (await api0.valid_crypto_kinds())[0]
            cs = await api0.get_crypto_system(kind)
            async with cs:

                mbcount = int(os.getenv("FILLDHT") or "0")
                for mbn in range(0, mbcount):

                    # Number of records
                    COUNT = 1
                    # Number of subkeys per record
                    SUBKEY_COUNT = 32
                    # BareNonce to encrypt test data
                    NONCE = Nonce.from_bytes(b"A"*await cs.nonce_length())
                    # Secret to encrypt test data
                    SECRET = SharedSecret.from_value(await cs.kind(), BareSharedSecret.from_bytes(b"A"*await cs.shared_secret_length()))
                    # Max subkey size
                    MAX_SUBKEY_SIZE = min(32768, 1024*1024//SUBKEY_COUNT)
                    # MAX_SUBKEY_SIZE = 256

                    # write dht records on server 0
                    records : list[DHTRecordDescriptor] = []
                    subkey_data_list : list[bytes] = []
                    schema = DHTSchema.dflt(SUBKEY_COUNT)
                    for n in range(COUNT):
                        desc = await rc0.create_dht_record(kind, schema)
                        records.append(desc)

                        # Make encrypted data that is consistent and hard to compress
                        subkey_data = bytes(chr(ord("A")+n%32)*MAX_SUBKEY_SIZE, 'ascii')
                        subkey_data = await cs.crypt_no_auth(subkey_data, NONCE, SECRET)
                        subkey_data_list.append(subkey_data)

                    start = time.time()
                    transaction = await api0.transact_dht_records([x.key for x in records], None)

                    init_set_futures : set[Coroutine[Any, Any, ValueData | None]] = set()

                    for i in range(SUBKEY_COUNT):
                        for n in range(COUNT):
                            key = records[n].key
                            subkey_data = subkey_data_list[n]

                            async def setter(key: RecordKey, subkey: ValueSubkey, data: bytes):
                                start = time.time()
                                cnt = 0
                                while True:
                                    try:
                                        await transaction.set(key, subkey, data)
                                        break
                                    except veilid.VeilidAPIErrorTryAgain:
                                        cnt += 1
                                        continue

                            init_set_futures.add(setter(key, ValueSubkey(i), subkey_data))

                    # Update all subkeys for all records simultaneously
                    start = time.time()
                    await asyncio.gather(*init_set_futures)

                    await transaction.commit()

                    print(f"record {mbn}: {time.time()-start}")

                    for desc in records:
                        await rc0.close_dht_record(desc.key)

