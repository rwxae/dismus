use clap::ValueEnum;
use musicbrainz_rs::{
    ApiEndpointError, Browse, BrowseQuery, MusicBrainzClient,
    entity::{Browsable, release_group::ReleaseGroupPrimaryType},
};
use serde::de::DeserializeOwned;
use std::sync::LazyLock;

pub static USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("CARGO_PKG_REPOSITORY"),
    " )"
);

pub static MUSICBRAINZ_CLIENT: LazyLock<MusicBrainzClient> =
    LazyLock::new(|| MusicBrainzClient::new(USER_AGENT));

pub static MUSIC_BRAINZ_VARIOUS_ARTISTS_ID: &str = "89ad4ac3-39f7-470e-963a-56509c546377";

pub trait BrowseQueryExt<T: Clone> {
    // TODO: ideally this method should not require a mutable reference
    /// Fetch all pages
    fn execute_all_with_client_async(
        &mut self,
        client: &MusicBrainzClient,
    ) -> impl Future<Output = Result<Vec<T>, Box<ApiEndpointError>>>
    where
        T: Browse + Browsable + DeserializeOwned + Sync;
}

impl<T: Clone> BrowseQueryExt<T> for BrowseQuery<T> {
    async fn execute_all_with_client_async(
        &mut self,
        client: &MusicBrainzClient,
    ) -> Result<Vec<T>, Box<ApiEndpointError>>
    where
        T: Browse + Browsable + DeserializeOwned + Sync,
    {
        let mut items = Vec::new();
        loop {
            let result = self.execute_with_client_async(client).await?;
            // I could probably check if entities.len() < LIMIT and avoid
            // one last request. But according to MusicBrainz API docs,
            // it sometimes may return less releases than the specified limit.
            // See: https://musicbrainz.org/doc/MusicBrainz_API#Paging
            if result.entities.is_empty() {
                break;
            }
            self.offset(self.offset.unwrap_or(0) + result.entities.len() as u16);
            items.extend(result.entities);
        }
        Ok(items)
    }
}

// TODO: is it really a good practice?
#[derive(Clone, ValueEnum)]
pub enum MusicBrainzReleaseType {
    Album,
    Ep,
    Single,
    Broadcast,
    Other,
}

impl From<MusicBrainzReleaseType> for ReleaseGroupPrimaryType {
    fn from(value: MusicBrainzReleaseType) -> Self {
        match value {
            MusicBrainzReleaseType::Album => ReleaseGroupPrimaryType::Album,
            MusicBrainzReleaseType::Ep => ReleaseGroupPrimaryType::Ep,
            MusicBrainzReleaseType::Single => ReleaseGroupPrimaryType::Single,
            MusicBrainzReleaseType::Broadcast => ReleaseGroupPrimaryType::Broadcast,
            MusicBrainzReleaseType::Other => ReleaseGroupPrimaryType::Other,
        }
    }
}
