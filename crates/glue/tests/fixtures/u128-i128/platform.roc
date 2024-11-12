platform "test-platform"
    requires {} { main : _ }
    exposes []
    packages {}
    imports []
    provides [mainForHost]

Numbers : { u128 : U128, i128 : I128 }

mainForHost : Numbers -> Numbers
mainForHost = \ns -> main ns
