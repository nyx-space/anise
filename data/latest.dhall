-- Latest planetary ephemerides, planetary constants, high precision Moon rotation, and daily Earth orientation parameter
{ files =
  [ { crc32 = Some 0x7286750a
    , uri = "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440s.bsp"
    }
  , { crc32 = Some 0xf151b735
    , uri = "http://public-data.nyxspace.com/anise/v0.10/pck11.pca"
    }
  , { crc32 = Some 0x32c8f9d7
    , uri = "http://public-data.nyxspace.com/anise/v0.10/moon_fk_de440.epa"
    }
  , { crc32 = Some 0xcde5ca7d
    , uri = "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/moon_pa_de440_200625.bpc"
    }
  , { crc32 = None Natural
    , uri =
        "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/earth_latest_high_prec.bpc"
    }
  ]
}
