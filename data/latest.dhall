-- Latest planetary ephemerides, planetary constants, high precision Moon rotation, and daily Earth orientation parameter
{ files =
  [ { crc32 = Some 0x7286750a
    , uri = "http://public-data.nyxspace.com/anise/de440s.bsp"
    }
  , { crc32 = Some 0x8213b6e9
    , uri = "http://public-data.nyxspace.com/anise/v0.5/pck11.pca"
    }
  , { crc32 = Some 0x6f0ad74c
    , uri = "http://public-data.nyxspace.com/anise/v0.5/moon_fk_de440.epa"
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
