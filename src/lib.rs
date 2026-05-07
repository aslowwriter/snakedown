pub mod config;
pub mod fs;
pub mod indexing;
pub mod parsing;
pub mod render;

use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::PathBuf;

use crate::config::ConfigBuilder;
use crate::fs::{crawl_notebooks, crawl_package};
pub use crate::fs::{get_module_name, get_package_modules, walk_package};
use crate::indexing::external::cache::init_cache;
use crate::indexing::external::fetch::fill_cache;
use crate::indexing::index::RawIndex;
use crate::render::formats::Renderer;
pub use crate::render::render_module;
use crate::render::{jupyter::render_notebook, render_object};

use sphinx_inv::{SphinxInventoryReader, SphinxReference, SphinxType, StdRole};

use color_eyre::Result;
use color_eyre::eyre::eyre;
use tera::Context;
use url::Url;

pub async fn render_docs(config_builder: ConfigBuilder) -> Result<Vec<PathBuf>> {
    let config = config_builder.build()?;
    let absolute_pkg_path = config.pkg_path.canonicalize()?;
    let out_api_path = if let Some(content_path) = config.renderer.content_path() {
        config
            .site_root
            .join(content_path)
            .join(&config.api_content_path)
    } else {
        config.site_root.join(&config.api_content_path)
    };

    let errored = vec![];

    let mut ctx = Context::new();
    let sd_version = env!("CARGO_PKG_VERSION_MAJOR");
    ctx.insert("SNAKEDOWN_VERSION", &sd_version);

    tracing::info!("indexing package at {}", &absolute_pkg_path.display());
    let mut index = RawIndex::new(
        absolute_pkg_path.clone(),
        config.skip_undoc,
        config.skip_private,
    )?;

    let cache_path = init_cache(None)?;

    if config.offline {
        tracing::info!("Skipping fetching external indexes because running in offline mode.")
    } else {
        fill_cache(&config.externals).await?;
    }

    for (key, ext_index) in config.externals {
        let inv_path = cache_path.join("sphinx").join(key).with_extension("inv");

        // TODO: This will be made more flexible once we add a permissive mode
        // see https://github.com/aslowwriter/snakedown/issues/38
        if !inv_path.exists() && config.offline {
            continue;
        }
        let external_base_url = Url::parse(&ext_index.url)?;

        let reference_reader = SphinxInventoryReader::from_path(&inv_path)?;
        for maybe_ref in reference_reader {
            let r = maybe_ref?;
            if !should_include_reference(&r) {
                continue;
            }
            let expanded_location = &r.expanded_location();
            index
                .external_object_store
                .insert(r.name, external_base_url.clone().join(expanded_location)?);
        }
    }

    crawl_package(
        &mut index,
        &absolute_pkg_path,
        config.skip_private,
        config.exclude.clone(),
    )?;

    if let Some(nb_path) = &config.notebook_path {
        tracing::debug!("crawling notebooks");
        crawl_notebooks(&mut index, nb_path)?;
    }

    match index.validate_references() {
        Ok(_) => Ok(()),
        Err(errors) => Err(eyre!(
            "Found {} invalid references(s):\n{:?}",
            errors.len(),
            errors
        )),
    }?;

    index.pre_process(&config.renderer, &config.api_content_path)?;

    if !config.skip_write {
        create_dir_all(&out_api_path)?;
    }

    for (key, object) in index.internal_object_store.iter() {
        let file_path = out_api_path.join(key).with_added_extension("md");
        let rendered = render_object(object, key.clone(), &config.renderer, &ctx)?;
        let rendered_trimmed = rendered.trim_start();
        if !config.skip_write {
            let mut file = File::create(file_path)?;
            file.write_all(rendered_trimmed.as_bytes())?;
        }
    }

    if let Some(notebook_path) = &config.notebook_path {
        let out_nb_path = if let Some(content_path) = config.renderer.content_path() {
            config.site_root.clone().join(content_path).join(
                config
                    .notebook_content_path
                    .clone()
                    .unwrap_or(notebook_path.clone()),
            )
        } else {
            config.site_root.clone().join(
                config
                    .notebook_content_path
                    .clone()
                    .unwrap_or(notebook_path.clone()),
            )
        };
        if !config.skip_write {
            create_dir_all(&out_nb_path)?;
        }
        for (key, cells) in index.notebook_store.iter() {
            let dir_path = out_nb_path.join(key);
            let file_path = dir_path.clone().join("index").with_added_extension("md");
            let mut rendered = render_notebook(
                dir_path
                    .file_stem()
                    .map(|p| p.display().to_string())
                    .as_deref(),
                cells,
                &config.renderer,
            )?;
            // some tools insert an extra EOL at the end of the file
            if !rendered.text.ends_with("\n") {
                rendered.text.push('\n');
            }

            if !config.skip_write {
                create_dir_all(dir_path.clone())?;
                let mut file = File::create(file_path)?;
                file.write_all(rendered.text.as_bytes())?;
                for img in rendered.images {
                    let mut img_file = File::create(dir_path.join(img.name))?;
                    img_file.write_all(&img.data)?;
                }
            }
        }
    }

    if let Some((index_file_path, index_file_content)) =
        &config.renderer.index_file(Some("API".to_string()))
        && !config.skip_write
    {
        let mut file = File::create(out_api_path.join(index_file_path))?;
        file.write_all(index_file_content.as_bytes())?;
    }

    Ok(errored)
}

