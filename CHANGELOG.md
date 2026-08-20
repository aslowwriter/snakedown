# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

## [0.4.0](https://github.com/aslowwriter/snakedown/compare/v0.3.0...v0.4.0) - 2026-08-20

### Fixed

- remove regex dependency
- remove unused struct
- fix breaking changes introduced in nbformat 3
- update github handle
- non existent pandas link in 404 fetch test
- add linking arg for flamegraph
- abort on HTTP error when fetching cache

### Other

- Update README roadmap
- remove outdated TODO
- add unwrap contrib guideline
- refactor index validation & render logic
- *(deps)* bump JamesIves/github-pages-deploy-action
- refactor content parsing
- update pixi version and lockfile
- simplify render_reference return type
- update dependencies
- refactor render_docs
- *(deps)* bump prefix-dev/setup-pixi from 0.10.0 to 0.10.1
- *(deps)* bump tera from 1.20.1 to 2.1.0
- add contrib guideines about adding ssgs
- *(deps)* bump prefix-dev/setup-pixi from 0.9.6 to 0.10.0
- *(deps)* bump actions/checkout from 6 to 7
- *(deps)* bump codecov/codecov-action from 6 to 7
- *(deps)* bump prefix-dev/setup-pixi from 0.9.5 to 0.9.6
- move to sphinx_inv for inv parsing
- *(deps)* bump codecov/codecov-action from 5 to 6
- *(deps)* bump prefix-dev/setup-pixi from 0.9.4 to 0.9.5

## [0.3.0](https://github.com/aslowwriter/snakedown/compare/v0.2.0...v0.3.0) - 2026-02-08

### Added

- *(cli)* add offline mode
- *(cli)* add skip write flag
- add support for executed jupyter notebooks
- add windows support
- suggest possible references on unknown ones
- add initial init wizard
- add numpy and pandas as predefined externals
- introduce negative flags
- adopt pixi in favour of just

### Fixed

- add arbitrary byte eq fallback for testing
- make sure we can deserialize generated configs
- don't overwrite existing config on init
- move cli into it's own subcrate
- [**breaking**] change positional arguments to flags

### Other

- tune compiler profiles
- add benchmarks
- *(deps)* bump prefix-dev/setup-pixi from 0.9.3 to 0.9.4
- add faq about failing system test
- add feature-request template
- add bug report template
- add PR template
- update roadmap in README
- *(deps)* bump JamesIves/github-pages-deploy-action
- *(deps)* bump actions/checkout from 4 to 6

## [0.2.0](https://github.com/aslowwriter/snakedown/compare/v0.1.0...v0.2.0) - 2026-01-11

### Added

- enable expansion of external references
- add GHA to deploy book
- fill out user guide
- add an mdbook for documentation
- add ability to read config from pyproject
- first impl of internal linking
- add comment about implementation strategy
- implement ref checking for fully qualified refs
- add internal links to docstrings in test pkg
- use tera to render page templates
- add tera as dependency
- refactor indexing logic
- add general python object docs enum
- move to flat output structure
- add python builting as external default example
- implement passing links to external inventory
- implement passing options to renderers from config
- implement internal link rendering
- implement external link rendering
- allow config file passed through cli
- implement merging for config builders
- impl config builder pattern
- implement basic object.inv fetching and caching
- hand implement sphinx ref parsing
- implement sphinx inventory header parsing

### Fixed

- remove unused dependendies
- justfile zola build command
- add an index file to output if necessary
- add unknown sphinx std roles
- add contributing.md to point to docs
- taplo lint
- make render_docs async
- add content path to Renderer
- add mdbook to test
- add doc to just file ci command
- replace local zola site with git submodule
- make generated links relative to site root
- move prefix stripping from rendering to parsing
- fix order of compare_tree
- remove prefix from docs extraction to rendering
- remove sub_package_index from package index struct
- move indexing/{cache,fetch}.rs to their own mod
- ignore snakedown cache in repo
- fix just zola command
- expand sphinx shorthands
- fetch correct objects.inv
- run fetching external obj as async

### Other

- sort cargo.toml
- update zola testsite submodule
- refactor render_docs api
- add Contributing guidelines
- *(deps)* bump actions/checkout from 5 to 6
- *(deps)* bump toml_edit from 0.22.26 to 0.23.2
- add .bacon-locations to .gitignore
- *(deps)* bump actions/checkout from 4 to 5
- add badges to readme


### ✨ Features

- Add doc and coverage just commands
- Detect on disk python modules
- Index ondisk python package
- Extract python documentation from ASTs
- Add dirty test package for integration test
- Add expression rendering skeleton
- Implement tuple rendering
- Implement list rendering
- Impl operator rendreing
- Impl dict & list comprehension rendering
- Impl call rendering
- Impl slice and subscript render
- Impl attribute rendering
- Increase rendering test coverage
- Impl doc rendering
- Impl remaining expression rendering
- Add rendered versions of test_pkg for testing
- Impl rendering of docs from full fs trees
- Impl a cli
- Add option to explicitly exclude files
- Add more granular logging (#2)
- Impl documenting exported attrs in modules (#3)
- Add submodule index to rendered output (#4)
- Add just cmd to open pr if ci locally passes (#5)
- Implement more general front matter rendering
- Add cli option for selecting ssg format
- Add minimal zola test site
- Impl zola front matter rendering
- Add zola installation in ci (#8)
- Add just cmd to build zola test site
- Implement generic rendering trait (#27)
- Add release-plz config

### 📖 Documentation

- Add roadmap to README
- Add description to README

### 🛠️ Maintenance

- Put separate doc types in their own mod
- Disable patch coverage (#6)
- Add zola integration test

### 🤕 Fixes

- Move doctests to module to increase coverage
- Incorrect upper case in boolean rendering
- Add space around bin op in rendering
- Remove dbg statements and add pre-commit hook (#7)
- Check tree is clean before opening pr (#9)
- Update description of cli exclude option (#10)
- Codecov format was incorrect (#11)
- Rename to snakedown
- Deprecate commitlintrc in favour of committed (#13)
- Add custom anchor render for zola headers
- Don't create extra dir at output path
- Remove syntax_error.py from output (#24)
- Revert rendering of submodules and exports (#25)

<!-- generated by git-cliff -->
