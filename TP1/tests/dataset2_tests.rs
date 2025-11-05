#[test]
fn test_dataset2_basic() {
    std::fs::remove_dir_all("temp_shards/").ok();

    let result = std::process::Command::new("cargo")
        .args(&["run", "tests/mini_dataset2.csv", "2", "test_output2.json"])
        .output();

    match result {
        Ok(output) => {
            assert!(output.status.success(), "El procesamiento falló");

            if std::path::Path::new("test_output2.json").exists() {
                let content = std::fs::read_to_string("test_output2.json").unwrap();
                assert!(content.contains("top_animes"));
                assert!(content.contains("top_genres"));
                assert!(content.contains("Attack on Titan"));
                assert!(content.contains("Your Name"));
                std::fs::remove_file("test_output2.json").ok();
            }
        }
        Err(e) => panic!("Error: {}", e),
    }
}

#[test]
fn test_dataset2_genre_fields() {
    std::fs::remove_dir_all("temp_shards/").ok();

    let result = std::process::Command::new("cargo")
        .args(&["run", "tests/mini_dataset2.csv", "2", "test_genres2.json"])
        .output();

    match result {
        Ok(output) => {
            assert!(output.status.success(), "El procesamiento falló");

            if std::path::Path::new("test_genres2.json").exists() {
                let content = std::fs::read_to_string("test_genres2.json").unwrap();
                let json: serde_json::Value = serde_json::from_str(&content).unwrap();

                if let Some(top_genres) = json.get("top_genres") {
                    if let Some(data) = top_genres.get("data") {
                        if let Some(genres) = data.as_array() {
                            assert!(genres.len() <= 5, "Debe haber máximo 5 géneros");

                            for genre in genres {
                                assert!(genre.get("genre").is_some());
                                assert!(genre.get("total_views").is_some());
                            }
                        }
                    }
                }

                std::fs::remove_file("test_genres2.json").ok();
            }
        }
        Err(e) => panic!("Error: {}", e),
    }
}