fn should_include_reference(r: &SphinxReference) -> bool {
    // just include python refs and std doc refs, we'll see if we actually
    // need/want the rest
    match r.sphinx_type {
        SphinxType::Std(StdRole::Doc) | SphinxType::Python(_) => true,
        SphinxType::C(_)
        | SphinxType::Std(_)
        | SphinxType::Mathematics(_)
        | SphinxType::Cpp(_)
        | SphinxType::JavaScript(_)
        | SphinxType::ReStructuredText(_)
        | SphinxType::Cmake(_)
        | SphinxType::Sip(_)
        | SphinxType::Http(_) => false,
    }
}

#[cfg(test)]
mod test {

    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    use crate::config::ConfigBuilder;
    use crate::render::SSG;
    use crate::render_docs;

    use pretty_assertions::assert_eq;
    use std::collections::HashSet;

    use color_eyre::eyre::{Result, WrapErr, bail, eyre};
    use walkdir::WalkDir;

    /// Asserts that two directory trees are identical in structure and content.
    /// Reports all differences including missing files and content mismatches.
    pub fn assert_dir_trees_equal<P: AsRef<Path>>(expected: P, actual: P) {
        match compare_dirs(expected.as_ref(), actual.as_ref()) {
            Ok(_) => (),
            Err(e) => panic!("Directory trees differ:\n{e}"),
        }
    }

