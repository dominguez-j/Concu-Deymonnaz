use crate::anime_data::AnimeData;
use crate::anime_data_wrapper::AnimeDataWrapper;
use crate::constants::{DEFAULT_BUFFER_SIZE, FastHashMap, HASH_MAP_CAPACITY};
use csv::ReaderBuilder;
use fxhash::FxHasher;
use std::collections::hash_map::Entry;
use std::fs::File;
use std::hash::BuildHasherDefault;
use std::io::BufReader;

pub enum CsvIndex {
    AnimeId = 1,
    UserScore = 2,
    Title = 5,
    AnimeType = 6,
    Source = 7,
    Genres = 12,
}

pub struct ShardProcessor;

impl ShardProcessor {
    /*
    Procesa un archivo shard y devuelve un mapa de animes con sus datos y puntajes.
    El mapa se construye a partir de los datos del archivo shard.
    */
    pub fn process_shard_file(
        shard_path: &str,
    ) -> Result<FastHashMap<u32, AnimeDataWrapper>, String> {
        let file = File::open(shard_path)
            .map_err(|e| format!("Error al abrir shard {}: {}", shard_path, e))?;
        let reader = BufReader::with_capacity(DEFAULT_BUFFER_SIZE, file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .from_reader(reader);

        let mut local_map = FastHashMap::with_capacity_and_hasher(
            HASH_MAP_CAPACITY,
            BuildHasherDefault::<FxHasher>::default(),
        );

        let mut is_first_line = true;

        for record_res in csv_reader.records() {
            let record =
                record_res.map_err(|e| format!("Error al leer shard {}: {}", shard_path, e))?;

            if is_first_line {
                is_first_line = false;
                continue;
            }

            if record.get(CsvIndex::Genres as usize).is_none() {
                continue;
            }

            let anime_id_str = record.get(CsvIndex::AnimeId as usize).unwrap_or("");
            let user_score_str = record.get(CsvIndex::UserScore as usize).unwrap_or("");

            if let (Ok(anime_id), Ok(user_score)) =
                (anime_id_str.parse::<u32>(), user_score_str.parse::<f32>())
            {
                match local_map.entry(anime_id) {
                    Entry::Occupied(mut entry) => {
                        let wrapper: &mut AnimeDataWrapper = entry.get_mut();
                        wrapper.add_count();
                        wrapper.add_score(user_score);
                    }
                    Entry::Vacant(entry) => {
                        let title = record
                            .get(CsvIndex::Title as usize)
                            .unwrap_or("")
                            .to_owned();
                        let anime_type = record
                            .get(CsvIndex::AnimeType as usize)
                            .unwrap_or("")
                            .to_owned();
                        let source = record
                            .get(CsvIndex::Source as usize)
                            .unwrap_or("")
                            .to_owned();
                        let genres = record.get(CsvIndex::Genres as usize).unwrap_or("");
                        let anime_data =
                            AnimeData::new(anime_id, title, anime_type, source, genres);
                        entry.insert(AnimeDataWrapper::new(anime_data, user_score, 1));
                    }
                }
            }
        }
        Ok(local_map)
    }
}

#[cfg(test)]
mod shard_processor_tests {
    use crate::shard_processor::ShardProcessor;
    use std::fs::{self, File};
    use std::io::Write;

    fn create_test_shard(path: &str) {
        let mut file = File::create(path).unwrap();
        writeln!(file, "username,anime_id,my_score,user_id,gender,title,type,source,score,scored_by,rank,popularity,genre").unwrap();
        writeln!(
            file,
            "1,1,8.5,1,male,Naruto,TV,Manga,PG-13,1000,5000,100,\"Action, Adventure, Shounen\""
        )
        .unwrap();
        writeln!(
            file,
            "2,1,9.0,1,male,Naruto,TV,Manga,PG-13,1000,5000,100,\"Action, Adventure, Shounen\""
        )
        .unwrap();
        writeln!(file, "3,2,7.5,1,male,Death Note,TV,Manga,R,2000,8000,200,\"Mystery, Thriller, Psychological\"").unwrap();
    }

    #[test]
    fn test_process_shard_file() {
        let shard_path = "test_data/test_shard.csv";
        fs::create_dir_all("test_data").unwrap();
        create_test_shard(shard_path);

        let result = ShardProcessor::process_shard_file(shard_path);
        assert!(result.is_ok());

        let map = result.unwrap();
        assert!(map.len() > 0);

        fs::remove_dir_all("test_data").ok();
    }

    #[test]
    fn test_process_shard_aggregates_scores() {
        let shard_path = "test_data/test_shard2.csv";
        fs::create_dir_all("test_data").unwrap();
        create_test_shard(shard_path);

        let map = ShardProcessor::process_shard_file(shard_path).unwrap();

        if let Some(wrapper) = map.get(&1) {
            assert_eq!(wrapper.get_count(), 2);
            assert_eq!(wrapper.get_sum_score(), 17.5);
        }

        fs::remove_dir_all("test_data").ok();
    }

    #[test]
    fn test_process_shard_invalid_file() {
        let result = ShardProcessor::process_shard_file("nonexistent.csv");
        assert!(result.is_err());
    }

    #[test]
    fn test_process_shard_extracts_genres() {
        let shard_path = "test_data/test_shard3.csv";
        fs::create_dir_all("test_data").unwrap();

        let mut file = File::create(shard_path).unwrap();
        writeln!(file, "user_id,anime_id,user_score,status,episodes_watched,title,anime_type,source,rating,scored_by,members,favorites,genres").unwrap();
        writeln!(file, "1,1,8.5,completed,24,Naruto,TV,Manga,PG-13,1000,5000,100,\"Action, Adventure, Shounen\"").unwrap();

        let map = ShardProcessor::process_shard_file(shard_path).unwrap();

        if let Some(wrapper) = map.get(&1) {
            let anime = wrapper.get_anime_data();
            assert!(
                anime.genres.contains("Action"),
                "Debe contener el género Action"
            );
            assert!(
                anime.genres.contains("Adventure"),
                "Debe contener el género Adventure"
            );
            assert!(
                anime.genres.contains("Shounen"),
                "Debe contener el género Shounen"
            );
            assert_eq!(anime.genres.len(), 3, "Debe tener exactamente 3 géneros");
        } else {
            panic!("El anime con ID 1 no existe en el mapa");
        }

        fs::remove_dir_all("test_data").ok();
    }
}
