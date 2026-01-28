// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'routing_context.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
DHTSchema _$DHTSchemaFromJson(
  Map<String, dynamic> json
) {
        switch (json['kind']) {
                  case 'DFLT':
          return DHTSchemaDFLT.fromJson(
            json
          );
                case 'SMPL':
          return DHTSchemaSMPL.fromJson(
            json
          );
        
          default:
            throw CheckedFromJsonException(
  json,
  'kind',
  'DHTSchema',
  'Invalid union type "${json['kind']}"!'
);
        }
      
}

/// @nodoc
mixin _$DHTSchema {

 int get oCnt;
/// Create a copy of DHTSchema
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DHTSchemaCopyWith<DHTSchema> get copyWith => _$DHTSchemaCopyWithImpl<DHTSchema>(this as DHTSchema, _$identity);

  /// Serializes this DHTSchema to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DHTSchema&&(identical(other.oCnt, oCnt) || other.oCnt == oCnt));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,oCnt);

@override
String toString() {
  return 'DHTSchema(oCnt: $oCnt)';
}


}

/// @nodoc
abstract mixin class $DHTSchemaCopyWith<$Res>  {
  factory $DHTSchemaCopyWith(DHTSchema value, $Res Function(DHTSchema) _then) = _$DHTSchemaCopyWithImpl;
@useResult
$Res call({
 int oCnt
});




}
/// @nodoc
class _$DHTSchemaCopyWithImpl<$Res>
    implements $DHTSchemaCopyWith<$Res> {
  _$DHTSchemaCopyWithImpl(this._self, this._then);

  final DHTSchema _self;
  final $Res Function(DHTSchema) _then;

/// Create a copy of DHTSchema
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? oCnt = null,}) {
  return _then(_self.copyWith(
oCnt: null == oCnt ? _self.oCnt : oCnt // ignore: cast_nullable_to_non_nullable
as int,
  ));
}

}


/// Adds pattern-matching-related methods to [DHTSchema].
extension DHTSchemaPatterns on DHTSchema {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( DHTSchemaDFLT value)?  dflt,TResult Function( DHTSchemaSMPL value)?  smpl,required TResult orElse(),}){
final _that = this;
switch (_that) {
case DHTSchemaDFLT() when dflt != null:
return dflt(_that);case DHTSchemaSMPL() when smpl != null:
return smpl(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( DHTSchemaDFLT value)  dflt,required TResult Function( DHTSchemaSMPL value)  smpl,}){
final _that = this;
switch (_that) {
case DHTSchemaDFLT():
return dflt(_that);case DHTSchemaSMPL():
return smpl(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( DHTSchemaDFLT value)?  dflt,TResult? Function( DHTSchemaSMPL value)?  smpl,}){
final _that = this;
switch (_that) {
case DHTSchemaDFLT() when dflt != null:
return dflt(_that);case DHTSchemaSMPL() when smpl != null:
return smpl(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( int oCnt)?  dflt,TResult Function( int oCnt,  List<DHTSchemaMember> members)?  smpl,required TResult orElse(),}) {final _that = this;
switch (_that) {
case DHTSchemaDFLT() when dflt != null:
return dflt(_that.oCnt);case DHTSchemaSMPL() when smpl != null:
return smpl(_that.oCnt,_that.members);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( int oCnt)  dflt,required TResult Function( int oCnt,  List<DHTSchemaMember> members)  smpl,}) {final _that = this;
switch (_that) {
case DHTSchemaDFLT():
return dflt(_that.oCnt);case DHTSchemaSMPL():
return smpl(_that.oCnt,_that.members);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( int oCnt)?  dflt,TResult? Function( int oCnt,  List<DHTSchemaMember> members)?  smpl,}) {final _that = this;
switch (_that) {
case DHTSchemaDFLT() when dflt != null:
return dflt(_that.oCnt);case DHTSchemaSMPL() when smpl != null:
return smpl(_that.oCnt,_that.members);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class DHTSchemaDFLT implements DHTSchema {
  const DHTSchemaDFLT({required this.oCnt, final  String? $type}): assert(oCnt >= 0, 'value must not be negative'),$type = $type ?? 'DFLT';
  factory DHTSchemaDFLT.fromJson(Map<String, dynamic> json) => _$DHTSchemaDFLTFromJson(json);

@override final  int oCnt;

@JsonKey(name: 'kind')
final String $type;


/// Create a copy of DHTSchema
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DHTSchemaDFLTCopyWith<DHTSchemaDFLT> get copyWith => _$DHTSchemaDFLTCopyWithImpl<DHTSchemaDFLT>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$DHTSchemaDFLTToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DHTSchemaDFLT&&(identical(other.oCnt, oCnt) || other.oCnt == oCnt));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,oCnt);

@override
String toString() {
  return 'DHTSchema.dflt(oCnt: $oCnt)';
}


}

/// @nodoc
abstract mixin class $DHTSchemaDFLTCopyWith<$Res> implements $DHTSchemaCopyWith<$Res> {
  factory $DHTSchemaDFLTCopyWith(DHTSchemaDFLT value, $Res Function(DHTSchemaDFLT) _then) = _$DHTSchemaDFLTCopyWithImpl;
@override @useResult
$Res call({
 int oCnt
});




}
/// @nodoc
class _$DHTSchemaDFLTCopyWithImpl<$Res>
    implements $DHTSchemaDFLTCopyWith<$Res> {
  _$DHTSchemaDFLTCopyWithImpl(this._self, this._then);

  final DHTSchemaDFLT _self;
  final $Res Function(DHTSchemaDFLT) _then;

/// Create a copy of DHTSchema
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? oCnt = null,}) {
  return _then(DHTSchemaDFLT(
oCnt: null == oCnt ? _self.oCnt : oCnt // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc
@JsonSerializable()

class DHTSchemaSMPL implements DHTSchema {
  const DHTSchemaSMPL({required this.oCnt, required final  List<DHTSchemaMember> members, final  String? $type}): assert(oCnt >= 0, 'value must not be negative'),_members = members,$type = $type ?? 'SMPL';
  factory DHTSchemaSMPL.fromJson(Map<String, dynamic> json) => _$DHTSchemaSMPLFromJson(json);

@override final  int oCnt;
 final  List<DHTSchemaMember> _members;
 List<DHTSchemaMember> get members {
  if (_members is EqualUnmodifiableListView) return _members;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_members);
}


@JsonKey(name: 'kind')
final String $type;


/// Create a copy of DHTSchema
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DHTSchemaSMPLCopyWith<DHTSchemaSMPL> get copyWith => _$DHTSchemaSMPLCopyWithImpl<DHTSchemaSMPL>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$DHTSchemaSMPLToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DHTSchemaSMPL&&(identical(other.oCnt, oCnt) || other.oCnt == oCnt)&&const DeepCollectionEquality().equals(other._members, _members));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,oCnt,const DeepCollectionEquality().hash(_members));

@override
String toString() {
  return 'DHTSchema.smpl(oCnt: $oCnt, members: $members)';
}


}

/// @nodoc
abstract mixin class $DHTSchemaSMPLCopyWith<$Res> implements $DHTSchemaCopyWith<$Res> {
  factory $DHTSchemaSMPLCopyWith(DHTSchemaSMPL value, $Res Function(DHTSchemaSMPL) _then) = _$DHTSchemaSMPLCopyWithImpl;
@override @useResult
$Res call({
 int oCnt, List<DHTSchemaMember> members
});




}
/// @nodoc
class _$DHTSchemaSMPLCopyWithImpl<$Res>
    implements $DHTSchemaSMPLCopyWith<$Res> {
  _$DHTSchemaSMPLCopyWithImpl(this._self, this._then);

  final DHTSchemaSMPL _self;
  final $Res Function(DHTSchemaSMPL) _then;

/// Create a copy of DHTSchema
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? oCnt = null,Object? members = null,}) {
  return _then(DHTSchemaSMPL(
oCnt: null == oCnt ? _self.oCnt : oCnt // ignore: cast_nullable_to_non_nullable
as int,members: null == members ? _self._members : members // ignore: cast_nullable_to_non_nullable
as List<DHTSchemaMember>,
  ));
}


}


