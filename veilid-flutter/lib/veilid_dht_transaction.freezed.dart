// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'veilid_dht_transaction.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$DHTTransactionSetValueOptions {

 KeyPair? get writer;
/// Create a copy of DHTTransactionSetValueOptions
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DHTTransactionSetValueOptionsCopyWith<DHTTransactionSetValueOptions> get copyWith => _$DHTTransactionSetValueOptionsCopyWithImpl<DHTTransactionSetValueOptions>(this as DHTTransactionSetValueOptions, _$identity);

  /// Serializes this DHTTransactionSetValueOptions to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DHTTransactionSetValueOptions&&(identical(other.writer, writer) || other.writer == writer));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,writer);

@override
String toString() {
  return 'DHTTransactionSetValueOptions(writer: $writer)';
}


}

/// @nodoc
abstract mixin class $DHTTransactionSetValueOptionsCopyWith<$Res>  {
  factory $DHTTransactionSetValueOptionsCopyWith(DHTTransactionSetValueOptions value, $Res Function(DHTTransactionSetValueOptions) _then) = _$DHTTransactionSetValueOptionsCopyWithImpl;
@useResult
$Res call({
 KeyPair? writer
});




}
/// @nodoc
class _$DHTTransactionSetValueOptionsCopyWithImpl<$Res>
    implements $DHTTransactionSetValueOptionsCopyWith<$Res> {
  _$DHTTransactionSetValueOptionsCopyWithImpl(this._self, this._then);

  final DHTTransactionSetValueOptions _self;
  final $Res Function(DHTTransactionSetValueOptions) _then;

/// Create a copy of DHTTransactionSetValueOptions
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? writer = freezed,}) {
  return _then(_self.copyWith(
writer: freezed == writer ? _self.writer : writer // ignore: cast_nullable_to_non_nullable
as KeyPair?,
  ));
}

}


/// Adds pattern-matching-related methods to [DHTTransactionSetValueOptions].
extension DHTTransactionSetValueOptionsPatterns on DHTTransactionSetValueOptions {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _DHTTransactionSetValueOptions value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _DHTTransactionSetValueOptions() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _DHTTransactionSetValueOptions value)  $default,){
final _that = this;
switch (_that) {
case _DHTTransactionSetValueOptions():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _DHTTransactionSetValueOptions value)?  $default,){
final _that = this;
switch (_that) {
case _DHTTransactionSetValueOptions() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( KeyPair? writer)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _DHTTransactionSetValueOptions() when $default != null:
return $default(_that.writer);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( KeyPair? writer)  $default,) {final _that = this;
switch (_that) {
case _DHTTransactionSetValueOptions():
return $default(_that.writer);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( KeyPair? writer)?  $default,) {final _that = this;
switch (_that) {
case _DHTTransactionSetValueOptions() when $default != null:
return $default(_that.writer);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _DHTTransactionSetValueOptions implements DHTTransactionSetValueOptions {
  const _DHTTransactionSetValueOptions({this.writer});
  factory _DHTTransactionSetValueOptions.fromJson(Map<String, dynamic> json) => _$DHTTransactionSetValueOptionsFromJson(json);

@override final  KeyPair? writer;

/// Create a copy of DHTTransactionSetValueOptions
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$DHTTransactionSetValueOptionsCopyWith<_DHTTransactionSetValueOptions> get copyWith => __$DHTTransactionSetValueOptionsCopyWithImpl<_DHTTransactionSetValueOptions>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$DHTTransactionSetValueOptionsToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _DHTTransactionSetValueOptions&&(identical(other.writer, writer) || other.writer == writer));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,writer);

@override
String toString() {
  return 'DHTTransactionSetValueOptions(writer: $writer)';
}


}

/// @nodoc
abstract mixin class _$DHTTransactionSetValueOptionsCopyWith<$Res> implements $DHTTransactionSetValueOptionsCopyWith<$Res> {
  factory _$DHTTransactionSetValueOptionsCopyWith(_DHTTransactionSetValueOptions value, $Res Function(_DHTTransactionSetValueOptions) _then) = __$DHTTransactionSetValueOptionsCopyWithImpl;
@override @useResult
$Res call({
 KeyPair? writer
});




}
/// @nodoc
class __$DHTTransactionSetValueOptionsCopyWithImpl<$Res>
    implements _$DHTTransactionSetValueOptionsCopyWith<$Res> {
  __$DHTTransactionSetValueOptionsCopyWithImpl(this._self, this._then);

  final _DHTTransactionSetValueOptions _self;
  final $Res Function(_DHTTransactionSetValueOptions) _then;

/// Create a copy of DHTTransactionSetValueOptions
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? writer = freezed,}) {
  return _then(_DHTTransactionSetValueOptions(
writer: freezed == writer ? _self.writer : writer // ignore: cast_nullable_to_non_nullable
as KeyPair?,
  ));
}


}

// dart format on
