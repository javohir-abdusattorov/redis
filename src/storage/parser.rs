use std::{fs::File, io::{BufReader, Read}, path::Path, sync::Arc, vec};
use anyhow::Result;
use crate::config::Config;
use super::constants::{MAGIC_NUMBER, OPCODE_EOF, OPCODE_META, OPCODE_START_DB};


pub struct RDBParser {
    reader: BufReader<File>,
}

impl RDBParser {
    pub fn new(config: Arc<Config>) -> Self {
        let path = Path::new(&config.rdb_dir).join(&config.rdb_file);
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);

        RDBParser {
            reader,
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
            return Err(anyhow::anyhow!(format!("[RDB Parser] Invalid magic number: {:?}", magic)));
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
            let mut marker = [0; 1];
            if self.reader.read_exact(&mut marker).is_err() {
                break;
            }

            match marker[0] {
                OPCODE_META => {
                    println!("[Section] OPCODE_META");
                    self.process_metadata()?;
                }
                OPCODE_START_DB => {
                    println!("");
                    println!("[Section] OPCODE_START_DB");
                    self.process_start_db()?;
                }
                0xFB => {
                    println!("[Section] RESIZE_DB");
                    self.process_resize_db()?;
                }
                0xFD | 0xFC => {
                    println!("[Section] KEY_WITH_EXPIRATION");
                    self.process_key_with_expiration(marker[0])?;
                }
                0x00 => {
                    println!("[Section] KEY_WITHOUT_EXPIRATION");
                    self.process_key_without_expiration()?;
                }
                OPCODE_EOF => {
                    println!("[Section] OPCODE_EOF");
                    break;
                }
                _ => eprintln!("Unknown / unsupported marker: 0x{:02X}", marker[0])
            }
        }