/// @nodoc
mixin _$DHTSchemaMember {

 BareMemberId get mKey; int get mCnt;
/// Create a copy of DHTSchemaMember
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DHTSchemaMemberCopyWith<DHTSchemaMember> get copyWith => _$DHTSchemaMemberCopyWithImpl<DHTSchemaMember>(this as DHTSchemaMember, _$identity);

  /// Serializes this DHTSchemaMember to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DHTSchemaMember&&(identical(other.mKey, mKey) || other.mKey == mKey)&&(identical(other.mCnt, mCnt) || other.mCnt == mCnt));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,mKey,mCnt);

@override
String toString() {
  return 'DHTSchemaMember(mKey: $mKey, mCnt: $mCnt)';
}


}

/// @nodoc
abstract mixin class $DHTSchemaMemberCopyWith<$Res>  {
  factory $DHTSchemaMemberCopyWith(DHTSchemaMember value, $Res Function(DHTSchemaMember) _then) = _$DHTSchemaMemberCopyWithImpl;
@useResult
$Res call({
 BareMemberId mKey, int mCnt
});




}
/// @nodoc
class _$DHTSchemaMemberCopyWithImpl<$Res>
    implements $DHTSchemaMemberCopyWith<$Res> {
  _$DHTSchemaMemberCopyWithImpl(this._self, this._then);

  final DHTSchemaMember _self;
  final $Res Function(DHTSchemaMember) _then;

/// Create a copy of DHTSchemaMember
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? mKey = null,Object? mCnt = null,}) {
  return _then(_self.copyWith(
mKey: null == mKey ? _self.mKey : mKey // ignore: cast_nullable_to_non_nullable
as BareMemberId,mCnt: null == mCnt ? _self.mCnt : mCnt // ignore: cast_nullable_to_non_nullable
as int,
  ));
}

}


