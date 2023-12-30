-- Example Dhall meta "kernel"
let MetaFile
    : Type
    = { uri : Text, crc32 : Optional Natural }

let Meta
    : Type
    = { files : List MetaFile }

let NyxAsset
    : Text -> Text
    = \(file : Text) -> "http://public-data.nyxspace.com/anise/${file}"

let JplAsset
    : Text -> Text
    = \(file : Text) ->
        "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/${file}"

let buildNyxAsset
    : Text -> MetaFile
    = \(file : Text) ->
        let crc32 = None Natural

        let uri
            : Text
            = NyxAsset file

        let thisAsset
            : MetaFile
            = { uri, crc32 }

        in  thisAsset

let buildJplAsset
    : Text -> MetaFile
    = \(file : Text) ->
        let crc32 = None Natural

        let uri
            : Text
            = JplAsset file

        let thisAsset
            : MetaFile
            = { uri, crc32 }

        in  thisAsset

in  { files =
      [ buildNyxAsset "de440s.bsp"
      , buildNyxAsset "pck08.pca"
      , buildJplAsset "pck/earth_latest_high_prec.bpc"
      ]
    }
