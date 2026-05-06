pub mod init;
mod verbosity;

use clap::Args;
use color_eyre::Result;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use snakedown::{config::ConfigBuilder, render::SSG};

pub fn resolve_runtime_config(args: CliArgs) -> Result<ConfigBuilder> {
    let mut config_builder = ConfigBuilder::default();

    let pyproject_path = PathBuf::from("pyproject.toml");

    if pyproject_path.exists() && pyproject_path.is_file() {
        let pyproject_config = ConfigBuilder::from_pyproject(&pyproject_path)?;
        config_builder = config_builder.merge(pyproject_config);
    }

    if let Some(config_file_path) = discover_config_file(args.config_file) {
        let file_config_builder = ConfigBuilder::from_path(&config_file_path)?;
        config_builder = config_builder.merge(file_config_builder);
    }

    let skip_write = match args.skip_write {
        Some(sp) => {
            if sp.skip_write {
                Some(true)
            } else if sp.no_skip_write {
                Some(false)
            } else {
                unreachable!()
            }
        }
        None => None,
    };

    let skip_private = match args.skip_private {
        Some(sp) => {
            if sp.skip_private {
                Some(true)
            } else if sp.no_skip_private {
                Some(false)
            } else {
                unreachable!()
            }
        }
        None => None,
    };

    let skip_undoc = match args.skip_undoc {
        Some(su) => {
            if su.skip_undoc {
                Some(true)
            } else if su.no_skip_undoc {
                Some(false)
            } else {
                unreachable!()
            }
        }
        None => None,
    };

    let cli_args_builder = ConfigBuilder::default()
        .with_api_content_path(args.api_content_path)
        .with_site_root(args.site_root)
        .with_pkg_path(args.pkg_path)
        .with_offline(args.offline)
        .with_skip_write(skip_write)
        .with_skip_undoc(skip_undoc)
        .with_skip_private(skip_private)
        .with_exclude(args.exclude)
        .with_notebook_content_path(args.notebooks_content_path)
        .with_notebook_path(args.notebooks_path)
        .with_ssg(args.ssg);

    config_builder = config_builder.merge(cli_args_builder);

    Ok(config_builder)
}

pub fn discover_config_file(arg_config_path: Option<PathBuf>) -> Option<PathBuf> {
    let mut candidates = vec![];

    if let Some(args_path) = arg_config_path {
        candidates.push(args_path);
    }

    candidates.push(PathBuf::from("snakedown.toml"));
    candidates.push(PathBuf::from("$HOME/.config/snakedown/snakedown.toml"));
    candidates
        .into_iter()
        .find(|candidate| candidate.exists() && candidate.is_file())
}

#[derive(Args, PartialEq, Eq, Debug)]
#[group(multiple = false)]
pub struct SkipWrite {
    /// skip writing the rendered output to disk.
    /// conflicts with --no-skip-write
    #[arg(long)]
    skip_write: bool,
    /// write the rendered output to disk. (to override skip_write set in config)
    /// conflicts with --skip-write
    #[arg(long)]
    no_skip_write: bool,
}

#[derive(Args, PartialEq, Eq, Debug)]
#[group(multiple = false)]
pub struct SkipPrivate {
    /// Skip generating pages for private objects (name starts with `_`)
    /// conflicts with --no-skip-private
    #[arg(long)]
    skip_private: bool,

    /// Generate pages for private objects (name starts with `_`)
    /// conflicts with --skip-private
    #[arg(long)]
    no_skip_private: bool,
}

#[derive(Args, PartialEq, Eq, Debug)]
#[group(multiple = false)]
pub struct SkipUndoc {
    /// Skip generating pages for undocumented objects
    /// conflicts with --skip-undoc
    #[arg(long)]
    skip_undoc: bool,

    /// Generate pages for undocumented objects
    /// conflicts with --no-skip-undoc
    #[arg(long)]
    no_skip_undoc: bool,
}

#[derive(Subcommand, Clone)]
pub enum SubCommand {
    /// Interactively generate a new config
    Init,
}

#[derive(Parser)]
#[command(version, about, long_about= None)]
pub struct CliArgs {
    #[command(subcommand)]
    pub subcommand: Option<SubCommand>,

    /// The path of the root of the package
    #[arg(long, short)]
    pub pkg_path: Option<PathBuf>,

    /// The root of the site. The output will be placed in a subfolder of the content folder in
    /// this folder. `docs` by default
    #[arg(long, short)]
    pub site_root: Option<PathBuf>,

    /// The path to where the api output should be placed relative to the site_root
    /// output will specifically be placed in `./<site_root>/<api_content_path>/`
    /// `api/` by default. If you want the output to be the site root set this to the empty string
    #[arg(long, short)]
    pub api_content_path: Option<PathBuf>,

    /// The path to where the notebook output should be placed relative to the site_root
    /// output will specifically be placed in `./<site_root>/<notebook_content_path>/`
    /// `user-guide/` by default. If you want the output to be the site root set this to the empty string
    #[arg(long)]
    pub notebooks_content_path: Option<PathBuf>,

