// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'veilid_config.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_VeilidFFIConfigLoggingTerminal _$VeilidFFIConfigLoggingTerminalFromJson(
  Map<String, dynamic> json,
) => _VeilidFFIConfigLoggingTerminal(
  enabled: json['enabled'] as bool,
  level: VeilidConfigLogLevel.fromJson(json['level']),
  ignoreLogTargets:
      (json['ignoreLogTargets'] as List<dynamic>?)
          ?.map((e) => e as String)
          .toList() ??
      const [],
);

Map<String, dynamic> _$VeilidFFIConfigLoggingTerminalToJson(
  _VeilidFFIConfigLoggingTerminal instance,
) => <String, dynamic>{
  'enabled': instance.enabled,
  'level': instance.level.toJson(),
  'ignoreLogTargets': instance.ignoreLogTargets,
};

_VeilidFFIConfigLoggingOtlp _$VeilidFFIConfigLoggingOtlpFromJson(
  Map<String, dynamic> json,
) => _VeilidFFIConfigLoggingOtlp(
  enabled: json['enabled'] as bool,
  level: VeilidConfigLogLevel.fromJson(json['level']),
  grpcEndpoint: json['grpcEndpoint'] as String,
  serviceName: json['serviceName'] as String,
  ignoreLogTargets:
      (json['ignoreLogTargets'] as List<dynamic>?)
          ?.map((e) => e as String)
          .toList() ??
      const [],
);

Map<String, dynamic> _$VeilidFFIConfigLoggingOtlpToJson(
  _VeilidFFIConfigLoggingOtlp instance,
) => <String, dynamic>{
  'enabled': instance.enabled,
  'level': instance.level.toJson(),
  'grpcEndpoint': instance.grpcEndpoint,
  'serviceName': instance.serviceName,
  'ignoreLogTargets': instance.ignoreLogTargets,
};

_VeilidFFIConfigLoggingApi _$VeilidFFIConfigLoggingApiFromJson(
  Map<String, dynamic> json,
) => _VeilidFFIConfigLoggingApi(
  enabled: json['enabled'] as bool,
  level: VeilidConfigLogLevel.fromJson(json['level']),
  ignoreLogTargets:
      (json['ignoreLogTargets'] as List<dynamic>?)
          ?.map((e) => e as String)
          .toList() ??
      const [],
);

Map<String, dynamic> _$VeilidFFIConfigLoggingApiToJson(
  _VeilidFFIConfigLoggingApi instance,
) => <String, dynamic>{
  'enabled': instance.enabled,
  'level': instance.level.toJson(),
  'ignoreLogTargets': instance.ignoreLogTargets,
};

_VeilidFFIConfigLoggingFlame _$VeilidFFIConfigLoggingFlameFromJson(
  Map<String, dynamic> json,
) => _VeilidFFIConfigLoggingFlame(
  enabled: json['enabled'] as bool,
  path: json['path'] as String,
);

Map<String, dynamic> _$VeilidFFIConfigLoggingFlameToJson(
  _VeilidFFIConfigLoggingFlame instance,
) => <String, dynamic>{'enabled': instance.enabled, 'path': instance.path};

_VeilidFFIConfigLogging _$VeilidFFIConfigLoggingFromJson(
  Map<String, dynamic> json,
) => _VeilidFFIConfigLogging(
  terminal: VeilidFFIConfigLoggingTerminal.fromJson(json['terminal']),
  otlp: VeilidFFIConfigLoggingOtlp.fromJson(json['otlp']),
  api: VeilidFFIConfigLoggingApi.fromJson(json['api']),
  flame: VeilidFFIConfigLoggingFlame.fromJson(json['flame']),
);

Map<String, dynamic> _$VeilidFFIConfigLoggingToJson(
  _VeilidFFIConfigLogging instance,
) => <String, dynamic>{
  'terminal': instance.terminal.toJson(),
  'otlp': instance.otlp.toJson(),
  'api': instance.api.toJson(),
  'flame': instance.flame.toJson(),
};

_VeilidFFIConfig _$VeilidFFIConfigFromJson(Map<String, dynamic> json) =>
    _VeilidFFIConfig(logging: VeilidFFIConfigLogging.fromJson(json['logging']));

Map<String, dynamic> _$VeilidFFIConfigToJson(_VeilidFFIConfig instance) =>
    <String, dynamic>{'logging': instance.logging.toJson()};

