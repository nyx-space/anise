#include <stdio.h>
#include "cspice/include/SpiceUsr.h"

// You must declare the function from your modified spke01.c
// so the linker knows about it.
typedef double doublereal;
typedef int integer;
int spke01_(doublereal *et, doublereal *record, doublereal *state);
int spkr01_(integer *handle, doublereal *descr, doublereal *et, doublereal *record);

int main() {
    // --- Setup ---
    SpiceDouble et;
    SpiceDouble descr[5];
    SpiceChar   ident[41];
    SpiceInt    handle;
    SpiceBoolean found;

    // The epoch that causes the large error
    et = 810652114.2299933;

    // Load the kernel
    furnsh_c( "data/mro.bsp" );

    // --- Find the Segment ---
    // Use spksfs_c to find the handle and descriptor for your object at 'et'
    // (This part depends on your specific SPK file's target/observer)
    spksfs_c( -74, et,68, &handle, descr, ident, &found );
    if (!found) {
        printf("Segment not found.\n");
        return 1;
    }

    // --- Get the Record ---
    SpiceDouble record[71];
    spkr01_( &handle, descr, &et, record );

    // --- Evaluate the Record ---
    // This will call YOUR modified spke01.c with all the printf statements
    SpiceDouble state[6];
    spke01_( &et, record, state );

    printf("\n--- Final State from C ---\n");
    for (int i=0; i<6; ++i) {
        printf("state[%d] = %1.16e\n", i, state[i]);
    }

    return 0;
}
