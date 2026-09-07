use std::{collections::HashMap, fs::File, path::Path};

use lofty::{
    file::{EXTENSIONS, TaggedFileExt},
    tag::{ItemKey, Tag},
};

use ignore::WalkBuilder;

use crate::metadata::{Artist, Release, Song};

#[derive(Default)]
pub struct FSLibraryIndex {
    releases: Vec<Release>,
    releases_index: HashMap<String, usize>,
}

impl FSLibraryIndex {
    pub fn has_release(&self, id: &str) -> bool {
        self.releases_index.contains_key(id)
    }

    pub fn has_release_group(&self, id: &str) -> bool {
        self.releases.iter().any(|release| release.group_id == id)
    }

    pub fn add_song(&mut self, song: Song, release_id: &str, group_id: &str, artists: Vec<Artist>) {
        let release = if self.has_release(release_id) {
            &mut self.releases[self.releases_index[release_id]]
        } else {
            let release = Release {
                id: release_id.to_string(),
                group_id: group_id.to_string(),
                artists,
                songs: Vec::new(),
            };
            self.releases.push(release);
            let index = self.releases.len() - 1;
            self.releases_index.insert(release_id.to_string(), index);
            &mut self.releases[index]
        };
        release.songs.push(song);
    }
}

#[derive(Default)]
pub struct FSLibraryScanner {
    index: FSLibraryIndex,
}

impl FSLibraryScanner {
    pub fn scan<T: AsRef<Path>>(mut self, paths: &[T]) -> FSLibraryIndex {
        let walk = WalkBuilder::from_iter(paths)
            .standard_filters(false)
            .filter_entry(|entry| {
                // Ignore stdin, include all dirs
                if entry.file_type().is_some_and(|f| f.is_dir()) {
                    return true;
                }
                let path = entry.path();
                let Some(extension) = path.extension() else {
                    return false;
                };
                EXTENSIONS.iter().any(|candidate| extension == *candidate)
            })
            // TODO: should i use build_parallel instead?
            // If yes, should I still be using spawn_blocking from tokio?
            .build();

        for entry in walk {
            match entry {
                Ok(entry) => {
                    if entry.file_type().is_some_and(|f| f.is_file()) {
                        self.process_file(entry.path());
                    }
                }
                Err(error) => {
                    eprintln!("WARN: encountered an error during the scan: {}", error);
                }
            }
        }

        self.index
    }

    fn process_file(&mut self, path: &Path) -> bool {
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(error) => {
                eprintln!(
                    "WARN: couldn't open a file at '{}'. Error: {}",
                    path.display(),
                    error
                );
                return false;
            }
        };
        let tagged_file = match lofty::read_from(&mut file) {
            Ok(tagged_file) => tagged_file,
            Err(error) => {
                eprintln!(
                    "WARN: couldn't parse metadata at '{}'. Error: {}",
                    path.display(),
                    error
                );
                return false;
            }
        };
        let Some(tag) = tagged_file.primary_tag() else {
            eprintln!("WARN: no metadata found in '{}'", path.display());
            return false;
        };
        if self.process_tags(tag).is_none() {
            eprintln!(
                "WARN: no MusicBrainz metadata found in '{}'",
                path.display()
            );
            return false;
        }
        true
    }

    fn process_tags(&mut self, tag: &Tag) -> Option<()> {
        let artists = tag
            .get_strings(ItemKey::MusicBrainzReleaseArtistId)
            .map(|id| Artist { id: id.to_string() })
            .collect();
        let group_id = tag.get_string(ItemKey::MusicBrainzReleaseGroupId)?;
        let release_id = tag.get_string(ItemKey::MusicBrainzReleaseId)?;
        let track_id = tag.get_string(ItemKey::MusicBrainzTrackId)?;
        // let release_title = tag.get_string(ItemKey::AlbumTitle)?;

        let song = Song {
            id: track_id.to_string(),
        };

        self.index.add_song(song, release_id, group_id, artists);

        Some(())
    }
}
