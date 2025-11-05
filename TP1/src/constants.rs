use fxhash::FxHasher;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;

// Tipo de dato para mapas hash con un hasher rápido
pub type FastHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;

// Constantes de buffer y memoria
pub const DEFAULT_BUFFER_SIZE: usize = 1024 * 1024;
pub const LARGE_BUFFER_SIZE: usize = 8 * DEFAULT_BUFFER_SIZE;

// Constantes de procesamiento de datos
pub const HASH_MAP_CAPACITY: usize = 10000;
pub const SHARD_SIZE: usize = 50 * DEFAULT_BUFFER_SIZE;

// Constantes de transformación de datos
pub const TOP_ANIMES_LIMIT: usize = 3;
pub const TOP_GENRES_LIMIT: usize = 5;
pub const GENRE_CAPACITY: usize = 500;

// Constantes de shard
pub const SHARD_DIR: &str = "temp_shards/";
