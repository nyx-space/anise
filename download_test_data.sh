#!/bin/bash
mkdir -p data

# Define a function to download only if missing or if it's a Git LFS pointer
download_if_missing() {
    url="$1"
    # Ensure we strip any existing paths from the second argument to enforce the data/ directory
    filename=$(basename "$2")
    file="data/$filename"
    
    should_download=false

    if [ ! -f "$file" ]; then
        should_download=true
    else
        # File exists, check if it is an LFS pointer.
        # 1. Check size: LFS pointers are text files ~130 bytes. 
        #    Real SPICE kernels are binary and much larger.
        #    We use 300 bytes as a safe upper bound for a pointer.
        fsize=$(wc -c < "$file" | tr -d ' ')
        
        if [ "$fsize" -lt 300 ]; then
            # 2. Check content: Look for the LFS signature url
            if grep -q "version https://git-lfs.github.com" "$file"; then
                echo "Detected Git LFS pointer for $filename ($fsize bytes). Deleting..."
                rm "$file"
                should_download=true
            fi
        fi
    fi

    if [ "$should_download" = true ]; then
        echo "Downloading $filename..."
        wget -q -O "$file" "$url"
    else
        echo "Found $file (valid), skipping download."
    fi
}
download_if_missing "http://public-data.nyxspace.com/anise/de421.bsp" "de421.bsp"
download_if_missing "http://public-data.nyxspace.com/anise/de430.bsp" "de430.bsp"
download_if_missing "http://public-data.nyxspace.com/anise/de440s.bsp" "de440s.bsp"
download_if_missing "http://public-data.nyxspace.com/anise/de440.bsp" "de440.bsp"
download_if_missing "http://public-data.nyxspace.com/anise/de440_type3.bsp" "de440_type3.bsp"
download_if_missing "http://public-data.nyxspace.com/anise/v0.5/pck08.pca" "pck08.pca"
download_if_missing "http://public-data.nyxspace.com/anise/v0.5/pck11.pca" "pck11.pca"
download_if_missing "http://public-data.nyxspace.com/anise/v0.5/moon_fk.epa" "moon_fk.epa"
download_if_missing "http://public-data.nyxspace.com/anise/v0.5/moon_fk_de440.epa" "moon_fk_de440.epa"
download_if_missing "http://public-data.nyxspace.com/anise/moon_pa_de440_200625.bpc" "moon_pa_de440_200625.bpc"
download_if_missing "http://public-data.nyxspace.com/anise/ci/gmat-hermite.bsp" "gmat-hermite.bsp"
download_if_missing "http://public-data.nyxspace.com/anise/ci/gmat-hermite-big-endian.bsp" "gmat-hermite-big-endian.bsp"
download_if_missing "http://public-data.nyxspace.com/anise/ci/variable-seg-size-hermite.bsp" "variable-seg-size-hermite.bsp"
download_if_missing "http://public-data.nyxspace.com/anise/ci/earth_latest_high_prec-2023-09-08.bpc" "earth_latest_high_prec.bpc"
download_if_missing "http://public-data.nyxspace.com/nyx/examples/lrorg_2023349_2024075_v01_LE.bsp" "lro.bsp"
download_if_missing "http://public-data.nyxspace.com/anise/ci/mro.bsp" "mro.bsp"
download_if_missing "http://public-data.nyxspace.com/anise/ci/earth_2025_250826_2125_predict.bpc" "data/earth_2025_250826_2125_predict.bpc"
