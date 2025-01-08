platform "test-platform"
    requires {} { main : _ }
    exposes []
    packages {}
    imports []
    provides [mainForHost]

# this tag union is eligable to store the discriminant via pointer tagging
# on 64-bit architectures, but not on 32-bit ones. This test tests if this
# determination is made seperately for each architecture.
Expr : [
    String Str,
    Concat Expr Expr,
    Tag3 I8,
    Tag4 I8,
    Tag5 I8,
]

mainForHost : {} -> Expr
mainForHost = \{} -> main