/// Adds pattern-matching-related methods to [DHTSchemaMember].
extension DHTSchemaMemberPatterns on DHTSchemaMember {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _DHTSchemaMember value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DHTSchemaMember() when $default != null:
return $default(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _DHTSchemaMember value)  $default,){
final _that = this;
switch (_that) {
case _DHTSchemaMember():
return $default(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _DHTSchemaMember value)?  $default,){
final _that = this;
switch (_that) {
case _DHTSchemaMember() when $default != null:
return $default(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( BareMemberId mKey,  int mCnt)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DHTSchemaMember() when $default != null:
return $default(_that.mKey,_that.mCnt);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( BareMemberId mKey,  int mCnt)  $default,) {final _that = this;
switch (_that) {
case _DHTSchemaMember():
return $default(_that.mKey,_that.mCnt);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( BareMemberId mKey,  int mCnt)?  $default,) {final _that = this;
switch (_that) {
case _DHTSchemaMember() when $default != null:
return $default(_that.mKey,_that.mCnt);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _DHTSchemaMember implements DHTSchemaMember {
  const _DHTSchemaMember({required this.mKey, required this.mCnt}): assert(mCnt >= 0, 'value must not be negative');
  factory _DHTSchemaMember.fromJson(Map<String, dynamic> json) => _$DHTSchemaMemberFromJson(json);

@override final  BareMemberId mKey;
@override final  int mCnt;

/// Create a copy of DHTSchemaMember
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DHTSchemaMemberCopyWith<_DHTSchemaMember> get copyWith => __$DHTSchemaMemberCopyWithImpl<_DHTSchemaMember>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$DHTSchemaMemberToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DHTSchemaMember&&(identical(other.mKey, mKey) || other.mKey == mKey)&&(identical(other.mCnt, mCnt) || other.mCnt == mCnt));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,mKey,mCnt);

@override
String toString() {
  return 'DHTSchemaMember(mKey: $mKey, mCnt: $mCnt)';
}


}

/// @nodoc
abstract mixin class _$DHTSchemaMemberCopyWith<$Res> implements $DHTSchemaMemberCopyWith<$Res> {
  factory _$DHTSchemaMemberCopyWith(_DHTSchemaMember value, $Res Function(_DHTSchemaMember) _then) = __$DHTSchemaMemberCopyWithImpl;
@override @useResult
$Res call({
 BareMemberId mKey, int mCnt
});




}
/// @nodoc
class __$DHTSchemaMemberCopyWithImpl<$Res>
    implements _$DHTSchemaMemberCopyWith<$Res> {
  __$DHTSchemaMemberCopyWithImpl(this._self, this._then);

  final _DHTSchemaMember _self;
  final $Res Function(_DHTSchemaMember) _then;

/// Create a copy of DHTSchemaMember
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? mKey = null,Object? mCnt = null,}) {
  return _then(_DHTSchemaMember(
mKey: null == mKey ? _self.mKey : mKey // ignore: cast_nullable_to_non_nullable
as BareMemberId,mCnt: null == mCnt ? _self.mCnt : mCnt // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}


/// @nodoc
mixin _$DHTRecordDescriptor {

 RecordKey get key; PublicKey get owner; DHTSchema get schema; SecretKey? get ownerSecret;
/// Create a copy of DHTRecordDescriptor
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DHTRecordDescriptorCopyWith<DHTRecordDescriptor> get copyWith => _$DHTRecordDescriptorCopyWithImpl<DHTRecordDescriptor>(this as DHTRecordDescriptor, _$identity);

  /// Serializes this DHTRecordDescriptor to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DHTRecordDescriptor&&(identical(other.key, key) || other.key == key)&&(identical(other.owner, owner) || other.owner == owner)&&(identical(other.schema, schema) || other.schema == schema)&&(identical(other.ownerSecret, ownerSecret) || other.ownerSecret == ownerSecret));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,key,owner,schema,ownerSecret);

@override
String toString() {
  return 'DHTRecordDescriptor(key: $key, owner: $owner, schema: $schema, ownerSecret: $ownerSecret)';
}


}

/// @nodoc
abstract mixin class $DHTRecordDescriptorCopyWith<$Res>  {
  factory $DHTRecordDescriptorCopyWith(DHTRecordDescriptor value, $Res Function(DHTRecordDescriptor) _then) = _$DHTRecordDescriptorCopyWithImpl;
@useResult
$Res call({
 RecordKey key, PublicKey owner, DHTSchema schema, SecretKey? ownerSecret
});


$DHTSchemaCopyWith<$Res> get schema;

}
/// @nodoc
class _$DHTRecordDescriptorCopyWithImpl<$Res>
    implements $DHTRecordDescriptorCopyWith<$Res> {
  _$DHTRecordDescriptorCopyWithImpl(this._self, this._then);

  final DHTRecordDescriptor _self;
  final $Res Function(DHTRecordDescriptor) _then;

/// Create a copy of DHTRecordDescriptor
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? key = null,Object? owner = null,Object? schema = null,Object? ownerSecret = freezed,}) {
  return _then(_self.copyWith(
key: null == key ? _self.key : key // ignore: cast_nullable_to_non_nullable
as RecordKey,owner: null == owner ? _self.owner : owner // ignore: cast_nullable_to_non_nullable
as PublicKey,schema: null == schema ? _self.schema : schema // ignore: cast_nullable_to_non_nullable
as DHTSchema,ownerSecret: freezed == ownerSecret ? _self.ownerSecret : ownerSecret // ignore: cast_nullable_to_non_nullable
as SecretKey?,
  ));
}
/// Create a copy of DHTRecordDescriptor
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$DHTSchemaCopyWith<$Res> get schema {
  
  return $DHTSchemaCopyWith<$Res>(_self.schema, (value) {
    return _then(_self.copyWith(schema: value));
  });
}
}


/// Adds pattern-matching-related methods to [DHTRecordDescriptor].
extension DHTRecordDescriptorPatterns on DHTRecordDescriptor {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _DHTRecordDescriptor value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DHTRecordDescriptor() when $default != null:
return $default(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _DHTRecordDescriptor value)  $default,){
final _that = this;
switch (_that) {
case _DHTRecordDescriptor():
return $default(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _DHTRecordDescriptor value)?  $default,){
final _that = this;
switch (_that) {
case _DHTRecordDescriptor() when $default != null:
return $default(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( RecordKey key,  PublicKey owner,  DHTSchema schema,  SecretKey? ownerSecret)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DHTRecordDescriptor() when $default != null:
return $default(_that.key,_that.owner,_that.schema,_that.ownerSecret);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( RecordKey key,  PublicKey owner,  DHTSchema schema,  SecretKey? ownerSecret)  $default,) {final _that = this;
switch (_that) {
case _DHTRecordDescriptor():
return $default(_that.key,_that.owner,_that.schema,_that.ownerSecret);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( RecordKey key,  PublicKey owner,  DHTSchema schema,  SecretKey? ownerSecret)?  $default,) {final _that = this;
switch (_that) {
case _DHTRecordDescriptor() when $default != null:
return $default(_that.key,_that.owner,_that.schema,_that.ownerSecret);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _DHTRecordDescriptor implements DHTRecordDescriptor {
  const _DHTRecordDescriptor({required this.key, required this.owner, required this.schema, this.ownerSecret});
  factory _DHTRecordDescriptor.fromJson(Map<String, dynamic> json) => _$DHTRecordDescriptorFromJson(json);

@override final  RecordKey key;
@override final  PublicKey owner;
@override final  DHTSchema schema;
@override final  SecretKey? ownerSecret;

/// Create a copy of DHTRecordDescriptor
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DHTRecordDescriptorCopyWith<_DHTRecordDescriptor> get copyWith => __$DHTRecordDescriptorCopyWithImpl<_DHTRecordDescriptor>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$DHTRecordDescriptorToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DHTRecordDescriptor&&(identical(other.key, key) || other.key == key)&&(identical(other.owner, owner) || other.owner == owner)&&(identical(other.schema, schema) || other.schema == schema)&&(identical(other.ownerSecret, ownerSecret) || other.ownerSecret == ownerSecret));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,key,owner,schema,ownerSecret);

@override
String toString() {
  return 'DHTRecordDescriptor(key: $key, owner: $owner, schema: $schema, ownerSecret: $ownerSecret)';
}


}

/// @nodoc
abstract mixin class _$DHTRecordDescriptorCopyWith<$Res> implements $DHTRecordDescriptorCopyWith<$Res> {
  factory _$DHTRecordDescriptorCopyWith(_DHTRecordDescriptor value, $Res Function(_DHTRecordDescriptor) _then) = __$DHTRecordDescriptorCopyWithImpl;
@override @useResult
$Res call({
 RecordKey key, PublicKey owner, DHTSchema schema, SecretKey? ownerSecret
});


@override $DHTSchemaCopyWith<$Res> get schema;

}
/// @nodoc
class __$DHTRecordDescriptorCopyWithImpl<$Res>
    implements _$DHTRecordDescriptorCopyWith<$Res> {
  __$DHTRecordDescriptorCopyWithImpl(this._self, this._then);

  final _DHTRecordDescriptor _self;
  final $Res Function(_DHTRecordDescriptor) _then;

/// Create a copy of DHTRecordDescriptor
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? key = null,Object? owner = null,Object? schema = null,Object? ownerSecret = freezed,}) {
  return _then(_DHTRecordDescriptor(
key: null == key ? _self.key : key // ignore: cast_nullable_to_non_nullable
as RecordKey,owner: null == owner ? _self.owner : owner // ignore: cast_nullable_to_non_nullable
as PublicKey,schema: null == schema ? _self.schema : schema // ignore: cast_nullable_to_non_nullable
as DHTSchema,ownerSecret: freezed == ownerSecret ? _self.ownerSecret : ownerSecret // ignore: cast_nullable_to_non_nullable
as SecretKey?,
  ));
}

/// Create a copy of DHTRecordDescriptor
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$DHTSchemaCopyWith<$Res> get schema {
  
  return $DHTSchemaCopyWith<$Res>(_self.schema, (value) {
    return _then(_self.copyWith(schema: value));
  });
}
}


/// @nodoc
mixin _$ValueData {

 int get seq;@Uint8ListJsonConverter.jsIsArray() Uint8List get data; PublicKey get writer;
/// Create a copy of ValueData
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ValueDataCopyWith<ValueData> get copyWith => _$ValueDataCopyWithImpl<ValueData>(this as ValueData, _$identity);

  /// Serializes this ValueData to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ValueData&&(identical(other.seq, seq) || other.seq == seq)&&const DeepCollectionEquality().equals(other.data, data)&&(identical(other.writer, writer) || other.writer == writer));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,seq,const DeepCollectionEquality().hash(data),writer);

@override
String toString() {
  return 'ValueData(seq: $seq, data: $data, writer: $writer)';
}


}

/// @nodoc
abstract mixin class $ValueDataCopyWith<$Res>  {
  factory $ValueDataCopyWith(ValueData value, $Res Function(ValueData) _then) = _$ValueDataCopyWithImpl;
@useResult
$Res call({
 int seq,@Uint8ListJsonConverter.jsIsArray() Uint8List data, PublicKey writer
});




}
/// @nodoc
class _$ValueDataCopyWithImpl<$Res>
    implements $ValueDataCopyWith<$Res> {
  _$ValueDataCopyWithImpl(this._self, this._then);

  final ValueData _self;
  final $Res Function(ValueData) _then;

/// Create a copy of ValueData
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? seq = null,Object? data = null,Object? writer = null,}) {
  return _then(_self.copyWith(
seq: null == seq ? _self.seq : seq // ignore: cast_nullable_to_non_nullable
as int,data: null == data ? _self.data : data // ignore: cast_nullable_to_non_nullable
as Uint8List,writer: null == writer ? _self.writer : writer // ignore: cast_nullable_to_non_nullable
as PublicKey,
  ));
}

}


/// Adds pattern-matching-related methods to [ValueData].
extension ValueDataPatterns on ValueData {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ValueData value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ValueData() when $default != null:
return $default(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ValueData value)  $default,){
final _that = this;
switch (_that) {
case _ValueData():
return $default(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ValueData value)?  $default,){
final _that = this;
switch (_that) {
case _ValueData() when $default != null:
return $default(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int seq, @Uint8ListJsonConverter.jsIsArray()  Uint8List data,  PublicKey writer)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ValueData() when $default != null:
return $default(_that.seq,_that.data,_that.writer);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int seq, @Uint8ListJsonConverter.jsIsArray()  Uint8List data,  PublicKey writer)  $default,) {final _that = this;
switch (_that) {
case _ValueData():
return $default(_that.seq,_that.data,_that.writer);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int seq, @Uint8ListJsonConverter.jsIsArray()  Uint8List data,  PublicKey writer)?  $default,) {final _that = this;
switch (_that) {
case _ValueData() when $default != null:
return $default(_that.seq,_that.data,_that.writer);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _ValueData implements ValueData {
  const _ValueData({required this.seq, @Uint8ListJsonConverter.jsIsArray() required this.data, required this.writer}): assert(seq >= 0 && seq <= 4294967295, 'seq out of range'),assert(data.length <= ValueData.maxLen, 'data too large');
  factory _ValueData.fromJson(Map<String, dynamic> json) => _$ValueDataFromJson(json);

@override final  int seq;
@override@Uint8ListJsonConverter.jsIsArray() final  Uint8List data;
@override final  PublicKey writer;

/// Create a copy of ValueData
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ValueDataCopyWith<_ValueData> get copyWith => __$ValueDataCopyWithImpl<_ValueData>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$ValueDataToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ValueData&&(identical(other.seq, seq) || other.seq == seq)&&const DeepCollectionEquality().equals(other.data, data)&&(identical(other.writer, writer) || other.writer == writer));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,seq,const DeepCollectionEquality().hash(data),writer);

@override
String toString() {
  return 'ValueData(seq: $seq, data: $data, writer: $writer)';
}


}

/// @nodoc
abstract mixin class _$ValueDataCopyWith<$Res> implements $ValueDataCopyWith<$Res> {
  factory _$ValueDataCopyWith(_ValueData value, $Res Function(_ValueData) _then) = __$ValueDataCopyWithImpl;
@override @useResult
$Res call({
 int seq,@Uint8ListJsonConverter.jsIsArray() Uint8List data, PublicKey writer
});




}
/// @nodoc
class __$ValueDataCopyWithImpl<$Res>
    implements _$ValueDataCopyWith<$Res> {
  __$ValueDataCopyWithImpl(this._self, this._then);

  final _ValueData _self;
  final $Res Function(_ValueData) _then;

/// Create a copy of ValueData
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? seq = null,Object? data = null,Object? writer = null,}) {
  return _then(_ValueData(
seq: null == seq ? _self.seq : seq // ignore: cast_nullable_to_non_nullable
as int,data: null == data ? _self.data : data // ignore: cast_nullable_to_non_nullable
as Uint8List,writer: null == writer ? _self.writer : writer // ignore: cast_nullable_to_non_nullable
as PublicKey,
  ));
}


}


/// @nodoc
mixin _$SafetySpec {

 int get hopCount; Stability get stability; Sequencing get sequencing; RouteId? get preferredRoute;
/// Create a copy of SafetySpec
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SafetySpecCopyWith<SafetySpec> get copyWith => _$SafetySpecCopyWithImpl<SafetySpec>(this as SafetySpec, _$identity);

  /// Serializes this SafetySpec to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SafetySpec&&(identical(other.hopCount, hopCount) || other.hopCount == hopCount)&&(identical(other.stability, stability) || other.stability == stability)&&(identical(other.sequencing, sequencing) || other.sequencing == sequencing)&&(identical(other.preferredRoute, preferredRoute) || other.preferredRoute == preferredRoute));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,hopCount,stability,sequencing,preferredRoute);

@override
String toString() {
  return 'SafetySpec(hopCount: $hopCount, stability: $stability, sequencing: $sequencing, preferredRoute: $preferredRoute)';
}


}

/// @nodoc
abstract mixin class $SafetySpecCopyWith<$Res>  {
  factory $SafetySpecCopyWith(SafetySpec value, $Res Function(SafetySpec) _then) = _$SafetySpecCopyWithImpl;
@useResult
$Res call({
 int hopCount, Stability stability, Sequencing sequencing, RouteId? preferredRoute
});




}
/// @nodoc
class _$SafetySpecCopyWithImpl<$Res>
    implements $SafetySpecCopyWith<$Res> {
  _$SafetySpecCopyWithImpl(this._self, this._then);

  final SafetySpec _self;
  final $Res Function(SafetySpec) _then;

/// Create a copy of SafetySpec
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? hopCount = null,Object? stability = null,Object? sequencing = null,Object? preferredRoute = freezed,}) {
  return _then(_self.copyWith(
hopCount: null == hopCount ? _self.hopCount : hopCount // ignore: cast_nullable_to_non_nullable
as int,stability: null == stability ? _self.stability : stability // ignore: cast_nullable_to_non_nullable
as Stability,sequencing: null == sequencing ? _self.sequencing : sequencing // ignore: cast_nullable_to_non_nullable
as Sequencing,preferredRoute: freezed == preferredRoute ? _self.preferredRoute : preferredRoute // ignore: cast_nullable_to_non_nullable
as RouteId?,
  ));
}

}


/// Adds pattern-matching-related methods to [SafetySpec].
extension SafetySpecPatterns on SafetySpec {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _SafetySpec value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _SafetySpec() when $default != null:
return $default(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _SafetySpec value)  $default,){
final _that = this;
switch (_that) {
case _SafetySpec():
return $default(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _SafetySpec value)?  $default,){
final _that = this;
switch (_that) {
case _SafetySpec() when $default != null:
return $default(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int hopCount,  Stability stability,  Sequencing sequencing,  RouteId? preferredRoute)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _SafetySpec() when $default != null:
return $default(_that.hopCount,_that.stability,_that.sequencing,_that.preferredRoute);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int hopCount,  Stability stability,  Sequencing sequencing,  RouteId? preferredRoute)  $default,) {final _that = this;
switch (_that) {
case _SafetySpec():
return $default(_that.hopCount,_that.stability,_that.sequencing,_that.preferredRoute);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int hopCount,  Stability stability,  Sequencing sequencing,  RouteId? preferredRoute)?  $default,) {final _that = this;
switch (_that) {
case _SafetySpec() when $default != null:
return $default(_that.hopCount,_that.stability,_that.sequencing,_that.preferredRoute);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _SafetySpec implements SafetySpec {
  const _SafetySpec({required this.hopCount, required this.stability, required this.sequencing, this.preferredRoute});
  factory _SafetySpec.fromJson(Map<String, dynamic> json) => _$SafetySpecFromJson(json);

@override final  int hopCount;
@override final  Stability stability;
@override final  Sequencing sequencing;
@override final  RouteId? preferredRoute;

/// Create a copy of SafetySpec
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$SafetySpecCopyWith<_SafetySpec> get copyWith => __$SafetySpecCopyWithImpl<_SafetySpec>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$SafetySpecToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _SafetySpec&&(identical(other.hopCount, hopCount) || other.hopCount == hopCount)&&(identical(other.stability, stability) || other.stability == stability)&&(identical(other.sequencing, sequencing) || other.sequencing == sequencing)&&(identical(other.preferredRoute, preferredRoute) || other.preferredRoute == preferredRoute));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,hopCount,stability,sequencing,preferredRoute);

@override
String toString() {
  return 'SafetySpec(hopCount: $hopCount, stability: $stability, sequencing: $sequencing, preferredRoute: $preferredRoute)';
}


}

/// @nodoc
abstract mixin class _$SafetySpecCopyWith<$Res> implements $SafetySpecCopyWith<$Res> {
  factory _$SafetySpecCopyWith(_SafetySpec value, $Res Function(_SafetySpec) _then) = __$SafetySpecCopyWithImpl;
@override @useResult
$Res call({
 int hopCount, Stability stability, Sequencing sequencing, RouteId? preferredRoute
});




}
/// @nodoc
class __$SafetySpecCopyWithImpl<$Res>
    implements _$SafetySpecCopyWith<$Res> {
  __$SafetySpecCopyWithImpl(this._self, this._then);

  final _SafetySpec _self;
  final $Res Function(_SafetySpec) _then;

/// Create a copy of SafetySpec
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? hopCount = null,Object? stability = null,Object? sequencing = null,Object? preferredRoute = freezed,}) {
  return _then(_SafetySpec(
hopCount: null == hopCount ? _self.hopCount : hopCount // ignore: cast_nullable_to_non_nullable
as int,stability: null == stability ? _self.stability : stability // ignore: cast_nullable_to_non_nullable
as Stability,sequencing: null == sequencing ? _self.sequencing : sequencing // ignore: cast_nullable_to_non_nullable
as Sequencing,preferredRoute: freezed == preferredRoute ? _self.preferredRoute : preferredRoute // ignore: cast_nullable_to_non_nullable
as RouteId?,
  ));
}


}