_VeilidWASMConfigLoggingPerformance
_$VeilidWASMConfigLoggingPerformanceFromJson(Map<String, dynamic> json) =>
    _VeilidWASMConfigLoggingPerformance(
      enabled: json['enabled'] as bool,
      level: VeilidConfigLogLevel.fromJson(json['level']),
      logsInTimings: json['logsInTimings'] as bool,
      logsInConsole: VeilidWASMConfigLoggingLogsInConsole.fromJson(
        json['logsInConsole'],
      ),
      ignoreLogTargets:
          (json['ignoreLogTargets'] as List<dynamic>?)
              ?.map((e) => e as String)
              .toList() ??
          const [],
    );

Map<String, dynamic> _$VeilidWASMConfigLoggingPerformanceToJson(
  _VeilidWASMConfigLoggingPerformance instance,
) => <String, dynamic>{
  'enabled': instance.enabled,
  'level': instance.level.toJson(),
  'logsInTimings': instance.logsInTimings,
  'logsInConsole': instance.logsInConsole.toJson(),
  'ignoreLogTargets': instance.ignoreLogTargets,
};

_VeilidWASMConfigLoggingApi _$VeilidWASMConfigLoggingApiFromJson(
  Map<String, dynamic> json,
) => _VeilidWASMConfigLoggingApi(
  enabled: json['enabled'] as bool,
  level: VeilidConfigLogLevel.fromJson(json['level']),
  ignoreLogTargets:
      (json['ignoreLogTargets'] as List<dynamic>?)
          ?.map((e) => e as String)
          .toList() ??
      const [],
);

Map<String, dynamic> _$VeilidWASMConfigLoggingApiToJson(
  _VeilidWASMConfigLoggingApi instance,
) => <String, dynamic>{
  'enabled': instance.enabled,
  'level': instance.level.toJson(),
  'ignoreLogTargets': instance.ignoreLogTargets,
};

_VeilidWASMConfigLogging _$VeilidWASMConfigLoggingFromJson(
  Map<String, dynamic> json,
) => _VeilidWASMConfigLogging(
  performance: VeilidWASMConfigLoggingPerformance.fromJson(json['performance']),
  api: VeilidWASMConfigLoggingApi.fromJson(json['api']),
);

Map<String, dynamic> _$VeilidWASMConfigLoggingToJson(
  _VeilidWASMConfigLogging instance,
) => <String, dynamic>{
  'performance': instance.performance.toJson(),
  'api': instance.api.toJson(),
};

_VeilidWASMConfig _$VeilidWASMConfigFromJson(Map<String, dynamic> json) =>
    _VeilidWASMConfig(
      logging: VeilidWASMConfigLogging.fromJson(json['logging']),
    );

Map<String, dynamic> _$VeilidWASMConfigToJson(_VeilidWASMConfig instance) =>
    <String, dynamic>{'logging': instance.logging.toJson()};

_VeilidConfigUDP _$VeilidConfigUDPFromJson(Map<String, dynamic> json) =>
    _VeilidConfigUDP(
      enabled: json['enabled'] as bool,
      socketPoolSize: (json['socketPoolSize'] as num).toInt(),
      listenAddress: json['listenAddress'] as String,
      publicAddress: json['publicAddress'] as String?,
    );

Map<String, dynamic> _$VeilidConfigUDPToJson(_VeilidConfigUDP instance) =>
    <String, dynamic>{
      'enabled': instance.enabled,
      'socketPoolSize': instance.socketPoolSize,
      'listenAddress': instance.listenAddress,
      'publicAddress': instance.publicAddress,
    };

_VeilidConfigTCP _$VeilidConfigTCPFromJson(Map<String, dynamic> json) =>
    _VeilidConfigTCP(
      connect: json['connect'] as bool,
      listen: json['listen'] as bool,
      maxConnections: (json['maxConnections'] as num).toInt(),
      listenAddress: json['listenAddress'] as String,
      publicAddress: json['publicAddress'] as String?,
    );

Map<String, dynamic> _$VeilidConfigTCPToJson(_VeilidConfigTCP instance) =>
    <String, dynamic>{
      'connect': instance.connect,
      'listen': instance.listen,
      'maxConnections': instance.maxConnections,
      'listenAddress': instance.listenAddress,
      'publicAddress': instance.publicAddress,
    };

