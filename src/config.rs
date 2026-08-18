use color_eyre::{Result, eyre::OptionExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use toml;
use url::Url;

use crate::render::{
    SSG,
    formats::{Renderer, md::MdRenderer, zola::ZolaRenderer},
};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct ExternalIndex {
    pub name: Option<String>,
    pub url: String,
}

impl ExternalIndex {
    pub fn new(name: Option<String>, url: String) -> Self {
        let url = if url.ends_with("/") {
            url
        } else {
            format!("{}/", url)
        };
        Self { name, url }
    }
}

pub struct Config {
    pub site_root: PathBuf,
    pub api_content_path: PathBuf,
    pub notebook_content_path: Option<PathBuf>,
    pub pkg_path: PathBuf,
    pub skip_undoc: bool,
    pub skip_private: bool,
    pub renderer: Box<dyn Renderer>,
    pub exclude: Vec<PathBuf>,
    pub externals: HashMap<String, ExternalIndex>,
    pub notebook_path: Option<PathBuf>,
    pub skip_write: bool,
    pub offline: bool,
    pub permissive: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone)]
pub struct RenderConfig {
    zola: Option<ZolaConfig>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone)]
pub struct ZolaConfig {
    use_shortcodes: bool,
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct ConfigBuilder {
    site_root: Option<PathBuf>,
    api_content_path: Option<PathBuf>,
    notebook_content_path: Option<PathBuf>,
    pkg_path: Option<PathBuf>,
    skip_undoc: Option<bool>,
    skip_private: Option<bool>,
    ssg: Option<SSG>,
    render: Option<RenderConfig>,
    exclude: Option<Vec<PathBuf>>,
    externals: Option<HashMap<String, ExternalIndex>>,
    notebook_path: Option<PathBuf>,
    skip_write: Option<bool>,
    offline: Option<bool>,
    permissive: Option<bool>,
}

impl ConfigBuilder {
    pub fn init_with_defaults(mut self) -> Self {
        self = self
            .with_site_root(Some(PathBuf::from("docs")))
            .with_api_content_path(Some(PathBuf::from("api/")))
            .with_skip_undoc(Some(false))
            .with_skip_private(Some(false))
            .with_skip_write(Some(false))
            .with_pkg_path(Some(PathBuf::from(".")))
            .with_exclude(Some(Vec::new()))
            .with_ssg(Some(SSG::Markdown))
            .with_externals(Some(predefined_externals()))
            .with_render_config(Some(RenderConfig {
                zola: Some(ZolaConfig {
                    use_shortcodes: true,
                }),
            }));

        self
    }
    pub fn with_render_config(mut self, render_config: Option<RenderConfig>) -> Self {
        if render_config.is_some() {
            self.render = render_config;
        }
        self
    }
    pub fn with_api_content_path(mut self, api_content_path: Option<PathBuf>) -> Self {
        if api_content_path.is_some() {
            self.api_content_path = api_content_path;
        }
        self
    }
    pub fn with_skip_write(mut self, skip_write: Option<bool>) -> Self {
        if skip_write.is_some() {
            self.skip_write = skip_write;
        }
        self
    }
    pub fn with_site_root(mut self, site_root: Option<PathBuf>) -> Self {
        if site_root.is_some() {
            self.site_root = site_root;
        }
        self
    }
    pub fn with_pkg_path(mut self, pkg_path: Option<PathBuf>) -> Self {
        if pkg_path.is_some() {
            self.pkg_path = pkg_path;
        }
        self
    }
    pub fn with_skip_undoc(mut self, skip_undoc: Option<bool>) -> Self {
        if skip_undoc.is_some() {
            self.skip_undoc = skip_undoc;
        }
        self
    }
    pub fn with_skip_private(mut self, skip_private: Option<bool>) -> Self {
        if skip_private.is_some() {
            self.skip_private = skip_private;
        }
        self
    }
    pub fn exclude_paths(&mut self, excluded: Vec<PathBuf>) {
        match &mut self.exclude {
            Some(v) => v.extend(excluded),
            None => self.exclude = Some(excluded),
        }
    }
    pub fn exclude_path(&mut self, excluded: PathBuf) {
        match &mut self.exclude {
            Some(v) => v.push(excluded),
            None => self.exclude = Some(vec![excluded]),
        }
    }
    pub fn with_exclude(mut self, exclude: Option<Vec<PathBuf>>) -> Self {
        if exclude.is_some() {
            self.exclude = exclude;
        }
        self
    }
    pub fn with_notebook_content_path(mut self, notebook_content_path: Option<PathBuf>) -> Self {
        if notebook_content_path.is_some() {
            self.notebook_content_path = notebook_content_path;
        }
        self
    }
    pub fn with_notebook_path(mut self, notebook_path: Option<PathBuf>) -> Self {
        if notebook_path.is_some() {
            self.notebook_path = notebook_path;
        }
        self
    }
    pub fn add_external(&mut self, key: String, name: Option<String>, link: String) -> Result<()> {
        if self.externals.is_none() {
            self.externals = Some(HashMap::new());
        }

        let externals = self.externals.get_or_insert_default();

        let external_index = ExternalIndex::new(name, link);

        externals.insert(key, external_index);

        Ok(())
    }
    pub fn with_externals(mut self, externals: Option<HashMap<String, ExternalIndex>>) -> Self {
        if externals.is_some() {
            self.externals = externals
        }
        self
    }
    pub fn with_ssg(mut self, ssg: Option<SSG>) -> Self {
        if ssg.is_some() {
            self.ssg = ssg;
        }
        self
    }
    pub fn with_offline(mut self, offline: Option<bool>) -> Self {
        if offline.is_some() {
            self.offline = offline;
        }
        self
    }
    pub fn build(self) -> Result<Config> {
        let renderer: Box<dyn Renderer> = match self.ssg {
            Some(SSG::Markdown) | None => Box::new(MdRenderer::new()),
            Some(SSG::Zola) => Box::new(ZolaRenderer {
                rel_path_to_api: PathBuf::from("api"),
            }),
        };

        let mut external_linkings = HashMap::new();

        if let Some(external_links) = self.externals {
            for (key, external_index) in external_links {
                let _ = Url::parse(&external_index.url)?; // check if url is valid
                let external_index = ExternalIndex {
                    name: external_index.name,
                    url: external_index.url,
                };
                external_linkings.insert(key, external_index);
            }
        }

        Ok(Config {
            api_content_path: self.api_content_path.unwrap_or(PathBuf::from("api/")),
            notebook_content_path: self.notebook_content_path,
            site_root: self.site_root.unwrap_or(PathBuf::from("docs")),
            pkg_path: self.pkg_path.unwrap_or(PathBuf::from(".")),
            skip_undoc: self.skip_undoc.unwrap_or(true),
            skip_private: self.skip_private.unwrap_or(false),
            exclude: self.exclude.unwrap_or_default(),
            renderer,
            externals: external_linkings,
            notebook_path: self.notebook_path,
            skip_write: self.skip_write.unwrap_or(false),
            offline: self.offline.unwrap_or(false),
            permissive: self.permissive.unwrap_or(false),
        })
    }

