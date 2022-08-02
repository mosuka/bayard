use std::path::Path;

use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt},
};

const TMP_FILE_EXT: &str = "tmp";
const OLD_FILE_EXT: &str = "old";

pub async fn write_file(path: &Path, content: &[u8]) -> anyhow::Result<()> {
    let tmp_file_path = path.with_extension(TMP_FILE_EXT);
    let mut tmp_file = File::create(&tmp_file_path)
        .await
        .map_err(|error| anyhow::anyhow!(error))?;
    tmp_file
        .write_all(content)
        .await
        .map_err(|error| anyhow::anyhow!(error))?;

    if path.exists() {
        let old_file_path = path.with_extension(OLD_FILE_EXT);
        fs::rename(path, &old_file_path)
            .await
            .map_err(|error| anyhow::anyhow!(error))?;
    }

    fs::rename(&tmp_file_path, path)
        .await
        .map_err(|error| anyhow::anyhow!(error))?;

    Ok(())
}

pub async fn read_file(path: &Path) -> anyhow::Result<Vec<u8>> {
    let mut file = File::open(path)
        .await
        .map_err(|error| anyhow::anyhow!(error))?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)
        .await
        .map_err(|error| anyhow::anyhow!(error))?;
    Ok(content)
}

pub async fn remove_file(path: &Path) -> anyhow::Result<()> {
    let tmp_file_path = path.with_extension(TMP_FILE_EXT);
    if tmp_file_path.exists() {
        fs::remove_file(&tmp_file_path)
            .await
            .map_err(|error| anyhow::anyhow!(error))?;
    }

    let old_file_path = path.with_extension(OLD_FILE_EXT);
    if old_file_path.exists() {
        fs::remove_file(&old_file_path)
            .await
            .map_err(|error| anyhow::anyhow!(error))?;
    }

    if path.exists() {
        fs::remove_file(path)
            .await
            .map_err(|error| anyhow::anyhow!(error))?;
    }
    Ok(())
}
