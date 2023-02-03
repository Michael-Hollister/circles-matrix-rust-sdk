// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[cfg(feature = "crypto-store")]
use async_trait::async_trait;
use deadpool_sqlite::CreatePoolError;
#[cfg(feature = "crypto-store")]
use deadpool_sqlite::Object as SqliteConn;
#[cfg(feature = "crypto-store")]
use matrix_sdk_crypto::{store::Result, CryptoStoreError};
#[cfg(feature = "crypto-store")]
use matrix_sdk_store_encryption::StoreCipher;
#[cfg(feature = "crypto-store")]
use rusqlite::OptionalExtension;
use thiserror::Error;
use tracing::error;

#[cfg(feature = "crypto-store")]
mod crypto_store;
#[cfg(feature = "crypto-store")]
mod utils;

#[cfg(feature = "crypto-store")]
pub use self::crypto_store::SqliteCryptoStore;
#[cfg(feature = "crypto-store")]
use self::utils::SqliteObjectExt;

/// All the errors that can occur when opening a sled store.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum OpenStoreError {
    /// An error occurred with the crypto store implementation.
    #[cfg(feature = "crypto-store")]
    #[error(transparent)]
    Crypto(#[from] CryptoStoreError),

    /// An error occurred with sqlite.
    #[error(transparent)]
    Sqlite(#[from] CreatePoolError),
}

#[cfg(feature = "crypto-store")]
async fn get_or_create_store_cipher(passphrase: &str, conn: &SqliteConn) -> Result<StoreCipher> {
    let encrypted_cipher = conn.get_kv("cipher").await?;

    let cipher = if let Some(encrypted) = encrypted_cipher {
        StoreCipher::import(passphrase, &encrypted)
            .map_err(|_| CryptoStoreError::UnpicklingError)?
    } else {
        let cipher = StoreCipher::new().map_err(CryptoStoreError::backend)?;
        #[cfg(not(test))]
        let export = cipher.export(passphrase);
        #[cfg(test)]
        let export = cipher._insecure_export_fast_for_testing(passphrase);
        conn.set_kv("cipher", export.map_err(CryptoStoreError::backend)?).await?;
        cipher
    };

    Ok(cipher)
}

#[cfg(feature = "crypto-store")]
trait SqliteConnectionExt {
    fn set_kv(&self, key: &str, value: &[u8]) -> rusqlite::Result<()>;
}

#[cfg(feature = "crypto-store")]
impl SqliteConnectionExt for rusqlite::Connection {
    fn set_kv(&self, key: &str, value: &[u8]) -> rusqlite::Result<()> {
        self.execute(
            "INSERT INTO kv VALUES (?1, ?2) ON CONFLICT (key) DO UPDATE SET value = ?2",
            (key, value),
        )?;
        Ok(())
    }
}

#[cfg(feature = "crypto-store")]
#[async_trait]
trait SqliteObjectStoreExt: SqliteObjectExt {
    async fn get_kv(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let key = key.to_owned();
        self.query_row("SELECT value FROM kv WHERE key = ?", (key,), |row| row.get(0))
            .await
            .optional()
            .map_err(CryptoStoreError::backend)
    }

    async fn set_kv(&self, key: &str, value: Vec<u8>) -> Result<()>;
}

#[cfg(feature = "crypto-store")]
#[async_trait]
impl SqliteObjectStoreExt for deadpool_sqlite::Object {
    async fn set_kv(&self, key: &str, value: Vec<u8>) -> Result<()> {
        let key = key.to_owned();
        self.interact(move |conn| conn.set_kv(&key, &value))
            .await
            .unwrap()
            .map_err(CryptoStoreError::backend)?;

        Ok(())
    }
}

#[cfg(test)]
#[ctor::ctor]
fn init_logging() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().with_test_writer())
        .init();
}