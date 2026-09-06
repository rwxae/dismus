use clap::Parser;
use dismus::fs_library::FSLibraryScanner;
use dismus::musicbrainz::{BrowseQueryExt, MUSIC_BRAINZ_VARIOUS_ARTISTS_ID, MUSICBRAINZ_CLIENT};
use dismus::reports::ArtistReport;
use musicbrainz_rs::entity::release::Release;
use musicbrainz_rs::prelude::*;
use std::io::ErrorKind;
use std::{io, path::PathBuf};

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// MusicBrainz Artist ID
    #[arg(short, long = "artist", required = true)]
    artists: Vec<String>,

    /// Skip releases where the artist appears as a featured artist
    #[arg(long)]
    skip_featured: bool,

    /// Path (or paths) to music files
    #[arg(required = true)]
    inputs: Vec<PathBuf>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();
    // TODO: spawn blocking thread and send via watch channel
    let library = FSLibraryScanner::default().scan(&args.inputs);

    // TODO: use tokio::JoinSet.
    for artist in &args.artists {
        if artist == MUSIC_BRAINZ_VARIOUS_ARTISTS_ID {
            eprintln!("'Various Artists' has too many releases to process");
            continue;
        }
        let releases = Release::browse()
            .by_artist(artist)
            .with_artist_credits()
            .with_release_groups()
            .execute_all_with_client_async(&MUSICBRAINZ_CLIENT)
            .await
            // TODO: better error handling
            .map_err(|_| io::Error::from(ErrorKind::ConnectionAborted))?;

        ArtistReport::new(artist, &library, &releases)
            .skip_featured(args.skip_featured)
            .execute();
    }

    Ok(())
}
