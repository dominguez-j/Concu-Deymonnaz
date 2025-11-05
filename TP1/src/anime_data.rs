use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeData {
    pub anime_id: u32,
    pub title: String,
    pub anime_type: String,
    pub source: String,
    pub genres: HashSet<String>,
}

impl AnimeData {
    pub fn new(
        anime_id: u32,
        title: String,
        anime_type: String,
        source: String,
        genres_str: &str,
    ) -> Self {
        let genres = Self::parse_genres(genres_str);
        Self {
            anime_id,
            title,
            anime_type,
            source,
            genres,
        }
    }

    fn parse_genres(genres_str: &str) -> HashSet<String> {
        genres_str
            .split(',')
            .map(|g| g.trim().to_string())
            .filter(|g| !g.is_empty())
            .collect()
    }

    pub fn get_sorted_genres(&self) -> Vec<String> {
        let mut genres: Vec<String> = self.genres.iter().cloned().collect();
        genres.sort();
        genres
    }
}

#[cfg(test)]
mod anime_data_tests {
    use crate::anime_data::AnimeData;

    #[test]
    fn test_parse_genres_multiple() {
        let genres_str = "Action, Drama, Comedy, Slice of Life";
        let anime = AnimeData::new(
            1,
            "Test".to_string(),
            "TV".to_string(),
            "Original".to_string(),
            genres_str,
        );

        assert_eq!(anime.genres.len(), 4);
        assert!(anime.genres.contains("Action"));
        assert!(anime.genres.contains("Drama"));
        assert!(anime.genres.contains("Comedy"));
        assert!(anime.genres.contains("Slice of Life"));
    }
}