_VeilidConfigWS _$VeilidConfigWSFromJson(Map<String, dynamic> json) =>
    _VeilidConfigWS(
      connect: json['connect'] as bool,
      listen: json['listen'] as bool,
      maxConnections: (json['maxConnections'] as num).toInt(),
      listenAddress: json['listenAddress'] as String,
      path: json['path'] as String,
      url: json['url'] as String?,
    );

Map<String, dynamic> _$VeilidConfigWSToJson(_VeilidConfigWS instance) =>
    <String, dynamic>{
      'connect': instance.connect,
      'listen': instance.listen,
      'maxConnections': instance.maxConnections,
      'listenAddress': instance.listenAddress,
      'path': instance.path,
      'url': instance.url,
    };

_VeilidConfigProtocol _$VeilidConfigProtocolFromJson(
  Map<String, dynamic> json,
) => _VeilidConfigProtocol(
  udp: VeilidConfigUDP.fromJson(json['udp']),
  tcp: VeilidConfigTCP.fromJson(json['tcp']),
  ws: VeilidConfigWS.fromJson(json['ws']),
);

Map<String, dynamic> _$VeilidConfigProtocolToJson(
  _VeilidConfigProtocol instance,
) => <String, dynamic>{
  'udp': instance.udp.toJson(),
  'tcp': instance.tcp.toJson(),
  'ws': instance.ws.toJson(),
};

_VeilidConfigPrivacy _$VeilidConfigPrivacyFromJson(Map<String, dynamic> json) =>
    _VeilidConfigPrivacy(
      requireInboundRelay: json['requireInboundRelay'] as bool,
    );

Map<String, dynamic> _$VeilidConfigPrivacyToJson(
  _VeilidConfigPrivacy instance,
) => <String, dynamic>{'requireInboundRelay': instance.requireInboundRelay};

_VeilidConfigTLS _$VeilidConfigTLSFromJson(Map<String, dynamic> json) =>
    _VeilidConfigTLS(
      certificatePath: json['certificatePath'] as String,
      privateKeyPath: json['privateKeyPath'] as String,
      connectionInitialTimeoutMs: (json['connectionInitialTimeoutMs'] as num)
          .toInt(),
    );

Map<String, dynamic> _$VeilidConfigTLSToJson(_VeilidConfigTLS instance) =>
    <String, dynamic>{
      'certificatePath': instance.certificatePath,
      'privateKeyPath': instance.privateKeyPath,
      'connectionInitialTimeoutMs': instance.connectionInitialTimeoutMs,
    };

_VeilidConfigDHT _$VeilidConfigDHTFromJson(Map<String, dynamic> json) =>
    _VeilidConfigDHT(
      resolveNodeTimeoutMs: (json['resolveNodeTimeoutMs'] as num).toInt(),
      resolveNodeCount: (json['resolveNodeCount'] as num).toInt(),
      resolveNodeFanout: (json['resolveNodeFanout'] as num).toInt(),
      maxFindNodeCount: (json['maxFindNodeCount'] as num).toInt(),
      getValueTimeoutMs: (json['getValueTimeoutMs'] as num).toInt(),
      getValueCount: (json['getValueCount'] as num).toInt(),
      getValueFanout: (json['getValueFanout'] as num).toInt(),
      setValueTimeoutMs: (json['setValueTimeoutMs'] as num).toInt(),
      setValueCount: (json['setValueCount'] as num).toInt(),
      setValueFanout: (json['setValueFanout'] as num).toInt(),
      consensusWidth: (json['consensusWidth'] as num).toInt(),
      minPeerCount: (json['minPeerCount'] as num).toInt(),
      minPeerRefreshTimeMs: (json['minPeerRefreshTimeMs'] as num).toInt(),
      validateDialInfoReceiptTimeMs:
          (json['validateDialInfoReceiptTimeMs'] as num).toInt(),
      localSubkeyCacheSize: (json['localSubkeyCacheSize'] as num).toInt(),
      localMaxSubkeyCacheMemoryMb: (json['localMaxSubkeyCacheMemoryMb'] as num)
          .toInt(),
      remoteSubkeyCacheSize: (json['remoteSubkeyCacheSize'] as num).toInt(),
      remoteMaxRecords: (json['remoteMaxRecords'] as num).toInt(),
      remoteMaxSubkeyCacheMemoryMb:
          (json['remoteMaxSubkeyCacheMemoryMb'] as num).toInt(),
      remoteMaxStorageSpaceMb: (json['remoteMaxStorageSpaceMb'] as num).toInt(),
      publicWatchLimit: (json['publicWatchLimit'] as num).toInt(),
      memberWatchLimit: (json['memberWatchLimit'] as num).toInt(),
      maxWatchExpirationMs: (json['maxWatchExpirationMs'] as num).toInt(),
      publicTransactionLimit: (json['publicTransactionLimit'] as num).toInt(),
      memberTransactionLimit: (json['memberTransactionLimit'] as num).toInt(),
    );

