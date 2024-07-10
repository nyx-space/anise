#[cfg(feature = "embed_ephem")]
fn main() {
    // Download the files to embed at build time.
    use std::{fs::File, io::Write, time::Duration};
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

    let embedded_files = [
        (
            "http://public-data.nyxspace.com/anise/v0.4/pck11.pca",
            format!("{}/../data/pck11.pca", env!("CARGO_MANIFEST_DIR")),
        ),
        (
            "http://public-data.nyxspace.com/anise/de440s.bsp",
            format!("{}/../data/de440s.bsp", env!("CARGO_MANIFEST_DIR")),
        ),
    ];

    for (url, dest_path) in embedded_files {
        let resp = client
            .get(url)
            .send()
            .expect(&format!("could not download {url}"));

        let bytes = resp
            .bytes()
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
