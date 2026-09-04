use clap::Parser;
use dismus::fs_library::FileSystemLibrary;
use musicbrainz_rs::entity::release::Release;
use musicbrainz_rs::prelude::*;
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

fn main() -> io::Result<()> {
    let args = Args::parse();
    let mut library = FileSystemLibrary::default();
    library.load(&args.inputs)?;

    for artist in &args.artists {
        let releases = Release::browse()
            .by_artist(artist)
            .with_release_groups()
            .limit(100)
            .execute()
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
