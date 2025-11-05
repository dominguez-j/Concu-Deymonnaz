use crate::anime_data::AnimeData;

#[derive(Debug, Clone)]
pub struct AnimeDataWrapper {
    anime_data: AnimeData,
    sum_score: f32,
    count: u32,
}

impl AnimeDataWrapper {
    pub fn new(anime_data: AnimeData, sum_score: f32, count: u32) -> Self {
        Self {
            anime_data,
            sum_score,
            count,
        }
    }

    pub fn get_anime_data(&self) -> &AnimeData {
        &self.anime_data
    }

    pub fn get_sum_score(&self) -> f32 {
        self.sum_score
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn add_score(&mut self, score: f32) {
        self.sum_score += score;
    }

    pub fn add_count(&mut self) {
        self.count += 1;
    }

    pub fn add_count_by(&mut self, amount: u32) {
        self.count += amount;
    }
}