/// @nodoc
mixin _$RouteBlob {

 RouteId get routeId;@Uint8ListJsonConverter.jsIsArray() Uint8List get blob;
/// Create a copy of RouteBlob
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RouteBlobCopyWith<RouteBlob> get copyWith => _$RouteBlobCopyWithImpl<RouteBlob>(this as RouteBlob, _$identity);

  /// Serializes this RouteBlob to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RouteBlob&&(identical(other.routeId, routeId) || other.routeId == routeId)&&const DeepCollectionEquality().equals(other.blob, blob));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,routeId,const DeepCollectionEquality().hash(blob));

@override
String toString() {
  return 'RouteBlob(routeId: $routeId, blob: $blob)';
}


}

/// @nodoc
abstract mixin class $RouteBlobCopyWith<$Res>  {
  factory $RouteBlobCopyWith(RouteBlob value, $Res Function(RouteBlob) _then) = _$RouteBlobCopyWithImpl;
@useResult
$Res call({
 RouteId routeId,@Uint8ListJsonConverter.jsIsArray() Uint8List blob
});




}
/// @nodoc
class _$RouteBlobCopyWithImpl<$Res>
    implements $RouteBlobCopyWith<$Res> {
  _$RouteBlobCopyWithImpl(this._self, this._then);

  final RouteBlob _self;
  final $Res Function(RouteBlob) _then;

/// Create a copy of RouteBlob
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? routeId = null,Object? blob = null,}) {
  return _then(_self.copyWith(
routeId: null == routeId ? _self.routeId : routeId // ignore: cast_nullable_to_non_nullable
as RouteId,blob: null == blob ? _self.blob : blob // ignore: cast_nullable_to_non_nullable
as Uint8List,
  ));
}

}