Map<String, dynamic> _$VeilidConfigDHTToJson(_VeilidConfigDHT instance) =>
    <String, dynamic>{
      'resolveNodeTimeoutMs': instance.resolveNodeTimeoutMs,
      'resolveNodeCount': instance.resolveNodeCount,
      'resolveNodeFanout': instance.resolveNodeFanout,
      'maxFindNodeCount': instance.maxFindNodeCount,
      'getValueTimeoutMs': instance.getValueTimeoutMs,
      'getValueCount': instance.getValueCount,
      'getValueFanout': instance.getValueFanout,
      'setValueTimeoutMs': instance.setValueTimeoutMs,
      'setValueCount': instance.setValueCount,
      'setValueFanout': instance.setValueFanout,
      'consensusWidth': instance.consensusWidth,
      'minPeerCount': instance.minPeerCount,
      'minPeerRefreshTimeMs': instance.minPeerRefreshTimeMs,
      'validateDialInfoReceiptTimeMs': instance.validateDialInfoReceiptTimeMs,
      'localSubkeyCacheSize': instance.localSubkeyCacheSize,
      'localMaxSubkeyCacheMemoryMb': instance.localMaxSubkeyCacheMemoryMb,
      'remoteSubkeyCacheSize': instance.remoteSubkeyCacheSize,
      'remoteMaxRecords': instance.remoteMaxRecords,
      'remoteMaxSubkeyCacheMemoryMb': instance.remoteMaxSubkeyCacheMemoryMb,
      'remoteMaxStorageSpaceMb': instance.remoteMaxStorageSpaceMb,
      'publicWatchLimit': instance.publicWatchLimit,
      'memberWatchLimit': instance.memberWatchLimit,
      'maxWatchExpirationMs': instance.maxWatchExpirationMs,
      'publicTransactionLimit': instance.publicTransactionLimit,
      'memberTransactionLimit': instance.memberTransactionLimit,
    };

_VeilidConfigRPC _$VeilidConfigRPCFromJson(Map<String, dynamic> json) =>
    _VeilidConfigRPC(
      concurrency: (json['concurrency'] as num).toInt(),
      queueSize: (json['queueSize'] as num).toInt(),
      timeoutMs: (json['timeoutMs'] as num).toInt(),
      maxRouteHopCount: (json['maxRouteHopCount'] as num).toInt(),
      defaultRouteHopCount: (json['defaultRouteHopCount'] as num).toInt(),
      maxTimestampBehindMs: (json['maxTimestampBehindMs'] as num?)?.toInt(),
      maxTimestampAheadMs: (json['maxTimestampAheadMs'] as num?)?.toInt(),
    );

Map<String, dynamic> _$VeilidConfigRPCToJson(_VeilidConfigRPC instance) =>
    <String, dynamic>{
      'concurrency': instance.concurrency,
      'queueSize': instance.queueSize,
      'timeoutMs': instance.timeoutMs,
      'maxRouteHopCount': instance.maxRouteHopCount,
      'defaultRouteHopCount': instance.defaultRouteHopCount,
      'maxTimestampBehindMs': instance.maxTimestampBehindMs,
      'maxTimestampAheadMs': instance.maxTimestampAheadMs,
    };

