#[test]
fn test_dataset1_basic() {
    std::fs::remove_dir_all("temp_shards/").ok();

    let result = std::process::Command::new("cargo")
        .args(&["run", "tests/mini_dataset1.csv", "1", "test_output1.json"])
        .output();

    match result {
        Ok(output) => {
            assert!(output.status.success(), "El procesamiento falló");

            if std::path::Path::new("test_output1.json").exists() {
                let content = std::fs::read_to_string("test_output1.json").unwrap();
                assert!(content.contains("top_animes"));
                assert!(content.contains("top_genres"));
                assert!(content.contains("Death Note"));
                assert!(content.contains("One Piece"));
                std::fs::remove_file("test_output1.json").ok();
            }
        }
        Err(e) => panic!("Error: {}", e),
    }
}

#[test]
fn test_dataset1_json_structure() {
    std::fs::remove_dir_all("temp_shards/").ok();

    let result = std::process::Command::new("cargo")
        .args(&[
            "run",
            "tests/mini_dataset1.csv",
            "1",
            "test_structure1.json",
        ])
        .output();

    match result {
        Ok(output) => {
            assert!(output.status.success(), "El procesamiento falló");

            if std::path::Path::new("test_structure1.json").exists() {
                let content = std::fs::read_to_string("test_structure1.json").unwrap();
                let json: serde_json::Value = serde_json::from_str(&content).unwrap();

                assert!(json.get("top_animes").is_some());
                assert!(json.get("top_genres").is_some());

                if let Some(top_animes) = json.get("top_animes") {
                    assert!(top_animes.get("data").is_some());
                    assert!(top_animes.get("description").is_some());
                }

                if let Some(top_genres) = json.get("top_genres") {
                    assert!(top_genres.get("data").is_some());
                    assert!(top_genres.get("description").is_some());
                }

                std::fs::remove_file("test_structure1.json").ok();
            }
        }
        Err(e) => panic!("Error: {}", e),
    }
}

#[test]
fn test_dataset1_anime_fields() {
    std::fs::remove_dir_all("temp_shards/").ok();

    let result = std::process::Command::new("cargo")
        .args(&["run", "tests/mini_dataset1.csv", "1", "test_fields1.json"])
        .output();

    match result {
        Ok(output) => {
            assert!(output.status.success(), "El procesamiento falló");

            if std::path::Path::new("test_fields1.json").exists() {
                let content = std::fs::read_to_string("test_fields1.json").unwrap();
                let json: serde_json::Value = serde_json::from_str(&content).unwrap();

                if let Some(top_animes) = json.get("top_animes") {
                    if let Some(data) = top_animes.get("data") {
                        if let Some(animes) = data.as_array() {
                            assert!(animes.len() <= 3, "Debe haber máximo 3 animes");

                            for anime in animes {
                                assert!(anime.get("anime_id").is_some());
                                assert!(anime.get("title").is_some());
                                assert!(anime.get("average_score").is_some());
                                assert!(anime.get("genres").is_some());
                                assert!(anime.get("anime_type").is_some());
                                assert!(anime.get("source").is_some());
                            }
                        }
                    }
                }

                std::fs::remove_file("test_fields1.json").ok();
            }
        }
        Err(e) => panic!("Error: {}", e),
    }
}