/// Adds pattern-matching-related methods to [RouteBlob].
extension RouteBlobPatterns on RouteBlob {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _RouteBlob value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _RouteBlob() when $default != null:
return $default(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _RouteBlob value)  $default,){
final _that = this;
switch (_that) {
case _RouteBlob():
return $default(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _RouteBlob value)?  $default,){
final _that = this;
switch (_that) {
case _RouteBlob() when $default != null:
return $default(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( RouteId routeId, @Uint8ListJsonConverter.jsIsArray()  Uint8List blob)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _RouteBlob() when $default != null:
return $default(_that.routeId,_that.blob);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( RouteId routeId, @Uint8ListJsonConverter.jsIsArray()  Uint8List blob)  $default,) {final _that = this;
switch (_that) {
case _RouteBlob():
return $default(_that.routeId,_that.blob);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( RouteId routeId, @Uint8ListJsonConverter.jsIsArray()  Uint8List blob)?  $default,) {final _that = this;
switch (_that) {
case _RouteBlob() when $default != null:
return $default(_that.routeId,_that.blob);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _RouteBlob implements RouteBlob {
  const _RouteBlob({required this.routeId, @Uint8ListJsonConverter.jsIsArray() required this.blob});
  factory _RouteBlob.fromJson(Map<String, dynamic> json) => _$RouteBlobFromJson(json);

@override final  RouteId routeId;
@override@Uint8ListJsonConverter.jsIsArray() final  Uint8List blob;

/// Create a copy of RouteBlob
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$RouteBlobCopyWith<_RouteBlob> get copyWith => __$RouteBlobCopyWithImpl<_RouteBlob>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$RouteBlobToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _RouteBlob&&(identical(other.routeId, routeId) || other.routeId == routeId)&&const DeepCollectionEquality().equals(other.blob, blob));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,routeId,const DeepCollectionEquality().hash(blob));

@override
String toString() {
  return 'RouteBlob(routeId: $routeId, blob: $blob)';
}


}

/// @nodoc
abstract mixin class _$RouteBlobCopyWith<$Res> implements $RouteBlobCopyWith<$Res> {
  factory _$RouteBlobCopyWith(_RouteBlob value, $Res Function(_RouteBlob) _then) = __$RouteBlobCopyWithImpl;
@override @useResult
$Res call({
 RouteId routeId,@Uint8ListJsonConverter.jsIsArray() Uint8List blob
});




}
/// @nodoc
class __$RouteBlobCopyWithImpl<$Res>
    implements _$RouteBlobCopyWith<$Res> {
  __$RouteBlobCopyWithImpl(this._self, this._then);

  final _RouteBlob _self;
  final $Res Function(_RouteBlob) _then;

/// Create a copy of RouteBlob
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? routeId = null,Object? blob = null,}) {
  return _then(_RouteBlob(
routeId: null == routeId ? _self.routeId : routeId // ignore: cast_nullable_to_non_nullable
as RouteId,blob: null == blob ? _self.blob : blob // ignore: cast_nullable_to_non_nullable
as Uint8List,
  ));
}


}


/// @nodoc
mixin _$DHTRecordReport {

 List<ValueSubkeyRange> get subkeys; List<ValueSubkeyRange> get offlineSubkeys; List<int?> get localSeqs; List<int?> get networkSeqs;
/// Create a copy of DHTRecordReport
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DHTRecordReportCopyWith<DHTRecordReport> get copyWith => _$DHTRecordReportCopyWithImpl<DHTRecordReport>(this as DHTRecordReport, _$identity);

  /// Serializes this DHTRecordReport to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DHTRecordReport&&const DeepCollectionEquality().equals(other.subkeys, subkeys)&&const DeepCollectionEquality().equals(other.offlineSubkeys, offlineSubkeys)&&const DeepCollectionEquality().equals(other.localSeqs, localSeqs)&&const DeepCollectionEquality().equals(other.networkSeqs, networkSeqs));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(subkeys),const DeepCollectionEquality().hash(offlineSubkeys),const DeepCollectionEquality().hash(localSeqs),const DeepCollectionEquality().hash(networkSeqs));

@override
String toString() {
  return 'DHTRecordReport(subkeys: $subkeys, offlineSubkeys: $offlineSubkeys, localSeqs: $localSeqs, networkSeqs: $networkSeqs)';
}


}

/// @nodoc
abstract mixin class $DHTRecordReportCopyWith<$Res>  {
  factory $DHTRecordReportCopyWith(DHTRecordReport value, $Res Function(DHTRecordReport) _then) = _$DHTRecordReportCopyWithImpl;
@useResult
$Res call({
 List<ValueSubkeyRange> subkeys, List<ValueSubkeyRange> offlineSubkeys, List<int?> localSeqs, List<int?> networkSeqs
});




}
/// @nodoc
class _$DHTRecordReportCopyWithImpl<$Res>
    implements $DHTRecordReportCopyWith<$Res> {
  _$DHTRecordReportCopyWithImpl(this._self, this._then);

  final DHTRecordReport _self;
  final $Res Function(DHTRecordReport) _then;

/// Create a copy of DHTRecordReport
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? subkeys = null,Object? offlineSubkeys = null,Object? localSeqs = null,Object? networkSeqs = null,}) {
  return _then(_self.copyWith(
subkeys: null == subkeys ? _self.subkeys : subkeys // ignore: cast_nullable_to_non_nullable
as List<ValueSubkeyRange>,offlineSubkeys: null == offlineSubkeys ? _self.offlineSubkeys : offlineSubkeys // ignore: cast_nullable_to_non_nullable
as List<ValueSubkeyRange>,localSeqs: null == localSeqs ? _self.localSeqs : localSeqs // ignore: cast_nullable_to_non_nullable
as List<int?>,networkSeqs: null == networkSeqs ? _self.networkSeqs : networkSeqs // ignore: cast_nullable_to_non_nullable
as List<int?>,
  ));
}

}


