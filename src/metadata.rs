pub struct Artist {
    pub id: String,
}

pub struct Release {
    pub artists: Vec<Artist>,
    pub group_id: String,
}
