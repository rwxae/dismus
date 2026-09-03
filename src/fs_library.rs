use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{self, Error, ErrorKind},
    path::Path,
};

use lofty::{
    file::{EXTENSIONS, TaggedFileExt},
    tag::{ItemKey, Tag},
};

static MUSIC_BRAINZ_VARIOUS_ARTISTS_ID: &str = "89ad4ac3-39f7-470e-963a-56509c546377";

pub struct FileSystemLibrary<'a> {
    root: &'a Path,
    artists: HashSet<String>,
    release_groups: HashMap<String, HashSet<String>>,
}

impl<'a> FileSystemLibrary<'a> {
    pub fn new(root: &'a Path) -> Self {
        Self {
            root,
            artists: HashSet::new(),
            release_groups: HashMap::new(),
        }
    }

    pub fn load(mut self) -> io::Result<Self> {
        if !self.root.try_exists()? {
            return Err(Error::from(ErrorKind::NotFound));
        }
        self.scan(self.root)?;
        Ok(self)
    }

    pub fn get_artists(&self) -> impl Iterator<Item = &String> {
        self.artists.iter()
    }

    pub fn has_release_group(&self, id: &str) -> bool {
        self.release_groups.contains_key(id)
    }

    fn scan(&mut self, path: &Path) -> io::Result<()> {
        if path.is_dir() {
            match fs::read_dir(path) {
                Ok(entries) => {
                    for entry in entries {
                        self.scan(&entry?.path())?;
                    }
                }
                Err(error) => match error.kind() {
                    ErrorKind::PermissionDenied => {
                        eprintln!("Warning: Permission denied accessing '{}'", path.display());
                    }
                    _ => return Err(error),
                },
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
        let artist_id = tag.get_string(ItemKey::MusicBrainzReleaseArtistId)?;
        if artist_id != MUSIC_BRAINZ_VARIOUS_ARTISTS_ID {
            self.artists.insert(artist_id.into());
        }
        let release_group_id = tag.get_string(ItemKey::MusicBrainzReleaseGroupId)?;
        let release_id = tag.get_string(ItemKey::MusicBrainzReleaseId)?;
        // let track_id = tag.get_string(ItemKey::MusicBrainzTrackId)?;
        // let release_title = tag.get_string(ItemKey::AlbumTitle)?;

        let release_group = self
            .release_groups
            .entry(release_group_id.into())
            .or_default();
        release_group.insert(release_id.into());

        Some(())
    }
}
