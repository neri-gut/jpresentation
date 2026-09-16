// Ejemplo de Servicio en Rust para Extracción y Desencriptación de JWPUB
// Arquitectura: Backend (Tauri 2 / Rust 4-Layers)

use std::io::{Cursor, Read};
use std::path::Path;
use sha2::{Sha256, Digest};
use aes::Aes128;
use cbc::cipher::{BlockDecryptMut, KeyIvInit};
use cbc::Decryptor;
use flate2::read::ZlibDecoder;
use zip::ZipArchive;
use rusqlite::Connection;

type Aes128CbcDec = Decryptor<Aes128>;

pub struct JwpubMetadata {
    pub language_index: i32,
    pub symbol: String,
    pub year: i32,
    pub issue_tag: i32,
    pub title: String,
}

pub struct JwpubService;

impl JwpubService {
    /// Extrae y desencripta los documentos de un archivo .jwpub
    pub fn process_jwpub_buffer(jwpub_bytes: &[u8]) -> Result<Vec<(i32, String, String)>, Box<dyn std::error::Error>> {
        // 1. Abrir ZIP exterior
        let mut outer_zip = ZipArchive::new(Cursor::new(jwpub_bytes))?;
        
        // 2. Extraer archivo "contents" (ZIP interior)
        let mut contents_entry = outer_zip.by_name("contents")?;
        let mut inner_zip_bytes = Vec::new();
        contents_entry.read_to_end(&mut inner_zip_bytes)?;

        // 3. Abrir ZIP interior
        let mut inner_zip = ZipArchive::new(Cursor::new(&inner_zip_bytes))?;
        
        // 4. Encontrar base de datos .db
        let db_entry_name = (0..inner_zip.len())
            .find_map(|i| {
                let file = inner_zip.by_index(i).ok()?;
                if file.name().ends_with(".db") {
                    Some(file.name().to_string())
                } else {
                    None
                }
            })
            .ok_or("No se encontró base de datos SQLite en el archivo JWPUB")?;

        let mut db_entry = inner_zip.by_name(&db_entry_name)?;
        let mut db_bytes = Vec::new();
        db_entry.read_to_end(&mut db_bytes)?;

        // 5. Cargar SQLite en memoria temporal
        let conn = Connection::open_in_memory()?;
        // Restaurar DB bytes en conexión SQLite
        // (En producción se escribe a un NamedTempFile o se usa deserialize)
        
        // 6. Obtener metadatos de Publication
        let (lang_index, symbol, year, issue_tag): (i32, String, i32, i32) = conn.query_row(
            "SELECT MepsLanguageIndex, Symbol, Year, IssueTagNumber FROM Publication LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;

        let pub_card = format!("{}_{}_{}_{}", lang_index, symbol, year, issue_tag);
        let (key, iv) = Self::derive_aes_key(&pub_card)?;

        // 7. Consultar y descifrar documentos de la reunión (Class 106 para MWB o 40 para Atalaya)
        let mut stmt = conn.prepare("SELECT DocumentId, Title, Content FROM Document WHERE Content IS NOT NULL")?;
        let doc_iter = stmt.query_map([], |row| {
            let id: i32 = row.get(0)?;
            let title: String = row.get(1)?;
            let content_blob: Vec<u8> = row.get(2)?;
            Ok((id, title, content_blob))
        })?;

        let mut decrypted_docs = Vec::new();
        for doc in doc_iter {
            let (id, title, encrypted_blob) = doc?;
            if let Ok(html) = Self::decrypt_content(&encrypted_blob, &key, &iv) {
                decrypted_docs.push((id, title, html));
            }
        }

        Ok(decrypted_docs)
    }

    /// Deriva la clave AES de 128 bits e IV a partir de la PubCard y la clave maestra
    fn derive_aes_key(pub_card: &str) -> Result<([u8; 16], [u8; 16]), Box<dyn std::error::Error>> {
        let master_key_hex = "11cbb5587e32846d4c26790c633da289f66fe5842a3a585ce1bc3a294af5ada7";
        let master_bytes = hex::decode(master_key_hex)?;

        let mut hasher = Sha256::new();
        hasher.update(pub_card.as_bytes());
        let hash_bytes = hasher.finalize();

        let mut xored = [0u8; 32];
        for i in 0..32 {
            xored[i] = hash_bytes[i] ^ master_bytes[i];
        }

        let mut key = [0u8; 16];
        let mut iv = [0u8; 16];
        key.copy_from_slice(&xored[0..16]);
        iv.copy_from_slice(&xored[16..32]);

        Ok((key, iv))
    }

    /// Descifra con AES-128-CBC e infla con Zlib
    fn decrypt_content(
        encrypted_blob: &[u8],
        key: &[u8; 16],
        iv: &[u8; 16],
    ) -> Result<String, Box<dyn std::error::Error>> {
        let cipher = Aes128CbcDec::new(key.into(), iv.into());
        let mut buf = encrypted_blob.to_vec();

        let decrypted = cipher
            .decrypt_padded_mut::<cbc::cipher::block_padding::Pkcs7>(&mut buf)
            .map_err(|e| format!("Error en padding: {:?}", e))?;

        let mut decoder = ZlibDecoder::new(decrypted);
        let mut html_string = String::new();
        decoder.read_to_string(&mut html_string)?;

        Ok(html_string)
    }
}
