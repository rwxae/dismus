use clap::Parser;
use dismus::fs_library::FileSystemLibrary;
use musicbrainz_rs::prelude::*;
use musicbrainz_rs::{MusicBrainzClient, entity::release::Release};
use std::{io, path::PathBuf, thread, time::Duration};

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// MusicBrainz Artist ID
    #[arg(short, long = "artist")]
    artists: Vec<String>,

    /// Path (or paths) to music files
    #[arg(required = true)]
    inputs: Vec<PathBuf>,
}

static USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("CARGO_PKG_REPOSITORY"),
    " )"
);

fn main() -> io::Result<()> {
    let args = Args::parse();
    let mut library = FileSystemLibrary::default();
    library.load(&args.inputs)?;

    let mb_client = MusicBrainzClient::new(USER_AGENT);

    for artist in &args.artists {
        let releases = Release::browse()
            .by_artist(artist)
            .with_release_groups()
            .limit(100)
            .execute_with_client(&mb_client)
            .unwrap();

        for release in releases.entities {
            if !library.has_release(&release.id) {
                println!("Missing: {}", release.title);
            }
        }

        // Force to sleep. It is required by MusicBrainz API.
        thread::sleep(Duration::from_secs(1));
    }
    Ok(())
}
