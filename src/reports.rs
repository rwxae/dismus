use std::collections::HashSet;

use musicbrainz_rs::entity::{release::Release, release_group::ReleaseGroupPrimaryType};

use crate::{fs_library::FSLibraryIndex, musicbrainz::MUSICBRAINZ_CLIENT};

pub struct ArtistReport<'a> {
    artist: &'a str,
    // TODO: ideally it shout be a trait.
    library: &'a FSLibraryIndex,
    // TODO: create a wrapper around it for useful methods
    releases: &'a [Release],
    skip_featured: bool,
    allowed_release_types: &'a [ReleaseGroupPrimaryType],
    include_all_from_group: bool,
    include_album_singles: bool,
    // TODO: it is a temporary state for Filter logic.
    // how to improve it?
    seen_missing_groups: HashSet<String>,
}

impl<'a> ArtistReport<'a> {
    pub fn new(
        artist: &'a str,
        library: &'a FSLibraryIndex,
        releases: &'a [Release],
        allowed_release_types: &'a [ReleaseGroupPrimaryType],
    ) -> Self {
        Self {
            artist,
            library,
            releases,
            allowed_release_types,
            skip_featured: false,
            include_all_from_group: false,
            include_album_singles: false,
            seen_missing_groups: HashSet::new(),
        }
    }

    pub fn skip_featured(&mut self, yes: bool) -> &mut Self {
        self.skip_featured = yes;
        self
    }

    pub fn include_all_from_group(&mut self, yes: bool) -> &mut Self {
        self.include_all_from_group = yes;
        self
    }

    pub fn include_album_singles(&mut self, yes: bool) -> &mut Self {
        self.include_album_singles = yes;
        self
    }

    pub fn execute(&mut self) {
        self.releases
            .iter()
            .filter(|&release| {
                if self.allowed_release_types.is_empty() {
                    return true;
                }
                release
                    .release_group
                    .as_ref()
                    .expect("Client must fetch releases with release-groups")
                    .primary_type
                    .as_ref()
                    .is_some_and(|kind| self.allowed_release_types.contains(kind))
            })
            .filter(|&release| {
                if !self.skip_featured {
                    return true;
                }
                let first_artist = release
                    .artist_credit
                    .as_ref()
                    .expect("Client must fetch releases with artist-credits")
                    .first()
                    .expect("Release must have at least one credited artist");
                first_artist.artist.id == self.artist
            })
            .filter(|&release| {
                if self.include_all_from_group {
                    return true;
                }
                let group_id = &release
                    .release_group
                    .as_ref()
                    .expect("Client must fetch releases with release-groups")
                    .id;
                if self.library.has_release_group(group_id) {
                    return self.library.has_release(&release.id);
                }
                if self.seen_missing_groups.contains(group_id) {
                    return false;
                }
                self.seen_missing_groups.insert(group_id.clone());
                true
            })
            .filter(|&release| {
                if self.include_album_singles {
                    return true;
                }
                let Some(release_group) = release.release_group.as_ref() else {
                    return true;
                };
                let is_single = release_group
                    .primary_type
                    .as_ref()
                    .is_some_and(|kind| *kind == ReleaseGroupPrimaryType::Single);
                if !is_single {
                    return true;
                }
                let maybe_recording = release
                    .media
                    .as_ref()
                    .and_then(|medias| medias.first())
                    .and_then(|media| media.tracks.as_ref())
                    .and_then(|tracks| tracks.first())
                    .and_then(|track| track.recording.as_ref());
                let Some(recording) = maybe_recording else {
                    return true;
                };
                // TODO: it is better to hide it in a separate api.
                !self.releases.iter().any(|r| {
                    if r.id == release.id {
                        return false;
                    }
                    let Some(rg) = r.release_group.as_ref() else {
                        return false;
                    };
                    if rg.id == release_group.id {
                        return false;
                    }
                    r.media.as_ref().is_some_and(|medias| {
                        medias.iter().any(|media| {
                            media.tracks.as_ref().is_some_and(|tracks| {
                                tracks.iter().any(|track| {
                                    track
                                        .recording
                                        .as_ref()
                                        .is_some_and(|rec| rec.id == recording.id)
                                })
                            })
                        })
                    })
                })
            })
            .for_each(|release| {
                if self.library.has_release(&release.id) {
                    // TODO: Perform an actual comparison
                    return;
                }
                // TODO: currently it is an opinionated report result.
                // ideally it should be a struct like Report::Missing
                // which could have a method like display() or something
                // like that.
                // This would allow to use these reports in many areas.
                // e.g. Navidrome plugin could use this data to display
                // missing releases in WEB UI.

                print!("Missing: ");
                let url = MUSICBRAINZ_CLIENT
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
            });
    }
}
