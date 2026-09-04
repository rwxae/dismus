use clap::Parser;
use dismus::fs_library::FileSystemLibrary;
use musicbrainz_rs::prelude::*;
use musicbrainz_rs::{MusicBrainzClient, entity::release::Release};
use std::{io, path::PathBuf, thread, time::Duration};

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
        let browse_result = Release::browse()
            .by_artist(artist)
            .with_artist_credits()
            .with_release_groups()
            .limit(100)
            .execute_with_client(&mb_client)
            .unwrap();

        let releases = browse_result.entities.iter().filter(|&release| {
            if !args.skip_featured {
                return true;
            }
            let first_artist = release
                .artist_credit
                .as_ref()
                .expect("Client must fetch releases with artist-credits")
                .first()
                .expect("Release must have at least one credited artist");
            &first_artist.artist.id == artist
        });

        for release in releases {
            if !library.has_release(&release.id) {
                print!("Missing: ");
                if let Some(ref date) = release.date {
                    print!("[{}] ", date);
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
                println!("{}", release.title);
            }
        }

        // Force to sleep. It is required by MusicBrainz API.
        thread::sleep(Duration::from_secs(1));
    }
    Ok(())
}