    #[arg(long, short)]
    pub notebooks_path: Option<PathBuf>,

    /// The path to the configuration file
    #[arg(long, short)]
    pub config_file: Option<PathBuf>,

    #[command(flatten)]
    pub skip_undoc: Option<SkipUndoc>,

    #[command(flatten)]
    pub skip_private: Option<SkipPrivate>,

    #[command(flatten)]
    pub skip_write: Option<SkipWrite>,

    /// Any files that should be excluded, can be file or directories and specific multiple times but currently globs are not supported
    #[arg(short, long)]
    pub exclude: Option<Vec<PathBuf>>,

    /// Do not download external references
    #[arg(long)]
    pub offline: Option<bool>,

    /// What format to render the front matter in, (zola, hugo, plain markdown, etc.)
    #[arg(long, value_enum)]
    pub ssg: Option<SSG>,

    #[command(flatten)]
    pub verbose: Verbosity,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::verbosity::CustomLogLevel;
    use clap::Parser;
    use clap_verbosity_flag::log::Level;
    use clap_verbosity_flag::{LogLevel, VerbosityFilter};
    use color_eyre::Result;

    #[test]
    fn test_custom_log_level_interface() -> Result<()> {
        // Explicitly call each method to ensure they are covered
        assert_eq!(CustomLogLevel::default_filter(), VerbosityFilter::Error);
        assert_eq!(
            CustomLogLevel::quiet_help(),
            Some("suppress all logging output")
        );
        assert_eq!(
            CustomLogLevel::quiet_long_help(),
            Some("Suppress the logging output of the application, including errors.")
        );
        assert_eq!(
            CustomLogLevel::verbose_help(),
            Some("Increase verbosity of the logging (can be specified multiple times).")
        );
        assert_eq!(
            CustomLogLevel::verbose_long_help(),
            Some(
                "Increase the logging verbosity of the application by one level (ERROR, WARN, INFO, DEBUG, TRACE)"
            )
        );
        Ok(())
    }

    #[test]
    fn test_args_defaults() -> Result<()> {
        let args = CliArgs::parse_from(["snakedown"]);
        assert!(args.pkg_path.is_none());
        assert!(args.site_root.is_none());
        assert!(args.api_content_path.is_none());
        assert_eq!(args.skip_undoc, None);
        assert_eq!(args.skip_private, None);
        assert!(args.exclude.is_none());
        assert!(args.ssg.is_none());
        Ok(())
    }

    #[test]
    fn test_args_negative_flags() -> Result<()> {
        let args = CliArgs::parse_from(["snakedown", "--no-skip-undoc", "--no-skip-private"]);
        assert_eq!(
            args.skip_undoc,
            Some(SkipUndoc {
                skip_undoc: false,
                no_skip_undoc: true
            })
        );
        assert_eq!(
            args.skip_private,
            Some(SkipPrivate {
                skip_private: false,
                no_skip_private: true
            })
        );
        Ok(())
    }
    #[test]
    fn test_args_all_flags() -> Result<()> {
        let args = CliArgs::parse_from([
            "snakedown",
            "-p",
            "src/pkg",
            "-s",
            "dist",
            "-a",
            "section1/section2/api",
            "--skip-undoc",
            "--skip-private",
            "--exclude",
            "path/to/exclude1",
            "--exclude",
            "path/to/exclude2",
            "--ssg",
            "markdown",
            "--skip-write",
            "-v",
            "-v",
        ]);
        assert_eq!(args.pkg_path, Some(PathBuf::from("src/pkg")));
        assert_eq!(args.site_root, Some(PathBuf::from("dist")));
        assert_eq!(
            args.api_content_path,
            Some(PathBuf::from("section1/section2/api"))
        );
        assert_eq!(
            args.skip_undoc,
            Some(SkipUndoc {
                skip_undoc: true,
                no_skip_undoc: false
            })
        );
        assert_eq!(
            args.skip_write,
            Some(SkipWrite {
                skip_write: true,
                no_skip_write: false
            })
        );
        assert_eq!(
            args.skip_private,
            Some(SkipPrivate {
                skip_private: true,
                no_skip_private: false
            })
        );
        assert_eq!(
            args.exclude,
            Some(vec![
                PathBuf::from("path/to/exclude1"),
                PathBuf::from("path/to/exclude2")
            ])
        );
        // Verbosity should be INFO with -v -v (test indirectly)
        let level = args.verbose.log_level();
        assert_eq!(level, Some(Level::Info));
        assert_eq!(args.ssg, Some(SSG::Markdown));
        Ok(())
    }

    #[test]
    fn test_args_exclude_short_flag() -> Result<()> {
        let args = CliArgs::parse_from(["mybin", "-e", "excluded"]);
        assert_eq!(args.exclude, Some(vec![PathBuf::from("excluded")]));
        Ok(())
    }
}
