import { veilidClient, VeilidWASMConfig } from 'veilid-wasm';
export interface VeilidConfigOptions {
    namespace: string;
    password: string;
}

export const veildCoreInitConfig: VeilidWASMConfig = {
    logging: {
        api: {
            enabled: true,
            level: 'Info',
            ignoreLogTargets: [],
        },
        performance: {
            enabled: false,
            level: 'Info',
            logsInTimings: false,
            logsInConsole: "Off",
            ignoreLogTargets: [],
        },
    },
};

export function getVeilidCoreStartupConfig(options: VeilidConfigOptions) {
    const defaultConfig = veilidClient.defaultConfig();

    defaultConfig.programName = 'veilid-wasm-test-bench';
    defaultConfig.namespace = options.namespace;
    defaultConfig.protectedStore.deviceEncryptionKeyPassword = options.password

    return defaultConfig;
}
