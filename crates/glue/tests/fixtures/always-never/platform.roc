platform "test-platform"
    requires {} { main : _ }
    exposes []
    packages {}
    imports []
    provides [mainForHost]

# A recursive tag union where the discriminant can't be stored using pointer tagging.
AlwaysNever : [
    Always U32,
    Never [],
]

mainForHost : AlwaysNever
mainForHost = main