/// Adds pattern-matching-related methods to [DHTRecordReport].
extension DHTRecordReportPatterns on DHTRecordReport {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _DHTRecordReport value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DHTRecordReport() when $default != null:
return $default(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _DHTRecordReport value)  $default,){
final _that = this;
switch (_that) {
case _DHTRecordReport():
return $default(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _DHTRecordReport value)?  $default,){
final _that = this;
switch (_that) {
case _DHTRecordReport() when $default != null:
return $default(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( List<ValueSubkeyRange> subkeys,  List<ValueSubkeyRange> offlineSubkeys,  List<int?> localSeqs,  List<int?> networkSeqs)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DHTRecordReport() when $default != null:
return $default(_that.subkeys,_that.offlineSubkeys,_that.localSeqs,_that.networkSeqs);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( List<ValueSubkeyRange> subkeys,  List<ValueSubkeyRange> offlineSubkeys,  List<int?> localSeqs,  List<int?> networkSeqs)  $default,) {final _that = this;
switch (_that) {
case _DHTRecordReport():
return $default(_that.subkeys,_that.offlineSubkeys,_that.localSeqs,_that.networkSeqs);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( List<ValueSubkeyRange> subkeys,  List<ValueSubkeyRange> offlineSubkeys,  List<int?> localSeqs,  List<int?> networkSeqs)?  $default,) {final _that = this;
switch (_that) {
case _DHTRecordReport() when $default != null:
return $default(_that.subkeys,_that.offlineSubkeys,_that.localSeqs,_that.networkSeqs);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _DHTRecordReport implements DHTRecordReport {
  const _DHTRecordReport({required final  List<ValueSubkeyRange> subkeys, required final  List<ValueSubkeyRange> offlineSubkeys, required final  List<int?> localSeqs, required final  List<int?> networkSeqs}): _subkeys = subkeys,_offlineSubkeys = offlineSubkeys,_localSeqs = localSeqs,_networkSeqs = networkSeqs;
  factory _DHTRecordReport.fromJson(Map<String, dynamic> json) => _$DHTRecordReportFromJson(json);

 final  List<ValueSubkeyRange> _subkeys;
@override List<ValueSubkeyRange> get subkeys {
  if (_subkeys is EqualUnmodifiableListView) return _subkeys;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_subkeys);
}

 final  List<ValueSubkeyRange> _offlineSubkeys;
@override List<ValueSubkeyRange> get offlineSubkeys {
  if (_offlineSubkeys is EqualUnmodifiableListView) return _offlineSubkeys;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_offlineSubkeys);
}

 final  List<int?> _localSeqs;
@override List<int?> get localSeqs {
  if (_localSeqs is EqualUnmodifiableListView) return _localSeqs;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_localSeqs);
}

 final  List<int?> _networkSeqs;
@override List<int?> get networkSeqs {
  if (_networkSeqs is EqualUnmodifiableListView) return _networkSeqs;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_networkSeqs);
}


/// Create a copy of DHTRecordReport
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DHTRecordReportCopyWith<_DHTRecordReport> get copyWith => __$DHTRecordReportCopyWithImpl<_DHTRecordReport>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$DHTRecordReportToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DHTRecordReport&&const DeepCollectionEquality().equals(other._subkeys, _subkeys)&&const DeepCollectionEquality().equals(other._offlineSubkeys, _offlineSubkeys)&&const DeepCollectionEquality().equals(other._localSeqs, _localSeqs)&&const DeepCollectionEquality().equals(other._networkSeqs, _networkSeqs));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_subkeys),const DeepCollectionEquality().hash(_offlineSubkeys),const DeepCollectionEquality().hash(_localSeqs),const DeepCollectionEquality().hash(_networkSeqs));

