use clap::Parser;
use dismus::fs_library::FSLibraryScanner;
use dismus::musicbrainz::{BrowseQueryExt, MUSIC_BRAINZ_VARIOUS_ARTISTS_ID, MUSICBRAINZ_CLIENT};
use dismus::reports::ArtistReport;
use musicbrainz_rs::entity::release::Release;
use musicbrainz_rs::prelude::*;
use std::{io, path::PathBuf};
use tokio::task::{JoinSet, spawn_blocking};

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

    let inputs = args.inputs.clone();
    let library_scan_job = spawn_blocking(move || FSLibraryScanner::default().scan(&inputs));

    let mut artists_jobs: JoinSet<_> = args
        .artists
        .clone()
        .into_iter()
        .filter(|artist| artist != MUSIC_BRAINZ_VARIOUS_ARTISTS_ID)
        .map(async |artist| {
            let releases = Release::browse()
                .by_artist(&artist)
                .with_artist_credits()
                .with_release_groups()
                .execute_all_with_client_async(&MUSICBRAINZ_CLIENT)
                .await
                .unwrap_or_else(|_| panic!("Could not fetch releases for artist '{}'", artist));
            (artist, releases)
        })
        .collect();

    let library = library_scan_job.await?;

    while let Some(artist_data) = artists_jobs.join_next().await {
        let (artist, releases) = artist_data?;
        ArtistReport::new(&artist, &library, &releases)
            .skip_featured(args.skip_featured)
            .execute();
    }

    Ok(())
}
