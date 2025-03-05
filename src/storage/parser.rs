use std::{fs::File,io::{BufReader, Read},path::Path,sync::{Arc, Mutex}};
use anyhow::Result;
use super::{constants::{blob_encoding, encoding_type, opcode, MAGIC_NUMBER}, db::Database};
use crate::{config::Config, operation::metadata::Metadata};


pub struct Parser {
    db: Arc<Mutex<Database>>,
    reader: BufReader<File>,
    database: u32,
    expiretime: Option<u64>,
}

impl Parser {
    pub fn new(config: Arc<Config>, db: Arc<Mutex<Database>>) -> Self {
        let path = Path::new(&config.rdb_dir).join(&config.rdb_file);
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);

        Parser {
            db,
            reader,
            database: 0,
            expiretime: None,
        }
    }

    pub fn parse(&mut self) -> Result<()> {
        self.verify_magic()?;
        self.verify_version()?;
        self.process_entries()?;

        Ok(())
    }

    fn verify_magic(&mut self) -> Result<()> {
        let mut magic = [0; 5];
        self.reader.read_exact(&mut magic)?;

        if &magic != MAGIC_NUMBER {
            return Err(anyhow::anyhow!(format!("[RDB Parser] Invalid magic number: {magic:?}")));
        }

        Ok(())
    }

    fn verify_version(&mut self) -> Result<()> {
        let mut version = [0; 4];
        self.reader.read_exact(&mut version)?;
        println!("[RDB] Version: {}", String::from_utf8_lossy(&version));

        Ok(())
    }

    fn process_entries(&mut self) -> Result<()> {
        loop {
            let operation = self.read_u8()?;
            match operation {
                opcode::META => {
                    let (_key_str, _value_str) = (
                        self.read_string()?,
                        self.read_string()?,
                    );
                }
                opcode::START_DB => {
                    self.database = self.read_length()?;
                    println!("[DB] Index = {}", self.database);
                }
                opcode::RESIZE_DB => {
                    let total_size = self.read_length()?;
                    let expires_size = self.read_length()?;
                    println!("[DB] hash-table size = {total_size}; expires size = {expires_size}");
                }
                opcode::KEY_WITH_EXPIRATION_MS => {
                    let expiretime_ms = self.read_u64()?;
                    self.expiretime = Some(expiretime_ms);
                }
                opcode::KEY_WITH_EXPIRATION_SEC => {
                    let expiretime_sec = self.read_u32()?;
                    self.expiretime = Some(expiretime_sec as u64 * 1000);
                }
                opcode::EOF => {
                    break;
                }
                _ => {
                    let key = self.read_string()?;
                    self.read_entry(key, operation)?;
                    self.expiretime = None;
                },
            }
        }

        Ok(())
    }

    fn read_entry(&mut self, key: String, value_encoding: u8) -> Result<()> {
        match value_encoding {
            encoding_type::STRING => {
                let value = self.read_string()?;
                let metadata = Metadata::try_from(self.expiretime)?;
                self.db.lock().unwrap().set(&key, value, metadata);
            },
            _ => return Err(anyhow::anyhow!("[DB Parser] Cannot parse encoding type: {value_encoding}"))
        };

        Ok(())
    }

    fn read_blob(&mut self) -> Result<Vec<u8>> {
        let (length, is_encoded) = self.read_length_with_encoding()?;

        if !is_encoded {
            self.read_exact(length as usize)
        } else {
            let int = match length {
                blob_encoding::INT8 => self.read_u8()? as i32,
                blob_encoding::INT16 => self.read_u16()? as i32,
                blob_encoding::INT32 => self.read_u32()? as i32,
                blob_encoding::LZF => {
                    let compressed_length = self.read_length()?;
                    let real_length = self.read_length()?;
                    let compressed_bytes = self.read_exact(compressed_length as usize)?;
                    let decompressed =
                        lzf::decompress(&compressed_bytes, real_length as usize).unwrap();

                    return Ok(decompressed);
                }
                _ => return Err(anyhow::anyhow!("")),
            };

            let buf = int
                .to_string()
                .as_bytes()
                .into_iter()
                .map(|c| *c)
                .collect::<Vec<u8>>();

            Ok(buf)
        }
    }

    fn read_string(&mut self) -> Result<String> {
        Ok(String::from_utf8_lossy(&self.read_blob()?).to_string())
    }

    fn read_length(&mut self) -> Result<u32> {
        Ok(self.read_length_with_encoding()?.0)
    }

    fn read_length_with_encoding(&mut self) -> Result<(u32, bool)> {
        let enc_type = self.read_u8()?;
        let mut is_encoded = false;

        let length = match (enc_type & 0xC0) >> 6 {
            3 => {
                is_encoded = true;
                (enc_type & 0x3F) as u32
            }
            0 => {
                (enc_type & 0x3F) as u32
            }
            1 => {
                let next_byte = self.read_u8()?;
                (((enc_type & 0x3F) as u32) << 8) | next_byte as u32
            }
            _ => {
                self.read_u32()? as u32
            }
        };

        Ok((length, is_encoded))
    }

    fn read_exact(&mut self, length: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0; length];
        self.reader.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0; 1];
        self.reader.read_exact(&mut buf)?;
        Ok(u8::from_le_bytes(buf))
    }

    fn read_u16(&mut self) -> Result<usize> {
        let mut buf = [0; 2];
        self.reader.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf) as usize)
    }

    fn read_u32(&mut self) -> Result<usize> {
        let mut buf = [0; 4];
        self.reader.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf) as usize)
    }

    fn read_u64(&mut self) -> Result<u64> {
        let mut buf = [0; 8];
        self.reader.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }
}
