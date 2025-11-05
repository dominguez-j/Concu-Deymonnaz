use crate::anime_data::AnimeData;
use crate::anime_data_wrapper::AnimeDataWrapper;
use crate::constants::{FastHashMap, GENRE_CAPACITY, TOP_ANIMES_LIMIT, TOP_GENRES_LIMIT};
use crate::data_saver::DataSaver;

use fxhash::FxHasher;
use rayon::prelude::*;
use std::hash::BuildHasherDefault;

#[derive(Debug, Clone)]
pub struct DataTransformer;

impl DataTransformer {
    fn get_top_animes(
        anime_data_map: &FastHashMap<u32, AnimeDataWrapper>,
    ) -> Vec<(&AnimeData, f32)> {
        let mut animes_with_scores: Vec<(&AnimeData, f32)> = anime_data_map
            .par_iter()
            .map(|(_, wrapper)| {
                let avg_score = wrapper.get_sum_score() / wrapper.get_count() as f32;
                (wrapper.get_anime_data(), avg_score)
            })
            .collect();

        animes_with_scores.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.anime_id.cmp(&b.0.anime_id))
        });
        animes_with_scores.truncate(TOP_ANIMES_LIMIT);
        animes_with_scores
    }

    fn get_top_genres(anime_data_map: &FastHashMap<u32, AnimeDataWrapper>) -> Vec<(String, u32)> {
        let genre_counts: FastHashMap<String, u32> = anime_data_map
            .par_iter()
            .fold(
                || {
                    FastHashMap::with_capacity_and_hasher(
                        GENRE_CAPACITY,
                        BuildHasherDefault::<FxHasher>::default(),
                    )
                },
                |mut acc, (_, wrapper)| {
                    for genre in &wrapper.get_anime_data().genres {
                        *acc.entry(genre.clone()).or_insert(0) += wrapper.get_count();
                    }
                    acc
                },
            )
            .reduce(
                || {
                    FastHashMap::with_capacity_and_hasher(
                        GENRE_CAPACITY,
                        BuildHasherDefault::<FxHasher>::default(),
                    )
                },
                |mut acc, map| {
                    for (genre, count) in map {
                        *acc.entry(genre).or_insert(0) += count;
                    }
                    acc
                },
            );

        let mut genres: Vec<_> = genre_counts.into_iter().collect();
        genres.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        genres.truncate(TOP_GENRES_LIMIT);
        genres
    }

    pub fn transform_dataset(
        &self,
        anime_data_map: &FastHashMap<u32, AnimeDataWrapper>,
        output_path: &str,
    ) -> Result<(), String> {
        let (top_animes, top_genres) = rayon::join(
            || Self::get_top_animes(anime_data_map),
            || Self::get_top_genres(anime_data_map),
        );

        let saver = DataSaver;
        saver.save_results(&top_animes, &top_genres, output_path)?;
        Ok(())
    }
}
