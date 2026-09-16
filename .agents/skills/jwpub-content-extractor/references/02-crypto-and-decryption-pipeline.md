# 🔐 02. Pipeline Criptográfico y Desencriptación de JWPUB

El contenido textual de los documentos (`Document.Content` y `Extract.Content`) en las bases de datos de los archivos `.jwpub` está cifrado para proteger la integridad del texto y ahorrar espacio.

---

## 🧮 1. Algoritmo Criptográfico Formal

El proceso consta de 4 etapas:

```text
[ SQLite Publication Table ]
            │
            ▼
    1. Generar PubCard ───> 2. SHA-256(PubCard) ───┐
                                                   ├──> 3. Operación XOR ──> 32 Bytes
    [ Clave Maestra Hex ] ─────────────────────────┘                            │
                                                                                ├──> Key (Primeros 16 Bytes)
                                                                                └──> IV  (Últimos 16 Bytes)
                                                                                          │
[ Document.Content BLOB ] ────────────────────────────────────────────────────────────────┼──> 4. AES-128-CBC Decrypt
                                                                                          │
                                                                                          ▼
                                                                                   [ Inflate Deflate ]
                                                                                          │
                                                                                          ▼
                                                                                 [ HTML Decodificado ]
```

---

## 🔑 2. Parámetros de Cifrado

### 2.1 Clave Maestra Constante
```text
Base64: MTFjYmI1NTg3ZTMyODQ2ZDRjMjY3OTBjNjMzZGEyODlmNjZmZTU4NDJhM2E1ODVjZTFiYzNhMjk0YWY1YWRhNw==
Hexadecimal: 11cbb5587e32846d4c26790c633da289f66fe5842a3a585ce1bc3a294af5ada7
```

### 2.2 Formato del PubCard
Se consulta en la tabla `Publication` de la base de datos SQLite extraída:
```sql
SELECT MepsLanguageIndex, Symbol, Year, IssueTagNumber FROM Publication LIMIT 1;
```
Formato: `"{MepsLanguageIndex}_{Symbol}_{Year}_{IssueTagNumber}"`
- Ejemplo MWB: `"0_mwb25_2025_20250900"`
- Ejemplo Atalaya (W): `"1_w25_2025_20250900"`

---

## ⚙️ 3. Implementación del Algoritmo en Rust

```rust
use sha2::{Sha256, Digest};
use aes::Aes128;
use cbc::cipher::{BlockDecryptMut, KeyIvInit};
use cbc::Decryptor;
use flate2::read::ZlibDecoder;
use std::io::Read;

type Aes128CbcDec = Decryptor<Aes128>;

pub fn derive_key_iv(pub_card: &str) -> Result<([u8; 16], [u8; 16]), Box<dyn std::error::Error>> {
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

pub fn decrypt_and_decompress(
    encrypted_blob: &[u8],
    key: &[u8; 16],
    iv: &[u8; 16],
) -> Result<String, Box<dyn std::error::Error>> {
    // 1. Descifrar con AES-128-CBC
    let cipher = Aes128CbcDec::new(key.into(), iv.into());
    let mut buf = encrypted_blob.to_vec();
    
    let decrypted = cipher
        .decrypt_padded_mut::<cbc::cipher::block_padding::Pkcs7>(&mut buf)
        .map_err(|e| format!("Error en padding PKCS7: {:?}", e))?;

    // 2. Descomprimir con Zlib / Deflate
    let mut decoder = ZlibDecoder::new(decrypted);
    let mut html_string = String::new();
    decoder.read_to_string(&mut html_string)?;

    Ok(html_string)
}
```
