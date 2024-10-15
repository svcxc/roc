module [TypeId, fromU64, toU64]

TypeId := U64 implements [Eq, Hash, Inspect, Encoding]

# renamed here so we can import the functions directly as a workaround for
# https://github.com/roc-lang/roc/issues/5477
toU64 : TypeId -> U64
toU64 = \@TypeId x -> x

fromU64 : U64 -> TypeId
fromU64 = @TypeId
