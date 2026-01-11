import { useState } from 'react';
import { veilidCrypto, DHTSchema } from 'veilid-wasm';

import { getRoutingContext } from '../veilid/veilid-core';

async function dhtStressTest() {
    const routingContext = getRoutingContext();

    const recordCount = 30;
    const subkeyCount = 32;
    const inspectCount = 1;

    // Create a 32KB data buffer
    const dataSize = 32 * 1024; // 32KB in bytes
    const dataArray = new Uint8Array(dataSize);

    // Fill the array with some pattern (using values 0-255 repeating)
    for (let i = 0; i < dataSize; i++) {
        dataArray[i] = i % 256;
    }

    let a = Array();
    for (let r = 0; r < recordCount; r++) {
        const schema: DHTSchema = {DFLT: {oCnt: subkeyCount}};

        const dhtRecord = await routingContext.createDHTRecord(veilidCrypto.CRYPTO_KIND_VLD0, schema)

        // Set all subkeys
        for (let n = 0; n < subkeyCount; n++) {
            a.push((async () => {
                const measureName = `${r}-setDhtValue-${n}`;

                performance.mark(measureName + "-start")
                await routingContext.setDHTValue(
                    dhtRecord.key,
                    n,
                    dataArray,
                );

                performance.measure(measureName, measureName + "-start")
            })());
        }

        // Inspect all records N times while sets are happening
        for (let n = 0; n < inspectCount; n++) {
            a.push((async () => {
                const measureName = `${r}-inspectDhtRecord-${n}`;

                performance.mark(measureName + "-start")
                await routingContext.inspectDHTRecord(
                    dhtRecord.key,
                    null,
                    "SyncSet",
                );

                performance.measure(measureName, measureName + "-start")
            })());
        }
    }

    // Wait for all results
    await Promise.all(a)
}


export function DhtStressTest() {
    const [isRunning, setIsRunning] = useState(false);

    return (
        <button onClick={() => {
            if (isRunning) {
                return;
            }
            setIsRunning(true);
            dhtStressTest().finally(() => {
                setIsRunning(false);
            });
        }} disabled={isRunning}>
            {isRunning ? 'Running...' : 'Run DHT Stress Test'}
        </button>
    )
}   