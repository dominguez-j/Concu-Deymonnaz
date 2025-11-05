#[test]
fn test_dataset3_basic() {
    std::fs::remove_dir_all("temp_shards/").ok();

    let result = std::process::Command::new("cargo")
        .args(&["run", "tests/mini_dataset3.csv", "1", "test_output3.json"])
        .output();

    match result {
        Ok(output) => {
            assert!(output.status.success(), "El procesamiento falló");

            if std::path::Path::new("test_output3.json").exists() {
                let content = std::fs::read_to_string("test_output3.json").unwrap();
                assert!(content.contains("top_animes"));
                assert!(content.contains("top_genres"));
                assert!(content.contains("Spirited Away"));
                assert!(content.contains("My Neighbor Totoro"));
                std::fs::remove_file("test_output3.json").ok();
            }
        }
        Err(e) => panic!("Error: {}", e),
    }
}

#[test]
fn test_dataset3_expected_order() {
    std::fs::remove_dir_all("temp_shards/").ok();

    let result = std::process::Command::new("cargo")
        .args(&["run", "tests/mini_dataset3.csv", "1", "test_order3.json"])
        .output();

    match result {
        Ok(output) => {
            assert!(output.status.success(), "El procesamiento falló");

            if std::path::Path::new("test_order3.json").exists() {
                let content = std::fs::read_to_string("test_order3.json").unwrap();
                let json: serde_json::Value = serde_json::from_str(&content).unwrap();

                if let Some(top_animes) = json.get("top_animes") {
                    if let Some(data) = top_animes.get("data") {
                        if let Some(animes) = data.as_array() {
                            assert_eq!(animes.len(), 3, "Debe haber exactamente 3 animes");

                            let first_anime = &animes[0];
                            assert_eq!(first_anime.get("anime_id").unwrap().as_u64().unwrap(), 1);
                            assert_eq!(
                                first_anime.get("title").unwrap().as_str().unwrap(),
                                "Spirited Away"
                            );

                            let second_anime = &animes[1];
                            assert_eq!(second_anime.get("anime_id").unwrap().as_u64().unwrap(), 2);
                            assert_eq!(
                                second_anime.get("title").unwrap().as_str().unwrap(),
                                "My Neighbor Totoro"
                            );

                            let third_anime = &animes[2];
                            assert_eq!(third_anime.get("anime_id").unwrap().as_u64().unwrap(), 3);
                            assert_eq!(
                                third_anime.get("title").unwrap().as_str().unwrap(),
                                "Princess Mononoke"
                            );
                        }
                    }
                }

                std::fs::remove_file("test_order3.json").ok();
            }
        }
        Err(e) => panic!("Error: {}", e),
    }
}

#[test]
fn test_dataset3_genre_dominance() {
    std::fs::remove_dir_all("temp_shards/").ok();

    let result = std::process::Command::new("cargo")
        .args(&["run", "tests/mini_dataset3.csv", "1", "test_genres3.json"])
        .output();

    match result {
        Ok(output) => {
            assert!(output.status.success(), "El procesamiento falló");

            if std::path::Path::new("test_genres3.json").exists() {
                let content = std::fs::read_to_string("test_genres3.json").unwrap();
                let json: serde_json::Value = serde_json::from_str(&content).unwrap();

                if let Some(top_genres) = json.get("top_genres") {
                    if let Some(data) = top_genres.get("data") {
                        if let Some(genres) = data.as_array() {
                            assert!(genres.len() <= 5, "Debe haber máximo 5 géneros");

                            let genre_names: Vec<&str> = genres
                                .iter()
                                .map(|g| g.get("genre").unwrap().as_str().unwrap())
                                .collect();

                            assert!(
                                genre_names.contains(&"Adventure"),
                                "Debe contener el género Adventure"
                            );
                            assert!(
                                genre_names.contains(&"Drama"),
                                "Debe contener el género Drama"
                            );
                            assert!(
                                genre_names.contains(&"Fantasy"),
                                "Debe contener el género Fantasy"
                            );
                        }
                    }
                }

                std::fs::remove_file("test_genres3.json").ok();
            }
        }
        Err(e) => panic!("Error: {}", e),
    }
}

#[test]
fn test_dataset3_movie_focus() {
    std::fs::remove_dir_all("temp_shards/").ok();

    let result = std::process::Command::new("cargo")
        .args(&["run", "tests/mini_dataset3.csv", "2", "test_movies3.json"])
        .output();

    match result {
        Ok(output) => {
            assert!(output.status.success(), "El procesamiento falló");

            if std::path::Path::new("test_movies3.json").exists() {
                let content = std::fs::read_to_string("test_movies3.json").unwrap();
                let json: serde_json::Value = serde_json::from_str(&content).unwrap();

                if let Some(top_animes) = json.get("top_animes") {
                    if let Some(data) = top_animes.get("data") {
                        if let Some(animes) = data.as_array() {
                            for anime in animes {
                                assert_eq!(
                                    anime.get("anime_type").unwrap().as_str().unwrap(),
                                    "Movie"
                                );
                                assert_eq!(
                                    anime.get("source").unwrap().as_str().unwrap(),
                                    "Original"
                                );
                            }
                        }
                    }
                }

                std::fs::remove_file("test_movies3.json").ok();
            }
        }
        Err(e) => panic!("Error: {}", e),
    }
}