@override
String toString() {
  return 'DHTRecordReport(subkeys: $subkeys, offlineSubkeys: $offlineSubkeys, localSeqs: $localSeqs, networkSeqs: $networkSeqs)';
}


}

/// @nodoc
abstract mixin class _$DHTRecordReportCopyWith<$Res> implements $DHTRecordReportCopyWith<$Res> {
  factory _$DHTRecordReportCopyWith(_DHTRecordReport value, $Res Function(_DHTRecordReport) _then) = __$DHTRecordReportCopyWithImpl;
@override @useResult
$Res call({
 List<ValueSubkeyRange> subkeys, List<ValueSubkeyRange> offlineSubkeys, List<int?> localSeqs, List<int?> networkSeqs
});




}
/// @nodoc
class __$DHTRecordReportCopyWithImpl<$Res>
    implements _$DHTRecordReportCopyWith<$Res> {
  __$DHTRecordReportCopyWithImpl(this._self, this._then);

  final _DHTRecordReport _self;
  final $Res Function(_DHTRecordReport) _then;

/// Create a copy of DHTRecordReport
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? subkeys = null,Object? offlineSubkeys = null,Object? localSeqs = null,Object? networkSeqs = null,}) {
  return _then(_DHTRecordReport(
subkeys: null == subkeys ? _self._subkeys : subkeys // ignore: cast_nullable_to_non_nullable
as List<ValueSubkeyRange>,offlineSubkeys: null == offlineSubkeys ? _self._offlineSubkeys : offlineSubkeys // ignore: cast_nullable_to_non_nullable
as List<ValueSubkeyRange>,localSeqs: null == localSeqs ? _self._localSeqs : localSeqs // ignore: cast_nullable_to_non_nullable
as List<int?>,networkSeqs: null == networkSeqs ? _self._networkSeqs : networkSeqs // ignore: cast_nullable_to_non_nullable
as List<int?>,
  ));
}


}


