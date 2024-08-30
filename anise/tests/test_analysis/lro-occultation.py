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

start_et = 755870469.1834258 # 2023-12-15T00:01:09.183425793 ET 
end_et = 758030469.1841377 # 2024-01-09T00:01:09.184137727 ET

# Input window
window = sp.stypes.SPICEDOUBLE_CELL(1000)
sp.wninsd(start_et, end_et, window)

# Create a result window
result = sp.stypes.SPICEDOUBLE_CELL(1000)

# Perform the occultation search
sp.occult(occtyp, front, fshape, fframe, back, bshape, bframe, abcorr, obsrvr, stepsz, cnfine, result)

# Process and display results
for i in range(sp.wncard(result)):
    [start, stop] = sp.wnfetd(result, i)
    print(f"Occultation from {sp.timout(start, 'YYYY MON DD HR:MN:SC.###')} to {sp.timout(stop, 'YYYY MON DD HR:MN:SC.###')}")