    #[allow(clippy::unwrap_used)]
    fn compare_dirs(expected: &Path, actual: &Path) -> Result<()> {
        let entries_expected = collect_files(expected)?;
        let entries_actual = collect_files(actual)?;

        let mut errors = Vec::new();

        // Get all unique relative paths from both directories
        let paths_expected: HashSet<_> = entries_expected.keys().collect();
        let paths_actual: HashSet<_> = entries_actual.keys().collect();

        let only_in_expected = paths_expected.difference(&paths_actual);
        let only_in_actual = paths_actual.difference(&paths_expected);
        let mut in_both: Vec<_> = paths_expected.intersection(&paths_actual).collect();

        in_both.sort();

        for path in only_in_expected {
            errors.push(format!("Only in {expected:?} (expected): {path:?}"));
        }

        for path in only_in_actual {
            errors.push(format!("Only in {actual:?} (actual): {path:?}"));
        }

        for path in in_both {
            let full_expected = entries_expected.get(*path).unwrap();
            let full_actual = entries_actual.get(*path).unwrap();

            let meta_expected = full_expected.metadata().wrap_err("reading metadata 1")?;
            let meta_actual = full_actual.metadata().wrap_err("reading metadata 2")?;

            match (meta_expected.is_file(), meta_actual.is_file()) {
                (true, true) => {
                    if let Err(e) = compare_files(full_expected, full_actual) {
                        errors.push(format!("Content differs at {path:?}: {e}"));
                    }
                }
                (false, false) => {} // Both are directories, skip
                _ => {
                    errors.push(format!("Type mismatch at {path:?}: file vs directory"));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(eyre!(
                "Found {} difference(s):\n{}",
                errors.len(),
                errors.join("\n")
            ))
        }
    }

    /// Recursively collects all files and directories with paths relative to `base`.
    fn collect_files(base: &Path) -> Result<std::collections::HashMap<PathBuf, PathBuf>> {
        let mut map = std::collections::HashMap::new();
        for entry in WalkDir::new(base)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|p| p.path().extension() != Some(&OsString::from("png")))
        {
            let path = entry.path();
            let rel = path.strip_prefix(base)?;
            map.insert(rel.to_path_buf(), path.to_path_buf());
        }
        Ok(map)
    }

    /// Compares the content of two files.
    fn compare_files(expected: &Path, actual: &Path) -> Result<()> {
        let buf_expected = std::fs::read(expected)?;
        let buf_actual = std::fs::read(actual)?;

        let expected_string_result = String::from_utf8(buf_expected.clone());

        match expected_string_result {
            Ok(mut expected_string) => {
                let mut actual_string = String::from_utf8(buf_actual)?;
                // to keep the tests compatible between windows, that uses \n\r
                // for line-endings instead of \n like the rest of us, we just strip
                // any \r from both reference and output

                actual_string = actual_string.replace("\r\n", "\n");
                expected_string = expected_string.replace("\r\n", "\n");

                assert_eq!(
                    expected_string,
                    actual_string,
                    "{} is different",
                    expected.display()
                );
            }
            Err(_) => {
                // If we can't do string conversion we'll just fall back to arbitrary byte equality

                assert_eq!(buf_expected, buf_actual);
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn render_test_pkg_docs_full() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()?;
        let test_pkg_dir = PathBuf::from("tests/test_pkg");
        let expected_api_result_dir = PathBuf::from("tests/rendered_full");
        let expected_notebooks_result_dir = PathBuf::from("tests/rendered_notebooks/");
        let api_content_path = PathBuf::from("api");
        let notebook_content_path = PathBuf::from("notebooks");
        let mut config_builder = ConfigBuilder::default()
            .init_with_defaults()
            .with_pkg_path(Some(test_pkg_dir))
            .with_api_content_path(Some(api_content_path.clone()))
            .with_notebook_content_path(Some(notebook_content_path.clone()))
            .with_site_root(Some(temp_dir.to_path_buf()))
            .with_skip_undoc(Some(false))
            .with_skip_private(Some(false))
            .with_notebook_path(Some(PathBuf::from("tests/test_notebooks")))
            .with_ssg(Some(crate::render::SSG::Markdown));
        config_builder.exclude_paths(vec![
            PathBuf::from("test_pkg/excluded_file.py"),
            PathBuf::from("test_pkg/excluded_module"),
            PathBuf::from("test_pkg/miss_spelled_ref.py"),
        ]);

        config_builder.add_external(
            "numpy".to_string(),
            Some("numpy".to_string()),
            "https://numpy.org/doc/stable".to_string(),
        )?;

        render_docs(config_builder).await?;

        assert_dir_trees_equal(
            expected_api_result_dir.as_path(),
            temp_dir.join(api_content_path).as_path(),
        );
        assert_dir_trees_equal(
            expected_notebooks_result_dir.as_path(),
            temp_dir.join(notebook_content_path).as_path(),
        );

        Ok(())
    }
    #[tokio::test]
    async fn render_test_pkg_docs_no_private_no_undoc() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()?;
        let test_pkg_dir = PathBuf::from("tests/test_pkg");
        let expected_api_result_dir = PathBuf::from("tests/rendered_no_private");
        let expected_notebooks_result_dir = PathBuf::from("tests/rendered_notebooks/");
        let notebook_path = PathBuf::from("tests/test_notebooks/");
        let api_content_path = PathBuf::from("api");
        let notebook_content_path = PathBuf::from("notebooks/");
        let mut config_builder = ConfigBuilder::default()
            .init_with_defaults()
            .with_pkg_path(Some(test_pkg_dir))
            .with_api_content_path(Some(api_content_path.clone()))
            .with_notebook_content_path(Some(notebook_content_path.clone()))
            .with_site_root(Some(temp_dir.to_path_buf()))
            .with_skip_undoc(Some(true))
            .with_notebook_path(Some(notebook_path))
            .with_ssg(Some(SSG::Markdown))
            .with_skip_private(Some(true));
        config_builder.exclude_paths(vec![
            PathBuf::from("test_pkg/excluded_file.py"),
            PathBuf::from("test_pkg/excluded_module"),
            PathBuf::from("test_pkg/miss_spelled_ref.py"),
        ]);

        render_docs(config_builder).await?;

        assert_dir_trees_equal(
            expected_api_result_dir.as_path(),
            temp_dir.join(api_content_path).as_path(),
        );
        assert_dir_trees_equal(
            expected_notebooks_result_dir.as_path(),
            temp_dir.join(notebook_content_path).as_path(),
        );

        Ok(())
    }

    #[tokio::test]
    async fn render_test_pkg_docs_skip_write_exit_on_err() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()?;
        let test_pkg_dir = PathBuf::from("tests/test_pkg");
        let api_content_path = PathBuf::from("api/");
        let mut config_builder = ConfigBuilder::default()
            .init_with_defaults()
            .with_pkg_path(Some(test_pkg_dir))
            .with_api_content_path(Some(api_content_path))
            .with_site_root(Some(temp_dir.to_path_buf()))
            .with_skip_undoc(Some(true))
            .with_skip_write(Some(true))
            .with_notebook_content_path(None)
            .with_notebook_path(None)
            .with_ssg(Some(SSG::Markdown))
            .with_skip_private(Some(true));
        config_builder.exclude_paths(vec![
            PathBuf::from("test_pkg/excluded_file.py"),
            PathBuf::from("test_pkg/excluded_module"),
            PathBuf::from("test_pkg/miss_spelled_ref.py"),
        ]);

        render_docs(config_builder).await?;

        Ok(())
    }
    #[tokio::test]
    async fn render_test_pkg_docs_exit_on_err() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()?;
        let test_pkg_dir = PathBuf::from("tests/test_pkg");
        let api_content_path = PathBuf::from("api/");
        let mut config_builder = ConfigBuilder::default()
            .init_with_defaults()
            .with_pkg_path(Some(test_pkg_dir))
            .with_api_content_path(Some(api_content_path))
            .with_site_root(Some(temp_dir.to_path_buf()))
            .with_skip_undoc(Some(true))
            .with_notebook_content_path(None)
            .with_notebook_path(None)
            .with_ssg(Some(SSG::Markdown))
            .with_skip_private(Some(true));
        config_builder.exclude_paths(vec![
            PathBuf::from("test_pkg/excluded_file.py"),
            PathBuf::from("test_pkg/excluded_module"),
            PathBuf::from("test_pkg/miss_spelled_ref.py"),
        ]);

        render_docs(config_builder).await?;

        Ok(())
    }

