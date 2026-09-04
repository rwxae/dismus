use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, ErrorKind},
    path::Path,
};

use lofty::{
    file::{EXTENSIONS, TaggedFileExt},
    tag::{ItemKey, Tag},
};

use crate::metadata::Release;

#[derive(Default)]
pub struct FileSystemLibrary {
    releases: HashMap<String, Release>,
}

impl FileSystemLibrary {
    pub fn load<T: AsRef<Path>>(&mut self, inputs: &[T]) -> io::Result<()> {
        for input in inputs {
            let input = input.as_ref();
            match input.try_exists() {
                Ok(true) => (),
                Ok(false) => {
                    eprintln!("Warning: Input not found '{}'", input.display());
                    continue;
                }
                Err(error) if error.kind() == ErrorKind::PermissionDenied => {
                    eprintln!("Warning: Permission denied accessing '{}'", input.display());
                    continue;
                }
                Err(error) => return Err(error),
            }
            self.scan(input)?;
        }
        Ok(())
    }

    pub fn has_release(&self, id: &str) -> bool {
        self.releases.contains_key(id)
    }

    // TODO: it is very simple, naive and error-prone approach
    fn scan(&mut self, path: &Path) -> io::Result<()> {
        if path.is_dir() {
            match fs::read_dir(path) {
                Ok(entries) => {
                    for entry in entries {
                        self.scan(&entry?.path())?;
                    }
                }
                Err(error) if error.kind() == ErrorKind::PermissionDenied => {
                    eprintln!("Warning: Permission denied accessing '{}'", path.display());
                }
                Err(error) => return Err(error),
            }
        } else {
            self.process_file(path)?;
        }
        Ok(())
    }

    fn process_file(&mut self, path: &Path) -> io::Result<bool> {
        let Some(extension) = path.extension() else {
            return Ok(false);
        };
        let is_song = EXTENSIONS.iter().any(|candidate| extension == *candidate);
        if !is_song {
            return Ok(false);
        }
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(error) => match error.kind() {
                ErrorKind::PermissionDenied => {
                    eprintln!("Warning: Permission denied accessing '{}'", path.display());
                    return Ok(false);
                }
                _ => return Err(error),
            },
        };
        // TODO: better error handling
        let tagged_file = lofty::read_from(&mut file).unwrap();
        let Some(tag) = tagged_file.primary_tag() else {
            eprintln!("Could not retrieve metadata from '{}'", path.display());
            return Ok(false);
        };
        // TODO: report if tags are missing
        self.process_tags(tag);
        Ok(true)
    }

    fn process_tags(&mut self, tag: &Tag) -> Option<()> {
        // let artist_id = tag.get_string(ItemKey::MusicBrainzReleaseArtistId)?;
        let group_id = tag.get_string(ItemKey::MusicBrainzReleaseGroupId)?;
        let release_id = tag.get_string(ItemKey::MusicBrainzReleaseId)?;
        // let track_id = tag.get_string(ItemKey::MusicBrainzTrackId)?;
        // let release_title = tag.get_string(ItemKey::AlbumTitle)?;

        self.releases
            .entry(release_id.into())
            .or_insert_with(|| Release {
                group_id: group_id.to_string(),
            });

        Some(())
    }
}
