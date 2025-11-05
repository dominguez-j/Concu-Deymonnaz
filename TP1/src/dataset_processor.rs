use crate::anime_data_wrapper::AnimeDataWrapper;
use crate::constants::{FastHashMap, HASH_MAP_CAPACITY};
use crate::data_transformer::DataTransformer;
use crate::dataset_splitter::DatasetSplitter;
use crate::shard_processor::ShardProcessor;
use fxhash::FxHasher;
use rayon::prelude::*;
use std::fs;
use std::hash::BuildHasherDefault;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DatasetProcessor {
    anime_data_map: FastHashMap<u32, AnimeDataWrapper>,
}

impl DatasetProcessor {
    pub fn new() -> Self {
        Self {
            anime_data_map: FastHashMap::with_capacity_and_hasher(
                HASH_MAP_CAPACITY,
                BuildHasherDefault::<FxHasher>::default(),
            ),
        }
    }

    /*
    Carga el dataset en el mapa de animes. Si no existe el directorio temporal, lo crea.
    Divide el dataset en shards si no existe el primer shard.
    Procesa los shards y los une en el mapa de animes.
    Transforma los datos y los guarda en el archivo de salida.
    */
    pub fn load_dataset(
        &mut self,
        dataset_path: &str,
        output_path: &str,
        shard_size: usize,
        shard_dir: &str,
    ) -> Result<(), String> {
        let output_dir = shard_dir;

        if !Path::new(output_dir).exists() {
            fs::create_dir_all(output_dir)
                .map_err(|e| format!("Error al crear directorio temporal: {}", e))?;
        }

        let base_name = Path::new(dataset_path)
            .file_name()
            .ok_or_else(|| "Nombre de archivo inválido".to_string())?
            .to_string_lossy()
            .into_owned();

        let first_shard = format!("{}{}-{}.csv", output_dir, base_name, 0);

        if !Path::new(&first_shard).exists() {
            let bytes_per_shard = shard_size;
            let mut splitter = DatasetSplitter::new(
                bytes_per_shard,
                dataset_path.to_string(),
                output_dir.to_string(),
            );
            splitter
                .split()
                .map_err(|e| format!("Error al dividir el dataset: {}", e))?;
        }

        let shard_files = self.get_shard_files(output_dir, dataset_path)?;

        let local_maps: Vec<FastHashMap<u32, AnimeDataWrapper>> = shard_files
            .par_iter()
            .map(|shard_path| ShardProcessor::process_shard_file(shard_path))
            .collect::<Result<Vec<_>, _>>()?;

        self.merge_local_maps(local_maps);

        let transformer = DataTransformer;
        transformer.transform_dataset(&self.anime_data_map, &output_path)?;
        Ok(())
    }

    /*
    Obtiene los archivos shards. Y los retorna en un vector.
    */
    fn get_shard_files(&self, output_dir: &str, dataset_path: &str) -> Result<Vec<String>, String> {
        let base_name = Path::new(dataset_path)
            .file_name()
            .ok_or_else(|| "Nombre de archivo inválido".to_string())?
            .to_string_lossy()
            .into_owned();

        let mut files = Vec::new();
        let mut i: u32 = 0;

        loop {
            let candidate = format!("{}{}-{}.csv", output_dir, base_name, i);
            if Path::new(&candidate).exists() {
                files.push(candidate);
                i += 1;
            } else {
                break;
            }
        }

        if files.is_empty() {
            return Err("No se generaron shards".to_string());
        }

        Ok(files)
    }

    fn merge_local_maps(&mut self, local_maps: Vec<FastHashMap<u32, AnimeDataWrapper>>) {
        if local_maps.is_empty() {
            return;
        }
        let merged = local_maps.into_par_iter().reduce(
            || {
                FastHashMap::with_capacity_and_hasher(
                    HASH_MAP_CAPACITY,
                    BuildHasherDefault::<FxHasher>::default(),
                )
            },
            |mut a, b| {
                for (anime_id, wrapper) in b {
                    if let Some(existing) = a.get_mut(&anime_id) {
                        existing.add_count_by(wrapper.get_count());
                        existing.add_score(wrapper.get_sum_score());
                    } else {
                        a.insert(anime_id, wrapper);
                    }
                }
                a
            },
        );
        self.anime_data_map = merged;
    }
}