        Ok(())
    }

    fn process_metadata(&mut self) -> Result<()> {
        let first_key_byte = self.read_u8()?;
        let key_length = self.read_length_or_integer(first_key_byte)?;
        let mut key_bytes = vec![0; key_length];
        self.reader.read_exact(&mut key_bytes)?;

        let key = String::from_utf8_lossy(&key_bytes).to_string();
        let first_value_byte = self.read_u8()?;
        let value = self.read_length_or_integer(first_value_byte)?;

        if first_value_byte >> 6 == 0b11 {
            println!("[Metadata] {} = {}", key, value);
        }
        else {
            let mut value_bytes = vec![0; value];
            self.reader.read_exact(&mut value_bytes)?;

            match String::from_utf8(value_bytes.clone()) {
                Ok(value) => println!("[Metadata] {} = {}", key, value),
                Err(_) => {
                    let hex_value = value_bytes.into_iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(" ");
                    println!("[Metadata] {} = {}", key, hex_value);
                }
            }
        }

        Ok(())
    }

    fn process_start_db(&mut self) -> Result<()> {
        let db_index = self.read_u8()?;
        println!("[DB] index = {db_index}");
        Ok(())
    }

    fn process_resize_db(&mut self) -> Result<()> {
        let total_size = self.read_u8()?;
        let expires_size = self.read_u8()?;
        println!("[DB] hash-table size = {total_size}; expires size = {expires_size}");

        Ok(())
    }

    fn process_key_with_expiration(&mut self, marker: u8) -> Result<()> {
        let expire_type = if marker == 0xFD { "s" } else { "ms" };
        let expiration_ms = if expire_type == "seconds" {
            let seconds = self.read_32bit_length()?;
            Some((seconds as u64) * 1000)
        } else {
            let ms = self.read_64bit_length()?;
            Some(ms)
        };

        let _value_type = self.read_u8()?;

        let key_length = self.read_u8()? as usize;
        let mut key_bytes = vec![0; key_length];
        self.reader.read_exact(&mut key_bytes)?;
        let key_str = String::from_utf8_lossy(&key_bytes).to_string();

        let value_length = self.read_u8()? as usize;
        let mut value_byte = vec![0; value_length];
        self.reader.read_exact(&mut value_byte)?;
        let value_str = String::from_utf8_lossy(&value_byte).to_string();

        println!("[EXPIRE Entry] key: {key_str:?}; value: {value_str}; expiration: {expiration_ms:?}");

        Ok(())
    }

    fn process_key_without_expiration(&mut self) -> Result<()> {
        let first_value_byte = self.read_u8()?;
        let key_length = first_value_byte as usize; // self.read_length_or_integer(first_value_byte)?;
        let mut key_bytes = vec![0; key_length];
        self.reader.read_exact(&mut key_bytes)?;
        let key_str = String::from_utf8_lossy(&key_bytes).to_string();
        println!("key_length = {key_length:?}; first_value_byte = {first_value_byte}");
        println!("key_str = {key_str:?}");

        // let first_value_byte = self.read_u8()?;
        // let value_length = first_value_byte as usize; // self.read_length_or_integer(first_value_byte)?;
        // println!("value_length = {value_length:?}; first_value_byte = {first_value_byte}");
        // let mut value_bytes = vec![0; value_length];
        // self.reader.read_exact(&mut value_bytes)?;
        // let value_str = String::from_utf8_lossy(&value_bytes).to_string();
        let value_str = self.read_integer_or_string()?;

        println!("[NON-EXPIRE Entry] key = {key_str:?}; value = {value_str:?}");
        Ok(())
    }


    fn read_integer_or_string(&mut self) -> Result<String> {
        let first_value_byte = self.read_u8()?;

        if first_value_byte >> 6 == 0b11 {
            let value = self.read_length_or_integer(first_value_byte)?;
            return Ok(value.to_string());
        }

        let value = self.read_length_or_integer(first_value_byte)?;
        let mut value_bytes = vec![0; value];
        self.reader.read_exact(&mut value_bytes)?;

        match String::from_utf8(value_bytes.clone()) {
            Ok(value) => Ok(value.to_string()),
            Err(_) => {
                Ok(value_bytes.into_iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" "))
            }
        }
    }

    fn read_length_or_integer(&mut self, first_byte: u8) -> Result<usize> {
        match first_byte >> 6 {
            0b00 => Ok((first_byte & 0x3F) as usize),
            0b01 => self.read_14bit_length(first_byte),
            0b10 => self.read_32bit_length(),
            0b11 => self.read_encoded_integer(first_byte & 0x3F),
            _ => Err(anyhow::anyhow!("[RDB Parser] Invalid length encoding"))
        }
    }

    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0; 1];
        self.reader.read_exact(&mut buf)?;
        Ok(u8::from_le_bytes(buf))
    }

    fn read_14bit_length(&mut self, first_byte: u8) -> Result<usize> {
        let second_byte = self.read_u8()?;
        Ok((((first_byte & 0x3F) as usize) << 8) | (second_byte as usize))
    }

    fn read_16bit_length(&mut self) -> Result<usize> {
        let mut buf = [0; 2];
        self.reader.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf) as usize)
    }

    fn read_32bit_length(&mut self) -> Result<usize> {
        let mut buf = [0; 4];
        self.reader.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf) as usize)
    }

    fn read_64bit_length(&mut self) -> Result<u64> {
        let mut buf = [0; 8];
        self.reader.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    fn read_encoded_integer(&mut self, encoding_type: u8) -> Result<Vec<u8>> {
        match encoding_type {
            0 => {
                let val = self.read_u8()?;
                Ok(vec![val])
            }
            1 => {
                let val = self.read_16bit_length()?;
                Ok(vec![val as u8])
            }
            2 => {
                let val = self.read_32bit_length()?;
                Ok(vec![val as u8])
            }
            3 => {
                let compressed_length = self.read_length()?;
                let real_length = self.read_length()?;

                let mut data_bytes = vec![0; compressed_length as usize];
                self.reader.read_exact(&mut data_bytes)?;
                let res = lzf::decompress(&data_bytes, real_length as usize).unwrap();
                println!("lzf: {}", String::from_utf8_lossy(&res).to_string());

                Ok(0)
            }
            n => Err(anyhow::anyhow!("Unsupported encoding-type for integer: {n}"))
        }
    }

    fn read_length(&mut self) -> Result<u32> {
        let length;
        let enc_type = self.read_u8()?;

        match (enc_type & 0xC0) >> 6 {
            3 => {
                length = (enc_type & 0x3F) as u32;
            },
            0 => {
                length = (enc_type & 0x3F) as u32;
            },
            1 => {
                let next_byte = self.read_u8()?;
                length = (((enc_type & 0x3F) as u32) <<8) | next_byte as u32;
            },
            _ => {
                length = self.read_32bit_length()? as u32;
            }
        }

        Ok(length)
    }
}