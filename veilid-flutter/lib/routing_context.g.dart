// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'routing_context.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

DHTSchemaDFLT _$DHTSchemaDFLTFromJson(Map<String, dynamic> json) =>
    DHTSchemaDFLT(
      oCnt: (json['oCnt'] as num).toInt(),
      $type: json['kind'] as String?,
    );

Map<String, dynamic> _$DHTSchemaDFLTToJson(DHTSchemaDFLT instance) =>
    <String, dynamic>{'oCnt': instance.oCnt, 'kind': instance.$type};

DHTSchemaSMPL _$DHTSchemaSMPLFromJson(Map<String, dynamic> json) =>
    DHTSchemaSMPL(
      oCnt: (json['oCnt'] as num).toInt(),
      members: (json['members'] as List<dynamic>)
          .map(DHTSchemaMember.fromJson)
          .toList(),
      $type: json['kind'] as String?,
    );

Map<String, dynamic> _$DHTSchemaSMPLToJson(DHTSchemaSMPL instance) =>
    <String, dynamic>{
      'oCnt': instance.oCnt,
      'members': instance.members.map((e) => e.toJson()).toList(),
      'kind': instance.$type,
    };

_DHTSchemaMember _$DHTSchemaMemberFromJson(Map<String, dynamic> json) =>
    _DHTSchemaMember(
      mKey: BareMemberId.fromJson(json['mKey']),
      mCnt: (json['mCnt'] as num).toInt(),
    );

Map<String, dynamic> _$DHTSchemaMemberToJson(_DHTSchemaMember instance) =>
    <String, dynamic>{'mKey': instance.mKey.toJson(), 'mCnt': instance.mCnt};

_DHTRecordDescriptor _$DHTRecordDescriptorFromJson(Map<String, dynamic> json) =>
    _DHTRecordDescriptor(
      key: RecordKey.fromJson(json['key']),
      owner: Typed<BarePublicKey>.fromJson(json['owner']),
      schema: DHTSchema.fromJson(json['schema']),
      ownerSecret: json['ownerSecret'] == null
          ? null
          : Typed<BareSecretKey>.fromJson(json['ownerSecret']),
    );

Map<String, dynamic> _$DHTRecordDescriptorToJson(
  _DHTRecordDescriptor instance,
) => <String, dynamic>{
  'key': instance.key.toJson(),
  'owner': instance.owner.toJson(),
  'schema': instance.schema.toJson(),
  'ownerSecret': instance.ownerSecret?.toJson(),
};

_ValueData _$ValueDataFromJson(Map<String, dynamic> json) => _ValueData(
  seq: (json['seq'] as num).toInt(),
  data: const Uint8ListJsonConverter.jsIsArray().fromJson(json['data']),
  writer: Typed<BarePublicKey>.fromJson(json['writer']),
);

Map<String, dynamic> _$ValueDataToJson(_ValueData instance) =>
    <String, dynamic>{
      'seq': instance.seq,
      'data': const Uint8ListJsonConverter.jsIsArray().toJson(instance.data),
      'writer': instance.writer.toJson(),
    };

_SafetySpec _$SafetySpecFromJson(Map<String, dynamic> json) => _SafetySpec(
  hopCount: (json['hopCount'] as num).toInt(),
  stability: Stability.fromJson(json['stability']),
  sequencing: Sequencing.fromJson(json['sequencing']),
  preferredRoute: json['preferredRoute'] == null
      ? null
      : Typed<BareRouteId>.fromJson(json['preferredRoute']),
);

Map<String, dynamic> _$SafetySpecToJson(_SafetySpec instance) =>
    <String, dynamic>{
      'hopCount': instance.hopCount,
      'stability': instance.stability.toJson(),
      'sequencing': instance.sequencing.toJson(),
      'preferredRoute': instance.preferredRoute?.toJson(),
    };

_RouteBlob _$RouteBlobFromJson(Map<String, dynamic> json) => _RouteBlob(
  routeId: Typed<BareRouteId>.fromJson(json['routeId']),
  blob: const Uint8ListJsonConverter.jsIsArray().fromJson(json['blob']),
);

Map<String, dynamic> _$RouteBlobToJson(_RouteBlob instance) =>
    <String, dynamic>{
      'routeId': instance.routeId.toJson(),
      'blob': const Uint8ListJsonConverter.jsIsArray().toJson(instance.blob),
    };

_DHTRecordReport _$DHTRecordReportFromJson(Map<String, dynamic> json) =>
    _DHTRecordReport(
      subkeys: (json['subkeys'] as List<dynamic>)
          .map(ValueSubkeyRange.fromJson)
          .toList(),
      offlineSubkeys: (json['offlineSubkeys'] as List<dynamic>)
          .map(ValueSubkeyRange.fromJson)
          .toList(),
      localSeqs: (json['localSeqs'] as List<dynamic>)
          .map((e) => (e as num?)?.toInt())
          .toList(),
      networkSeqs: (json['networkSeqs'] as List<dynamic>)
          .map((e) => (e as num?)?.toInt())
          .toList(),
    );

Map<String, dynamic> _$DHTRecordReportToJson(_DHTRecordReport instance) =>
    <String, dynamic>{
      'subkeys': instance.subkeys.map((e) => e.toJson()).toList(),
      'offlineSubkeys': instance.offlineSubkeys.map((e) => e.toJson()).toList(),
      'localSeqs': instance.localSeqs,
      'networkSeqs': instance.networkSeqs,
    };

_SetDHTValueOptions _$SetDHTValueOptionsFromJson(Map<String, dynamic> json) =>
    _SetDHTValueOptions(
      writer: json['writer'] == null ? null : KeyPair.fromJson(json['writer']),
      allowOffline: json['allowOffline'] as bool?,
    );

Map<String, dynamic> _$SetDHTValueOptionsToJson(_SetDHTValueOptions instance) =>
    <String, dynamic>{
      'writer': instance.writer?.toJson(),
      'allowOffline': instance.allowOffline,
    };
