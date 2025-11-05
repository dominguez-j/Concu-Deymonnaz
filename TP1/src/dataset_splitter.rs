use crate::constants::LARGE_BUFFER_SIZE;
use std::fs::{File, create_dir_all};
use std::io::{BufRead, BufReader, BufWriter, Result, Write};
use std::path::Path;

pub struct DatasetSplitter {
    bytes_per_shard: usize,
    count_of_bytes: usize,
    count_of_shards: usize,
    input: String,
    output: String,
    shard_writer: Option<BufWriter<File>>,
}

impl DatasetSplitter {
    pub fn new(bytes_per_shard: usize, input: String, output: String) -> Self {
        let normalized_output = if output.ends_with('/') {
            output
        } else {
            format!("{}/", output)
        };
        Self {
            bytes_per_shard,
            count_of_bytes: 0,
            count_of_shards: 0,
            input,
            output: normalized_output,
            shard_writer: None,
        }
    }

    /*
    Actualiza el writer creando un nuevo shard cuando es necesario.

    Condiciones para crear nuevo shard:
    - No hay writer activo (primera vez)
    - Se alcanzó el límite de bytes por shard
    */
    fn update_writer(&mut self) -> Result<()> {
        if self.shard_writer.is_none() || self.count_of_bytes >= self.bytes_per_shard {
            if let Some(mut writer) = self.shard_writer.take() {
                writer.flush()?;
            }
            let index = self.count_of_shards;
            create_dir_all(&self.output)?;
            let base = Path::new(self.input.as_str())
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "shard".to_string());
            let shard_path = format!("{}{}-{}.csv", self.output, base, index);
            let current_shard = File::create(&shard_path)?;
            let writer = BufWriter::with_capacity(LARGE_BUFFER_SIZE, current_shard);

            self.shard_writer = Some(writer);
            self.count_of_bytes = 0;
            self.count_of_shards += 1;
        }
        Ok(())
    }

    /*
    Divide el dataset en shards.
    */
    pub fn split(&mut self) -> Result<()> {
        let file = File::open(self.input.clone())?;
        let mut reader = BufReader::with_capacity(LARGE_BUFFER_SIZE, file);
        let mut buf = String::new();

        self.update_writer()?;

        while reader.read_line(&mut buf)? > 0 {
            let line = buf.trim_end_matches(&['\r', '\n'][..]);
            let line_size = line.len() + 1;
            if self.count_of_bytes == 0 && line_size >= self.bytes_per_shard {
                self.update_writer()?;
            } else if self.count_of_bytes + line_size > self.bytes_per_shard {
                self.update_writer()?;
            }
            self.count_of_bytes += line_size;
            if let Some(w) = self.shard_writer.as_mut() {
                w.write_all(line.as_bytes())?;
                w.write_all(b"\n")?;
            }
            buf.clear();
        }

        if let Some(mut writer) = self.shard_writer.take() {
            writer.flush()?;
        }

        if self.count_of_shards == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No se generaron shards",
            ));
        }

        Ok(())
    }
}
