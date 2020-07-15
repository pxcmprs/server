use std::fmt;

#[derive(Debug)]
pub enum CacheStatus {
    Hit,
    Miss,
    Expired,
}

impl fmt::Display for CacheStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CacheStatus::Hit => "HIT",
                CacheStatus::Miss => "MISS",
                CacheStatus::Expired => "EXPIRED",
            }
        )
    }
}

pub struct Cache {
    pub bytes: Vec<u8>,
    pub status: CacheStatus,
}
