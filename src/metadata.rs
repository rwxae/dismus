pub struct Artist {
    pub id: String,
}

pub struct Release {
    pub id: String,
    pub artists: Vec<Artist>,
    pub group_id: String,
    pub songs: Vec<Song>,
}

pub struct Song {
    pub id: String,
}
