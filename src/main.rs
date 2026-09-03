use clap::Parser;
use dismus::fs_library::FileSystemLibrary;
use musicbrainz_rs::entity::release_group::ReleaseGroup;
use musicbrainz_rs::prelude::*;
use std::{io, path::PathBuf};

#[derive(Parser)]
#[command(version, about)]
struct Args {
    library_path: PathBuf,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let library = FileSystemLibrary::new(&args.library_path).load()?;

    for artist in library.get_artists() {
        let release_groups = ReleaseGroup::browse()
            .by_artist(artist)
            .limit(100)
            .execute()
            .unwrap();
        for release_group in release_groups.entities {
            if !library.has_release_group(&release_group.id) {
                println!("Missing: {}", release_group.title);
            }
        }
    }

    Ok(())
}