/// @nodoc
mixin _$SetDHTValueOptions {

 KeyPair? get writer; bool? get allowOffline;
/// Create a copy of SetDHTValueOptions
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SetDHTValueOptionsCopyWith<SetDHTValueOptions> get copyWith => _$SetDHTValueOptionsCopyWithImpl<SetDHTValueOptions>(this as SetDHTValueOptions, _$identity);

  /// Serializes this SetDHTValueOptions to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SetDHTValueOptions&&(identical(other.writer, writer) || other.writer == writer)&&(identical(other.allowOffline, allowOffline) || other.allowOffline == allowOffline));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,writer,allowOffline);

@override
String toString() {
  return 'SetDHTValueOptions(writer: $writer, allowOffline: $allowOffline)';
}


}

/// @nodoc
abstract mixin class $SetDHTValueOptionsCopyWith<$Res>  {
  factory $SetDHTValueOptionsCopyWith(SetDHTValueOptions value, $Res Function(SetDHTValueOptions) _then) = _$SetDHTValueOptionsCopyWithImpl;
@useResult
$Res call({
 KeyPair? writer, bool? allowOffline
});




}
/// @nodoc
class _$SetDHTValueOptionsCopyWithImpl<$Res>
    implements $SetDHTValueOptionsCopyWith<$Res> {
  _$SetDHTValueOptionsCopyWithImpl(this._self, this._then);

  final SetDHTValueOptions _self;
  final $Res Function(SetDHTValueOptions) _then;

/// Create a copy of SetDHTValueOptions
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? writer = freezed,Object? allowOffline = freezed,}) {
  return _then(_self.copyWith(
writer: freezed == writer ? _self.writer : writer // ignore: cast_nullable_to_non_nullable
as KeyPair?,allowOffline: freezed == allowOffline ? _self.allowOffline : allowOffline // ignore: cast_nullable_to_non_nullable
as bool?,
  ));
}

}


/// Adds pattern-matching-related methods to [SetDHTValueOptions].
extension SetDHTValueOptionsPatterns on SetDHTValueOptions {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _SetDHTValueOptions value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _SetDHTValueOptions() when $default != null:
return $default(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _SetDHTValueOptions value)  $default,){
final _that = this;
switch (_that) {
case _SetDHTValueOptions():
return $default(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _SetDHTValueOptions value)?  $default,){
final _that = this;
switch (_that) {
case _SetDHTValueOptions() when $default != null:
return $default(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( KeyPair? writer,  bool? allowOffline)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _SetDHTValueOptions() when $default != null:
return $default(_that.writer,_that.allowOffline);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( KeyPair? writer,  bool? allowOffline)  $default,) {final _that = this;
switch (_that) {
case _SetDHTValueOptions():
return $default(_that.writer,_that.allowOffline);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( KeyPair? writer,  bool? allowOffline)?  $default,) {final _that = this;
switch (_that) {
case _SetDHTValueOptions() when $default != null:
return $default(_that.writer,_that.allowOffline);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _SetDHTValueOptions implements SetDHTValueOptions {
  const _SetDHTValueOptions({this.writer, this.allowOffline});
  factory _SetDHTValueOptions.fromJson(Map<String, dynamic> json) => _$SetDHTValueOptionsFromJson(json);

@override final  KeyPair? writer;
@override final  bool? allowOffline;

/// Create a copy of SetDHTValueOptions
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$SetDHTValueOptionsCopyWith<_SetDHTValueOptions> get copyWith => __$SetDHTValueOptionsCopyWithImpl<_SetDHTValueOptions>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$SetDHTValueOptionsToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _SetDHTValueOptions&&(identical(other.writer, writer) || other.writer == writer)&&(identical(other.allowOffline, allowOffline) || other.allowOffline == allowOffline));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,writer,allowOffline);

@override
String toString() {
  return 'SetDHTValueOptions(writer: $writer, allowOffline: $allowOffline)';
}


}

/// @nodoc
abstract mixin class _$SetDHTValueOptionsCopyWith<$Res> implements $SetDHTValueOptionsCopyWith<$Res> {
  factory _$SetDHTValueOptionsCopyWith(_SetDHTValueOptions value, $Res Function(_SetDHTValueOptions) _then) = __$SetDHTValueOptionsCopyWithImpl;
@override @useResult
$Res call({
 KeyPair? writer, bool? allowOffline
});




}
/// @nodoc
class __$SetDHTValueOptionsCopyWithImpl<$Res>
    implements _$SetDHTValueOptionsCopyWith<$Res> {
  __$SetDHTValueOptionsCopyWithImpl(this._self, this._then);

  final _SetDHTValueOptions _self;
  final $Res Function(_SetDHTValueOptions) _then;

/// Create a copy of SetDHTValueOptions
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? writer = freezed,Object? allowOffline = freezed,}) {
  return _then(_SetDHTValueOptions(
writer: freezed == writer ? _self.writer : writer // ignore: cast_nullable_to_non_nullable
as KeyPair?,allowOffline: freezed == allowOffline ? _self.allowOffline : allowOffline // ignore: cast_nullable_to_non_nullable
as bool?,
  ));
}


}

// dart format on
