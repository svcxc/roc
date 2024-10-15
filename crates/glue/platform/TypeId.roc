module [TypeId, fromU64, toU64]

TypeId := U64 implements [Eq, Hash, Inspect, Encoding]

toU64 : TypeId -> U64
toU64 = \@TypeId x -> x

fromU64 : U64 -> TypeId
fromU64 = @TypeId
