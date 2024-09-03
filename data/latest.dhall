-- Latest planetary ephemerides, planetary constants, high precision Moon rotation, and daily Earth orientation parameter
{ files =
  [ { crc32 = Some 0x7286750a
    , uri = "http://public-data.nyxspace.com/anise/de440s.bsp"
    }
  , { crc32 = Some 0x8213b6e9
    , uri = "http://public-data.nyxspace.com/anise/v0.4/pck11.pca"
    }
  , { crc32 = Some 0xb93ba21
    , uri = "http://public-data.nyxspace.com/anise/v0.4/moon_fk.epa"
    }
  , { crc32 = Some 0xcde5ca7d
    , uri = "http://public-data.nyxspace.com/anise/moon_pa_de440_200625.bpc"
    }
  , { crc32 = None Natural
    , uri =
        "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/earth_latest_high_prec.bpc"
    }
  ]
}