_VeilidConfigRoutingTable _$VeilidConfigRoutingTableFromJson(
  Map<String, dynamic> json,
) => _VeilidConfigRoutingTable(
  publicKeys: (json['publicKeys'] as List<dynamic>)
      .map(Typed<BarePublicKey>.fromJson)
      .toList(),
  secretKeys: (json['secretKeys'] as List<dynamic>)
      .map(Typed<BareSecretKey>.fromJson)
      .toList(),
  bootstrap: (json['bootstrap'] as List<dynamic>)
      .map((e) => e as String)
      .toList(),
  bootstrapKeys: (json['bootstrapKeys'] as List<dynamic>)
      .map(Typed<BarePublicKey>.fromJson)
      .toList(),
  limitOverAttached: (json['limitOverAttached'] as num).toInt(),
  limitFullyAttached: (json['limitFullyAttached'] as num).toInt(),
  limitAttachedStrong: (json['limitAttachedStrong'] as num).toInt(),
  limitAttachedGood: (json['limitAttachedGood'] as num).toInt(),
  limitAttachedWeak: (json['limitAttachedWeak'] as num).toInt(),
);

Map<String, dynamic> _$VeilidConfigRoutingTableToJson(
  _VeilidConfigRoutingTable instance,
) => <String, dynamic>{
  'publicKeys': instance.publicKeys.map((e) => e.toJson()).toList(),
  'secretKeys': instance.secretKeys.map((e) => e.toJson()).toList(),
  'bootstrap': instance.bootstrap,
  'bootstrapKeys': instance.bootstrapKeys.map((e) => e.toJson()).toList(),
  'limitOverAttached': instance.limitOverAttached,
  'limitFullyAttached': instance.limitFullyAttached,
  'limitAttachedStrong': instance.limitAttachedStrong,
  'limitAttachedGood': instance.limitAttachedGood,
  'limitAttachedWeak': instance.limitAttachedWeak,
};

_VeilidConfigNetwork _$VeilidConfigNetworkFromJson(Map<String, dynamic> json) =>
    _VeilidConfigNetwork(
      connectionInitialTimeoutMs: (json['connectionInitialTimeoutMs'] as num)
          .toInt(),
      connectionInactivityTimeoutMs:
          (json['connectionInactivityTimeoutMs'] as num).toInt(),
      maxConnectionsPerIp4: (json['maxConnectionsPerIp4'] as num).toInt(),
      maxConnectionsPerIp6Prefix: (json['maxConnectionsPerIp6Prefix'] as num)
          .toInt(),
      maxConnectionsPerIp6PrefixSize:
          (json['maxConnectionsPerIp6PrefixSize'] as num).toInt(),
      maxConnectionFrequencyPerMin:
          (json['maxConnectionFrequencyPerMin'] as num).toInt(),
      clientAllowlistTimeoutMs: (json['clientAllowlistTimeoutMs'] as num)
          .toInt(),
      reverseConnectionReceiptTimeMs:
          (json['reverseConnectionReceiptTimeMs'] as num).toInt(),
      holePunchReceiptTimeMs: (json['holePunchReceiptTimeMs'] as num).toInt(),
      routingTable: VeilidConfigRoutingTable.fromJson(json['routingTable']),
      rpc: VeilidConfigRPC.fromJson(json['rpc']),
      dht: VeilidConfigDHT.fromJson(json['dht']),
      upnp: json['upnp'] as bool,
      detectAddressChanges: json['detectAddressChanges'] as bool?,
      restrictedNatRetries: (json['restrictedNatRetries'] as num).toInt(),
      tls: VeilidConfigTLS.fromJson(json['tls']),
      protocol: VeilidConfigProtocol.fromJson(json['protocol']),
      privacy: VeilidConfigPrivacy.fromJson(json['privacy']),
      networkKeyPassword: json['networkKeyPassword'] as String?,
    );

Map<String, dynamic> _$VeilidConfigNetworkToJson(
  _VeilidConfigNetwork instance,
) => <String, dynamic>{
  'connectionInitialTimeoutMs': instance.connectionInitialTimeoutMs,
  'connectionInactivityTimeoutMs': instance.connectionInactivityTimeoutMs,
  'maxConnectionsPerIp4': instance.maxConnectionsPerIp4,
  'maxConnectionsPerIp6Prefix': instance.maxConnectionsPerIp6Prefix,
  'maxConnectionsPerIp6PrefixSize': instance.maxConnectionsPerIp6PrefixSize,
  'maxConnectionFrequencyPerMin': instance.maxConnectionFrequencyPerMin,
  'clientAllowlistTimeoutMs': instance.clientAllowlistTimeoutMs,
  'reverseConnectionReceiptTimeMs': instance.reverseConnectionReceiptTimeMs,
  'holePunchReceiptTimeMs': instance.holePunchReceiptTimeMs,
  'routingTable': instance.routingTable.toJson(),
  'rpc': instance.rpc.toJson(),
  'dht': instance.dht.toJson(),
  'upnp': instance.upnp,
  'detectAddressChanges': instance.detectAddressChanges,
  'restrictedNatRetries': instance.restrictedNatRetries,
  'tls': instance.tls.toJson(),
  'protocol': instance.protocol.toJson(),
  'privacy': instance.privacy.toJson(),
  'networkKeyPassword': instance.networkKeyPassword,
};

