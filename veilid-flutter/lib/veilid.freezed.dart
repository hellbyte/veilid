// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'veilid.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$TransactDHTRecordsOptions {

 KeyPair? get defaultSigningKeyPair;
/// Create a copy of TransactDHTRecordsOptions
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TransactDHTRecordsOptionsCopyWith<TransactDHTRecordsOptions> get copyWith => _$TransactDHTRecordsOptionsCopyWithImpl<TransactDHTRecordsOptions>(this as TransactDHTRecordsOptions, _$identity);

  /// Serializes this TransactDHTRecordsOptions to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TransactDHTRecordsOptions&&(identical(other.defaultSigningKeyPair, defaultSigningKeyPair) || other.defaultSigningKeyPair == defaultSigningKeyPair));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,defaultSigningKeyPair);

@override
String toString() {
  return 'TransactDHTRecordsOptions(defaultSigningKeyPair: $defaultSigningKeyPair)';
}


}

/// @nodoc
abstract mixin class $TransactDHTRecordsOptionsCopyWith<$Res>  {
  factory $TransactDHTRecordsOptionsCopyWith(TransactDHTRecordsOptions value, $Res Function(TransactDHTRecordsOptions) _then) = _$TransactDHTRecordsOptionsCopyWithImpl;
@useResult
$Res call({
 KeyPair? defaultSigningKeyPair
});




}
/// @nodoc
class _$TransactDHTRecordsOptionsCopyWithImpl<$Res>
    implements $TransactDHTRecordsOptionsCopyWith<$Res> {
  _$TransactDHTRecordsOptionsCopyWithImpl(this._self, this._then);

  final TransactDHTRecordsOptions _self;
  final $Res Function(TransactDHTRecordsOptions) _then;

/// Create a copy of TransactDHTRecordsOptions
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? defaultSigningKeyPair = freezed,}) {
  return _then(_self.copyWith(
defaultSigningKeyPair: freezed == defaultSigningKeyPair ? _self.defaultSigningKeyPair : defaultSigningKeyPair // ignore: cast_nullable_to_non_nullable
as KeyPair?,
  ));
}

}


/// Adds pattern-matching-related methods to [TransactDHTRecordsOptions].
extension TransactDHTRecordsOptionsPatterns on TransactDHTRecordsOptions {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _TransactDHTRecordsOptions value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _TransactDHTRecordsOptions() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _TransactDHTRecordsOptions value)  $default,){
final _that = this;
switch (_that) {
case _TransactDHTRecordsOptions():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _TransactDHTRecordsOptions value)?  $default,){
final _that = this;
switch (_that) {
case _TransactDHTRecordsOptions() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( KeyPair? defaultSigningKeyPair)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _TransactDHTRecordsOptions() when $default != null:
return $default(_that.defaultSigningKeyPair);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( KeyPair? defaultSigningKeyPair)  $default,) {final _that = this;
switch (_that) {
case _TransactDHTRecordsOptions():
return $default(_that.defaultSigningKeyPair);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( KeyPair? defaultSigningKeyPair)?  $default,) {final _that = this;
switch (_that) {
case _TransactDHTRecordsOptions() when $default != null:
return $default(_that.defaultSigningKeyPair);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _TransactDHTRecordsOptions implements TransactDHTRecordsOptions {
  const _TransactDHTRecordsOptions({this.defaultSigningKeyPair});
  factory _TransactDHTRecordsOptions.fromJson(Map<String, dynamic> json) => _$TransactDHTRecordsOptionsFromJson(json);

@override final  KeyPair? defaultSigningKeyPair;

/// Create a copy of TransactDHTRecordsOptions
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$TransactDHTRecordsOptionsCopyWith<_TransactDHTRecordsOptions> get copyWith => __$TransactDHTRecordsOptionsCopyWithImpl<_TransactDHTRecordsOptions>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$TransactDHTRecordsOptionsToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _TransactDHTRecordsOptions&&(identical(other.defaultSigningKeyPair, defaultSigningKeyPair) || other.defaultSigningKeyPair == defaultSigningKeyPair));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,defaultSigningKeyPair);

@override
String toString() {
  return 'TransactDHTRecordsOptions(defaultSigningKeyPair: $defaultSigningKeyPair)';
}


}

/// @nodoc
abstract mixin class _$TransactDHTRecordsOptionsCopyWith<$Res> implements $TransactDHTRecordsOptionsCopyWith<$Res> {
  factory _$TransactDHTRecordsOptionsCopyWith(_TransactDHTRecordsOptions value, $Res Function(_TransactDHTRecordsOptions) _then) = __$TransactDHTRecordsOptionsCopyWithImpl;
@override @useResult
$Res call({
 KeyPair? defaultSigningKeyPair
});




}
/// @nodoc
class __$TransactDHTRecordsOptionsCopyWithImpl<$Res>
    implements _$TransactDHTRecordsOptionsCopyWith<$Res> {
  __$TransactDHTRecordsOptionsCopyWithImpl(this._self, this._then);

  final _TransactDHTRecordsOptions _self;
  final $Res Function(_TransactDHTRecordsOptions) _then;

/// Create a copy of TransactDHTRecordsOptions
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? defaultSigningKeyPair = freezed,}) {
  return _then(_TransactDHTRecordsOptions(
defaultSigningKeyPair: freezed == defaultSigningKeyPair ? _self.defaultSigningKeyPair : defaultSigningKeyPair // ignore: cast_nullable_to_non_nullable
as KeyPair?,
  ));
}


}

// dart format on