    pub fn to_file(&self, path: &Path) -> Result<()> {
        let serialized = toml::to_string(&self)?;
        let mut file = File::create(path)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    pub fn merge(mut self, other: ConfigBuilder) -> Self {
        if other.site_root.is_some() {
            self.site_root = other.site_root;
        }

        if other.api_content_path.is_some() {
            self.api_content_path = other.api_content_path;
        }

        if other.pkg_path.is_some() {
            self.pkg_path = other.pkg_path;
        }

        if other.skip_undoc.is_some() {
            self.skip_undoc = other.skip_undoc;
        }

        if other.skip_write.is_some() {
            self.skip_write = other.skip_write;
        }

        if other.skip_private.is_some() {
            self.skip_private = other.skip_private;
        }

        if other.ssg.is_some() {
            self.ssg = other.ssg;
        }

        if other.notebook_path.is_some() {
            self.notebook_path = other.notebook_path;
        }
        if other.notebook_content_path.is_some() {
            self.notebook_content_path = other.notebook_content_path;
        }
        if other.skip_write.is_some() {
            self.skip_write = other.skip_write;
        }
        if other.offline.is_some() {
            self.offline = other.offline;
        }

        if let Some(v) = other.exclude {
            self.exclude_paths(v);
        }

        self
    }

    pub fn to_snakedown_toml(self, path: &Path) -> Result<()> {
        let contents = toml::to_string(&self)?;
        let mut file = File::create(path)?;
        file.write_all(contents.as_bytes())?;
        Ok(())
    }

    pub fn from_path(path: &Path) -> Result<ConfigBuilder> {
        let mut file_contents = String::new();
        let mut file = File::open(path)?;
        file.read_to_string(&mut file_contents)?;
        let config: ConfigBuilder = toml::from_str(&file_contents)?;
        Ok(config)
    }

    pub fn from_pyproject(path: &Path) -> Result<ConfigBuilder> {
        let mut file_contents = String::new();
        let mut file = File::open(path)?;
        file.read_to_string(&mut file_contents)?;
        let pyproject: PyProjectToml = toml::from_str(&file_contents)?;
        let tool = pyproject
            .tool
            .ok_or_eyre("pyproject.toml did not contain a [tool] table")?;
        let snakedown_table = tool
            .snakedown
            .ok_or_eyre("tool table in pyproject.toml did not contain a snakedown table")?;

        let config_builder = ConfigBuilder::default().merge(snakedown_table);

        Ok(config_builder)
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PyProjectToml {
    #[serde(flatten)]
    inner: pyproject_toml::PyProjectToml,
    tool: Option<Tool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Tool {
    snakedown: Option<ConfigBuilder>,
}

pub fn predefined_externals() -> HashMap<String, ExternalIndex> {
    let mut externals = HashMap::new();
    externals.insert(
        "builtins".to_string(),
        ExternalIndex::new(
            Some("Python builtins".to_string()),
            "https://docs.python.org/3/".to_string(),
        ),
    );
    externals.insert(
        "numpy".to_string(),
        ExternalIndex::new(
            Some("Numpy".to_string()),
            "https://numpy.org/doc/stable/".to_string(),
        ),
    );
    externals.insert(
        "pandas".to_string(),
        ExternalIndex::new(
            Some("Pandas".to_string()),
            "https://pandas.pydata.org/docs/".to_string(),
        ),
    );
    externals.insert(
        "scipy".to_string(),
        ExternalIndex::new(
            Some("Scipy".to_string()),
            "https://docs.scipy.org/doc/scipy".to_string(),
        ),
    );
    externals
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    use crate::render::SSG;

    use super::ConfigBuilder;
    use assert_fs::TempDir;
    use color_eyre::Result;

    #[test]
    fn test_pyproject_toml() -> Result<()> {
        let test_path = PathBuf::from("tests/test_pyproject.toml");

        let config_builder = ConfigBuilder::from_pyproject(&test_path)?;

        let expected = ConfigBuilder::default()
            .with_site_root(Some(PathBuf::from("bar")))
            .with_api_content_path(Some(PathBuf::from("foo/")))
            .with_pkg_path(Some(PathBuf::from("hydromt")))
            .with_skip_undoc(Some(true))
            .with_notebook_content_path(Some(PathBuf::from("user-guide")))
            .with_notebook_path(Some(PathBuf::from("examples")))
            .with_skip_private(Some(false))
            .with_exclude(Some(vec![]))
            .with_ssg(Some(SSG::Zola));

        assert_eq!(config_builder, expected);

        Ok(())
    }
    #[test]
    fn empty_builder_creates_valid_config() -> Result<()> {
        let config = ConfigBuilder::default().build();
        assert!(config.is_ok());
        Ok(())
    }

    #[test]
    fn config_round_trip() -> Result<()> {
        let mut builder = ConfigBuilder::default()
            .with_skip_undoc(Some(false))
            .with_skip_write(Some(true))
            .with_skip_private(Some(true));

        builder.exclude_paths(vec![PathBuf::from("asdf")]);

        let tmp_dir = TempDir::new()?;
        let path = tmp_dir.join("build_config.toml");
        builder.to_file(&path)?;
        let deserialized = ConfigBuilder::from_path(&path)?;
        assert_eq!(builder, deserialized);
        Ok(())
    }

    #[test]
    fn config_merge_other_takes_precedent() -> Result<()> {
        let mut first = ConfigBuilder::default()
            .with_pkg_path(Some(PathBuf::from(".")))
            .with_ssg(Some(SSG::Zola));

        first.exclude_path(PathBuf::from("asdf"));
        first.add_external(
            "numpy".to_string(),
            Some("Numpy".to_string()),
            "https://numpy.org/doc/stable".to_string(),
        )?;

        let second = ConfigBuilder::default()
            .with_pkg_path(Some(PathBuf::from("content")))
            .with_api_content_path(Some(PathBuf::from("bar/baz/apis")))
            .with_skip_undoc(Some(true))
            .with_skip_write(None)
            .with_exclude(Some(vec![PathBuf::from("zxcv")]));

        let third = ConfigBuilder::default()
            .with_site_root(Some(PathBuf::from("_output")))
            .with_api_content_path(Some(PathBuf::from("foo/bar/apis")))
            .with_skip_private(Some(false))
            .with_skip_write(Some(true))
            .with_pkg_path(Some(PathBuf::from("pkg")))
            .with_exclude(Some(vec![PathBuf::from("qwert")]))
            .with_ssg(Some(SSG::Zola));

        let mut expected = ConfigBuilder::default()
            .with_site_root(Some(PathBuf::from("_output")))
            .with_pkg_path(Some(PathBuf::from("pkg")))
            .with_api_content_path(Some(PathBuf::from("foo/bar/apis")))
            .with_skip_undoc(Some(true))
            .with_skip_write(Some(true))
            .with_skip_undoc(Some(true))
            .with_skip_private(Some(false))
            .with_exclude(Some(vec![
                PathBuf::from("asdf"),
                PathBuf::from("zxcv"),
                PathBuf::from("qwert"),
            ]))
            .with_skip_undoc(Some(true))
            .with_ssg(Some(SSG::Zola));

        expected.add_external(
            "numpy".to_string(),
            Some("Numpy".to_string()),
            "https://numpy.org/doc/stable".to_string(),
        )?;
        let computed = first.merge(second).merge(third);
        assert_eq!(expected, computed);

        Ok(())
    }

    #[test]
    fn can_deserialize_example_config() -> Result<()> {
        let example_config = ConfigBuilder::from_path(&PathBuf::from("snakedown.example.toml"))?;

        assert_eq!(
            example_config,
            ConfigBuilder::default().init_with_defaults()
        );

        Ok(())
    }

    #[test]
    fn can_deserialize_serialized_config() -> Result<()> {
        let config = ConfigBuilder::default().init_with_defaults();

        let tmp_dir = TempDir::new()?;
        let path = tmp_dir.join("generated_config.toml");

        config.clone().to_snakedown_toml(&path)?;

        let deserialized = ConfigBuilder::from_path(&path)?;

        assert_eq!(config, deserialized);

        Ok(())
    }
}
