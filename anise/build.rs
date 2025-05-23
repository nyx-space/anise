#[cfg(feature = "embed_ephem")]
fn main() {
    // Download the files to embed at build time.
    use std::{
        fs::{self, File},
        io::Write,
        path::Path,
        time::Duration,
    };

    let client: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .build()
        .into();

    let embedded_files = [
        (
            "http://public-data.nyxspace.com/anise/v0.5/pck11.pca",
            format!("{}/../data/pck11.pca", env!("CARGO_MANIFEST_DIR")),
        ),
        (
            "http://public-data.nyxspace.com/anise/de440s.bsp",
            format!("{}/../data/de440s.bsp", env!("CARGO_MANIFEST_DIR")),
        ),
    ];

    let data_path = Path::new(&env!("CARGO_MANIFEST_DIR")).join("../data");

    // Create the directory if it doesn't exist
    if !data_path.exists() {
        if fs::create_dir_all(&data_path).is_err() {
            eprintln!("EMBEDDED EPHEM UNAVAILABLE: failed to create directory {data_path:?}");
            // Try nothing else.
            return;
        }
    }

    for (url, dest_path) in embedded_files {
        let mut resp = client
            .get(url)
            .call()
            .expect(&format!("could not download {url}"));

        let bytes = resp
            .body_mut()
            .with_config()
            .limit(1024 * 1024 * 200) // 200 MB limit
            .read_to_vec()
            .expect(&format!("could not read bytes from {url}"));

        let mut file =
            File::create(&dest_path).expect(&format!("could not create the data path {dest_path}"));
        file.write_all(&bytes)
            .expect(&format!("could not write asset data to {dest_path}"));
    }
}

#[cfg(not(feature = "embed_ephem"))]
fn main() {
    // Nothing to do if we aren't embedded files.
}