_VeilidConfigTableStore _$VeilidConfigTableStoreFromJson(
  Map<String, dynamic> json,
) => _VeilidConfigTableStore(
  directory: json['directory'] as String,
  delete: json['delete'] as bool,
);

Map<String, dynamic> _$VeilidConfigTableStoreToJson(
  _VeilidConfigTableStore instance,
) => <String, dynamic>{
  'directory': instance.directory,
  'delete': instance.delete,
};

_VeilidConfigBlockStore _$VeilidConfigBlockStoreFromJson(
  Map<String, dynamic> json,
) => _VeilidConfigBlockStore(
  directory: json['directory'] as String,
  delete: json['delete'] as bool,
);

Map<String, dynamic> _$VeilidConfigBlockStoreToJson(
  _VeilidConfigBlockStore instance,
) => <String, dynamic>{
  'directory': instance.directory,
  'delete': instance.delete,
};

_VeilidConfigProtectedStore _$VeilidConfigProtectedStoreFromJson(
  Map<String, dynamic> json,
) => _VeilidConfigProtectedStore(
  allowInsecureFallback: json['allowInsecureFallback'] as bool,
  alwaysUseInsecureStorage: json['alwaysUseInsecureStorage'] as bool,
  directory: json['directory'] as String,
  delete: json['delete'] as bool,
  deviceEncryptionKeyPassword: json['deviceEncryptionKeyPassword'] as String,
  newDeviceEncryptionKeyPassword:
      json['newDeviceEncryptionKeyPassword'] as String?,
);

Map<String, dynamic> _$VeilidConfigProtectedStoreToJson(
  _VeilidConfigProtectedStore instance,
) => <String, dynamic>{
  'allowInsecureFallback': instance.allowInsecureFallback,
  'alwaysUseInsecureStorage': instance.alwaysUseInsecureStorage,
  'directory': instance.directory,
  'delete': instance.delete,
  'deviceEncryptionKeyPassword': instance.deviceEncryptionKeyPassword,
  'newDeviceEncryptionKeyPassword': instance.newDeviceEncryptionKeyPassword,
};

_VeilidConfigCapabilities _$VeilidConfigCapabilitiesFromJson(
  Map<String, dynamic> json,
) => _VeilidConfigCapabilities(
  disable: (json['disable'] as List<dynamic>).map((e) => e as String).toList(),
);

Map<String, dynamic> _$VeilidConfigCapabilitiesToJson(
  _VeilidConfigCapabilities instance,
) => <String, dynamic>{'disable': instance.disable};

_VeilidConfig _$VeilidConfigFromJson(Map<String, dynamic> json) =>
    _VeilidConfig(
      programName: json['programName'] as String,
      namespace: json['namespace'] as String,
      capabilities: VeilidConfigCapabilities.fromJson(json['capabilities']),
      protectedStore: VeilidConfigProtectedStore.fromJson(
        json['protectedStore'],
      ),
      tableStore: VeilidConfigTableStore.fromJson(json['tableStore']),
      blockStore: VeilidConfigBlockStore.fromJson(json['blockStore']),
      network: VeilidConfigNetwork.fromJson(json['network']),
    );

Map<String, dynamic> _$VeilidConfigToJson(_VeilidConfig instance) =>
    <String, dynamic>{
      'programName': instance.programName,
      'namespace': instance.namespace,
      'capabilities': instance.capabilities.toJson(),
      'protectedStore': instance.protectedStore.toJson(),
      'tableStore': instance.tableStore.toJson(),
      'blockStore': instance.blockStore.toJson(),
      'network': instance.network.toJson(),
    };
