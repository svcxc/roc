platform "test-platform"
    requires {} { main : _ }
    exposes []
    packages {}
    imports []
    provides [mainForHost]

Expr : [
    String Str,
    Concat Expr Expr,
    Tag3 I8,
    Tag4 I8,
    Tag5 I8,
    Tag6 I8,
    Tag7 I8,
    Tag8 I8,
    Tag9 I8,
]

mainForHost : {} -> Expr
mainForHost = \{} -> main
