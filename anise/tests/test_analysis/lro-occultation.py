import spiceypy as sp

# Load kernels
sp.furnsh('../../../data/lro.bsp')
sp.furnsh('../../../data/de440s.bsp')
sp.furnsh('../../../data/pck00008.tpc')

# Define parameters
occtyp = 'ANY'
front = 'MOON'
fshape = 'ELLIPSOID'
fframe = 'IAU_MOON'
back = 'SUN'
bshape = 'ELLIPSOID'
bframe = 'IAU_SUN'
abcorr = 'NONE'
obsrvr = '-85' # LRO
stepsz = 5.0  # Step size in seconds
cnfine = sp.stypes.SPICEDOUBLE_CELL(2)

prenumbra_start_et = 757339371.1839062 # 2024-01-01T00:01:42 UTC
prenumbra_end_et = 757339383.1839062 # 2024-01-01T00:01:54 UTC
pre_prenumbra_start_et = prenumbra_start_et - 1.0

umbra_et = 757339384.1839062 # 2024-01-01T00:01:55 UTC:

for epoch, rslt in [(pre_prenumbra_start_et, 0), (prenumbra_start_et, 1), (prenumbra_end_et, 1), (umbra_et, 3)]:
    # Compute occultation
    occult = sp.occult(front, fshape, fframe, back, bshape, bframe, abcorr, obsrvr, epoch)
    assert occult == rslt, f"want {rslt} got {occult}"

