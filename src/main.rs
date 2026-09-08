use clap::Parser;
use dismus::fs_library::FSLibraryScanner;
use dismus::musicbrainz::{
    BrowseQueryExt, MUSIC_BRAINZ_VARIOUS_ARTISTS_ID, MUSICBRAINZ_CLIENT, MusicBrainzReleaseType,
};
use dismus::reports::ArtistReport;
use musicbrainz_rs::entity::release::Release;
use musicbrainz_rs::entity::release_group::ReleaseGroupPrimaryType;
use musicbrainz_rs::prelude::*;
use std::{io, path::PathBuf};
use tokio::task::{JoinSet, spawn_blocking};
use url::Url;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// MusicBrainz Artist ID or URL
    #[arg(short, long = "artist", required = true, value_parser = parse_artist)]
    artists: Vec<String>,

    /// The type of a MusicBrainz release
    #[arg(long = "release-type", value_enum)]
    release_types: Vec<MusicBrainzReleaseType>,

    /// Skip releases where the artist appears as a featured artist
    #[arg(long)]
    skip_featured: bool,

    /// Include all releases belonging to a release group
    #[arg(long)]
    include_all_from_group: bool,

    /// Include singles that also appear as tracks on an album
    #[arg(long)]
    include_album_singles: bool,

    /// Path (or paths) to music files
    #[arg(required = true)]
    inputs: Vec<PathBuf>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let Args {
        artists,
        release_types,
        skip_featured,
        include_all_from_group,
        include_album_singles,
        inputs,
    } = Args::parse();

    let library_scan_job = spawn_blocking(move || FSLibraryScanner::default().scan(&inputs));

    let mut artists_jobs: JoinSet<_> = artists
        .into_iter()
        .filter(|artist| artist != MUSIC_BRAINZ_VARIOUS_ARTISTS_ID)
        .map(async |artist| {
            let releases = Release::browse()
                .by_artist(&artist)
                .with_artist_credits()
                .with_release_groups()
                .with_recordings()
                .execute_all_with_client_async(&MUSICBRAINZ_CLIENT)
                .await
                .unwrap_or_else(|_| panic!("Could not fetch releases for artist '{}'", artist));
            (artist, releases)
        })
        .collect();

    let library = library_scan_job.await?;

    let release_types: Vec<ReleaseGroupPrimaryType> =
        release_types.into_iter().map(|v| v.into()).collect();

    while let Some(artist_data) = artists_jobs.join_next().await {
        let (artist, releases) = artist_data?;
        ArtistReport::new(&artist, &library, &releases, &release_types)
            .skip_featured(skip_featured)
            .include_all_from_group(include_all_from_group)
            .include_album_singles(include_album_singles)
            .execute();
    }

    Ok(())
}

fn parse_artist(id_or_url: &str) -> Result<String, String> {
    let Ok(url) = Url::parse(id_or_url) else {
        return Ok(id_or_url.to_string());
    };
    let Some(mut segments) = url.path_segments() else {
        return Err("url doesn't contain path segments".to_string());
    };
    while let Some(segment) = segments.next() {
        if segment == "artist" {
            return match segments.next() {
                Some(id) => Ok(id.to_string()),
                None => Err("url doesn't contain artist id".to_string()),
            };
        }
    }
    Err("url doesn't contain artist id".to_string())
}
