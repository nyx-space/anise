# This shell script sets up the development environment on Linux
curl https://naif.jpl.nasa.gov/pub/naif/toolkit//C/PC_Linux_GCC_64bit/packages/cspice.tar.Z --output cspice.tar.Z
tar xzvf cspice.tar.Z
cd cspice
tcsh makeall.csh
mv lib/cspice.a lib/libcspice.a