    #[tokio::test]
    async fn render_with_skip_write_does_not_write_files() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()?;
        let test_pkg_dir = PathBuf::from("tests/test_pkg");
        let notebook_path = PathBuf::from("tests/test_notebooks/");
        let api_content_path = PathBuf::from("api");
        let notebook_content_path = PathBuf::from("notebooks/");
        let mut config_builder = ConfigBuilder::default()
            .init_with_defaults()
            .with_pkg_path(Some(test_pkg_dir))
            .with_api_content_path(Some(api_content_path.clone()))
            .with_notebook_content_path(Some(notebook_content_path.clone()))
            .with_site_root(Some(temp_dir.to_path_buf()))
            .with_skip_undoc(Some(true))
            .with_notebook_path(Some(notebook_path))
            .with_ssg(Some(SSG::Markdown))
            .with_skip_write(Some(true))
            .with_skip_private(Some(true));
        config_builder.exclude_paths(vec![
            PathBuf::from("test_pkg/excluded_file.py"),
            PathBuf::from("test_pkg/excluded_module"),
            PathBuf::from("test_pkg/miss_spelled_ref.py"),
        ]);

        render_docs(config_builder).await?;

        let number_of_files = std::fs::read_dir(temp_dir.path())?.count();

        assert_eq!(number_of_files, 0);

        Ok(())
    }

    #[tokio::test]
    async fn render_test_pkg_suggests_correct_unknown_refs() -> Result<()> {
        let temp_dir = assert_fs::TempDir::new()?;
        let test_pkg_dir = PathBuf::from("tests/test_pkg");
        let api_content_path = PathBuf::from("api/");
        let mut config_builder = ConfigBuilder::default()
            .init_with_defaults()
            .with_pkg_path(Some(test_pkg_dir))
            .with_api_content_path(Some(api_content_path))
            .with_site_root(Some(temp_dir.to_path_buf()))
            .with_skip_undoc(Some(true))
            .with_ssg(Some(SSG::Markdown))
            .with_skip_private(Some(true));
        config_builder.exclude_paths(vec![
            PathBuf::from("test_pkg/excluded_file.py"),
            PathBuf::from("test_pkg/excluded_module"),
        ]);

        let result = render_docs(config_builder).await;

        // TODO: find a way to handle errors more nicely
        // see also https://github.com/aslowwriter/snakedown/issues/89
        match result {
            Ok(_) => bail!("render_docs did not exit with an error"),
            Err(e) => {
                let err_msg = format!("{:?}", e);
                assert!(err_msg.contains("test_pkg.bar.great, in object test_pkg.miss_spelled_ref.the_little_function_that_could did you mean test_pkg.bar.greet?"));
                assert!(err_msg.contains("unknown reference: nimpy.fft, in object test_pkg.miss_spelled_ref.the_little_function_that_could did you mean numpy.fft?"));
                assert!(err_msg.contains("unknown reference: asdfasdfasdf, in object test_pkg.miss_spelled_ref.the_little_function_that_could"));
            }
        }

        Ok(())
    }
}
