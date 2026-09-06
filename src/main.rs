use clap::Parser;
use dismus::fs_library::FSLibraryScanner;
use dismus::musicbrainz::{BrowseQueryExt, USER_AGENT};
use musicbrainz_rs::prelude::*;
use musicbrainz_rs::{MusicBrainzClient, entity::release::Release};
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

    let mb_client = MusicBrainzClient::new(USER_AGENT);

    // TODO: use tokio::JoinSet.
    for artist in &args.artists {
        let releases = Release::browse()
            .by_artist(artist)
            .with_artist_credits()
            .with_release_groups()
            .execute_all_with_client_async(&mb_client)
            .await
            // TODO: better error handling
            .map_err(|_| io::Error::from(ErrorKind::ConnectionAborted))?;

        // TODO: perform filtering logic differently
        let releases = releases
            .into_iter()
            .filter(|release| {
                if !args.skip_featured {
                    return true;
                }
                let first_artist = release
                    .artist_credit
                    .as_ref()
                    .expect("Client must fetch releases with artist-credits")
                    .first()
                    .expect("Release must have at least one credited artist");
                first_artist.artist.id == *artist
            })
            .collect::<Vec<_>>();

        for release in releases {
            if !library.has_release(&release.id) {
                print!("Missing: ");
                let url = mb_client
                    .endpoints()
                    .endpoint_builder()
                    .add_path_fragment("release")
                    .add_path_fragment(&release.id)
                    .to_string();
                print!("\x1b]8;;");
                print!("{url}");
                print!("\x1b\\");
                if let Some(ref date) = release.date {
                    print!("[{date}] ");
                }
                if let Some(ref artist_credit) = release.artist_credit {
                    for artist in artist_credit {
                        print!(
                            "{}{}",
                            artist.name,
                            artist.joinphrase.as_deref().unwrap_or("")
                        );
                    }
                    if !artist_credit.is_empty() {
                        print!(" - ");
                    }
                }
                print!("{}", release.title);
                println!("\x1b]8;;\x1b\\");
            }
        }
    }
    Ok(())
}
