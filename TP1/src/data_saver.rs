use crate::anime_data::AnimeData;
use crate::constants::DEFAULT_BUFFER_SIZE;
use std::fs::{File, create_dir_all};
use std::io::BufWriter;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataSaver;

impl DataSaver {
    pub fn save_results(
        &self,
        top_animes: &Vec<(&AnimeData, f32)>,
        top_genres: &Vec<(String, u32)>,
        output_path: &str,
    ) -> Result<(), String> {
        let formatted_animes: Vec<_> = top_animes
            .iter()
            .map(|(anime, avg_score)| {
                serde_json::json!({
                    "title": anime.title,
                    "anime_id": anime.anime_id,
                    "anime_type": anime.anime_type,
                    "source": anime.source,
                    "genres": anime.get_sorted_genres(),
                    "average_score": avg_score
                })
            })
            .collect();

        let formatted_genres: Vec<_> = top_genres
            .iter()
            .map(|(genre, count)| {
                serde_json::json!({
                    "genre": genre,
                    "total_views": count
                })
            })
            .collect();

        let results = serde_json::json!({
            "top_animes": {
                "description": "Top 3 animes con mejor puntuación promedio",
                "data": formatted_animes
            },
            "top_genres": {
                "description": "Top 5 géneros más vistos",
                "data": formatted_genres
            }
        });

        if let Some(parent) = Path::new(output_path).parent() {
            create_dir_all(parent).map_err(|e| format!("Error al crear directorio: {}", e))?;
        }

        let file =
            File::create(output_path).map_err(|e| format!("Error al crear el archivo: {}", e))?;
        let writer = BufWriter::with_capacity(DEFAULT_BUFFER_SIZE, file);

        serde_json::to_writer_pretty(writer, &results)
            .map_err(|e| format!("Error al escribir en el archivo: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod data_saver_tests {
    use crate::anime_data::AnimeData;
    use crate::data_saver::DataSaver;
    use std::fs;
    use std::path::Path;

    fn create_test_anime(id: u32, title: &str) -> AnimeData {
        AnimeData::new(
            id,
            title.to_string(),
            "TV".to_string(),
            "Manga".to_string(),
            "Action, Drama",
        )
    }

    #[test]
    fn test_save_results_creates_file() {
        let saver = DataSaver;
        let output_path = "test_output/results.json";

        let anime1 = create_test_anime(1, "Anime 1");
        let anime2 = create_test_anime(2, "Anime 2");

        let top_animes = vec![(&anime1, 8.5), (&anime2, 7.8)];
        let top_genres = vec![("Action".to_string(), 100), ("Drama".to_string(), 80)];

        let result = saver.save_results(&top_animes, &top_genres, output_path);
        assert!(result.is_ok());
        assert!(Path::new(output_path).exists());

        fs::remove_dir_all("test_output").ok();
    }

    #[test]
    fn test_save_results_valid_json() {
        let saver = DataSaver;
        let output_path = "test_output/results2.json";

        let anime1 = create_test_anime(1, "Anime 1");
        let top_animes = vec![(&anime1, 9.0)];
        let top_genres = vec![("Action".to_string(), 50)];

        saver
            .save_results(&top_animes, &top_genres, output_path)
            .unwrap();

        let content = fs::read_to_string(output_path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert!(json.get("top_animes").is_some());
        assert!(json.get("top_genres").is_some());

        fs::remove_dir_all("test_output").ok();
    }

    #[test]
    fn test_save_results_empty_lists() {
        let saver = DataSaver;
        let output_path = "test_output/results3.json";

        let top_animes: Vec<(&AnimeData, f32)> = vec![];
        let top_genres: Vec<(String, u32)> = vec![];

        let result = saver.save_results(&top_animes, &top_genres, output_path);
        assert!(result.is_ok());

        fs::remove_dir_all("test_output").ok();
    }

    #[test]
    fn test_save_results_creates_parent_directory() {
        let saver = DataSaver;
        let output_path = "test_output/nested/deep/results.json";

        let anime1 = create_test_anime(1, "Test");
        let top_animes = vec![(&anime1, 8.0)];
        let top_genres = vec![("Action".to_string(), 10)];

        let result = saver.save_results(&top_animes, &top_genres, output_path);
        assert!(result.is_ok());
        assert!(Path::new("test_output/nested/deep").exists());

        fs::remove_dir_all("test_output").ok();
    }
}
