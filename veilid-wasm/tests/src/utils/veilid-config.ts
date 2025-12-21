import { VeilidWASMConfig, veilidClient, } from 'veilid-wasm';

export const DEBUGGING = process.env.DEBUG == "1" || process.env.DEBUG == "true";

export const veilidCoreInitConfig: VeilidWASMConfig = {
  logging: {
    api: {
      enabled: !DEBUGGING,
      level: "Info",
      ignoreLogTargets: [],
    },
    performance: {
      enabled: DEBUGGING,
      level: DEBUGGING ? "Debug" : "Info",
      logsInTimings: false,
      logsInConsole: DEBUGGING ? "NoColor" : "Off",
      ignoreLogTargets: DEBUGGING ? ["-veilid_api"] : [""],
    },
  },
};

export const veilidCoreStartupConfig = (() => {
  // console.log("starting config")
  const defaultConfig = veilidClient.defaultConfig();
  defaultConfig.programName = 'veilid-wasm-test';
  if (process.env.NETWORK_KEY) {
    defaultConfig.network.networkKeyPassword = process.env.NETWORK_KEY;
  }
  if (process.env.BOOTSTRAP_KEYS) {
    defaultConfig.network.routingTable.bootstrapKeys = process.env.BOOTSTRAP_KEYS.split(',')
  }
  if (process.env.BOOTSTRAP) {
    defaultConfig.network.routingTable.bootstrap = process.env.BOOTSTRAP.split(',');
  }
  // Ensure we are starting from scratch
  defaultConfig.tableStore.delete = true;
  defaultConfig.protectedStore.delete = true;
  defaultConfig.blockStore.delete = true;

  // Tests should not participate in heavy server operations
  defaultConfig.capabilities.disable = [veilidClient.VEILID_CAPABILITY_DHT, veilidClient.VEILID_CAPABILITY_ROUTE];
  // console.log("ending config")

  return defaultConfig;
})(); 
