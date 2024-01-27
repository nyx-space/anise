-- Latest planetary ephemerides, planetary constants, high precision Moon rotation, and daily Earth orientation parameter
{ files =
  [ { crc32 = Some 1921414410
    , uri = "http://public-data.nyxspace.com/anise/de440s.bsp"
    }
  , { crc32 = Some 2899443223
    , uri = "http://public-data.nyxspace.com/anise/v0.3/pck11.pca"
    }
  , { crc32 = Some 2133296540
    , uri = "http://public-data.nyxspace.com/anise/v0.3/moon_fk.epa"
    }
  , { crc32 = Some 1817759242
    , uri = "http://public-data.nyxspace.com/anise/moon_pa_de440_200625.bpc"
    }
  , { crc32 = None Natural
    , uri =
        "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/earth_latest_high_prec.bpc"
    }
  ]
}
