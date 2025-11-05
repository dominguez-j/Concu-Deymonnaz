use std::time::Instant;

mod anime_data;
mod anime_data_wrapper;
mod constants;
mod data_saver;
mod data_transformer;
mod dataset_processor;
mod dataset_splitter;
mod shard_processor;

use constants::{SHARD_DIR, SHARD_SIZE};
use dataset_processor::DatasetProcessor;

const NUM_ARGS: usize = 4;

enum ArgsIndex {
    DatasetPath = 1,
    NumThreads = 2,
    OutputPath = 3,
}

fn verify_args(args: &Vec<String>) -> Result<(), String> {
    if args.len() != NUM_ARGS {
        return Err(
            "Usar: cargo run <ruta dataset> <cantidad de threads> <nombre de salida>".to_string(),
        );
    }

    if !std::path::Path::new(&args[ArgsIndex::DatasetPath as usize]).is_file() {
        return Err("Archivo de dataset no encontrado".to_string());
    }

    if !args[ArgsIndex::NumThreads as usize]
        .parse::<usize>()
        .is_ok()
    {
        return Err("Cantidad de threads inválida".to_string());
    }

    Ok(())
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    verify_args(&args)?;

    let dataset_path = args[ArgsIndex::DatasetPath as usize].clone();
    let num_threads = args[ArgsIndex::NumThreads as usize]
        .clone()
        .parse::<usize>()
        .unwrap();
    let output_path = args[ArgsIndex::OutputPath as usize].clone();

    let time_start = Instant::now();

    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .map_err(|_| "Error al crear el pool de threads".to_string())?;

    let mut processor = DatasetProcessor::new();
    processor.load_dataset(&dataset_path, &output_path, SHARD_SIZE, SHARD_DIR)?;

    let time_end = Instant::now();
    println!(
        "Tiempo de ejecución: {:?}",
        time_end.duration_since(time_start)
    );
    Ok(())
}
