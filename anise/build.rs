fn main() {
    #[cfg(not(doc))]
    #[cfg(feature = "embed_ephem")]
    embed_ephem();
}

#[cfg(not(doc))]
#[cfg(feature = "embed_ephem")]
fn embed_ephem() {

    // Download the files to embed at build time.
    use std::{
        fs::{self, File},
        io::Write,
        path::Path,
        time::Duration,
    };
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

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
    if !data_path.exists() && fs::create_dir_all(&data_path).is_err() {
        eprintln!("EMBEDDED EPHEM UNAVAILABLE: failed to create directory {data_path:?}");
        // Try nothing else.
        return;
    }

    for (url, dest_path) in embedded_files {
        let resp = client
            .get(url)
            .send()
            .unwrap_or_else(|e| panic!("could not download {url}: {}", e));

        let bytes = resp
            .bytes()
            .unwrap_or_else(|e| panic!("could not read bytes from {url}: {}", e));

        let mut file = File::create(&dest_path)
            .unwrap_or_else(|e| panic!("could not create the data path {dest_path}: {}", e));

        file.write_all(&bytes)
            .unwrap_or_else(|e| panic!("could not write asset to {dest_path}: {}", e));
    }
}
