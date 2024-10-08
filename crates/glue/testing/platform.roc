platform "reproduction"
    requires {} { fromRocApp : U32 -> (U16 -> U8) }
    exposes []
    packages {}
    imports []
    provides [forHost]

forHost = fromRocApp
