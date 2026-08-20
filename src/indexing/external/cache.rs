use std::{
    fs::{create_dir_all, exists},
    path::PathBuf,
};

use color_eyre::Result;

pub fn get_cache_path() -> PathBuf {
    PathBuf::from(".").join(".snakedown").join("cache")
}

/// checks if cache exists at the path returned by `get_cache_path` if so that is returned
/// if not, it is created and then returned
pub fn init_cache(maybe_cache_path: Option<PathBuf>) -> Result<PathBuf> {
    let cache_path = maybe_cache_path.unwrap_or_else(get_cache_path);

    if !exists(&cache_path)? {
        create_dir_all(&cache_path)?;
    }

    if !exists(cache_path.join("sphinx"))? {
        create_dir_all(cache_path.join("sphinx"))?;
    }

    Ok(cache_path)
}

#[cfg(test)]
mod test {
    use std::{
        fs::{File, create_dir_all, exists, read_dir},
        io::{Read, Write},
    };

    use color_eyre::Result;
    use tempfile::TempDir;

    use crate::indexing::external::cache::init_cache;

    #[test]
    fn init_cache_tmp_dir() -> Result<()> {
        let tmp_dir = TempDir::new()?;
        let cache_path = tmp_dir.path().join("cache");

        init_cache(Some(cache_path.clone()))?;

        assert!(exists(&cache_path)?);
        assert!(exists(cache_path.join("sphinx"))?);

        Ok(())
    }

    #[test]
    fn init_cache_doesnt_touch_existing_dir() -> Result<()> {
        let tmp_dir = TempDir::new()?;
        let cache_path = tmp_dir.path().join("cache");
        let file_path = cache_path.join("file.txt");
        create_dir_all(&cache_path)?;
        assert_eq!(read_dir(&cache_path)?.count(), 0);
        {
            let mut f = File::create(&file_path)?;
            f.write_all("foo is bar actually".as_bytes())?;
        }
        assert!(!exists(cache_path.join("sphinx"))?);
        init_cache(Some(cache_path.clone()))?;
        assert!(exists(tmp_dir.path().join("cache"))?);
        assert!(exists(tmp_dir.path().join("cache").join("sphinx"))?);
        {
            let mut f = File::open(&file_path)?;
            let mut buf = String::new();
            f.read_to_string(&mut buf)?;
            assert_eq!(&buf, "foo is bar actually");
        }
        Ok(())
    }
}
