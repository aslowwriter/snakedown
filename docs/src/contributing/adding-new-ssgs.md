
## Adding new static site generators

Note that while adding new static site generators to snakedown might not be necessarily complicated, it does require some work. In this page you'll find the requirements, some guideines on what to do, and some tips you may find useful if you want to include support for a new ssg in snakedown.

As with all others, the rules laid out here are more guidelines than hard rules. You can deviate from them, but you should have a good reason for doing so.

### Inclusion cirteria

There are a couple of things to consider when deciding to add support for an SSG:

- It should be a well maintained, active project
- It should have a reasonable userbase (in terms of size)
- It should accept markdown as an input format
- It should be installable though `pixi` (which for all intents and purposes mean that it should be on either `pypi` or `conda-forge`) so that we can keep our testing environment consistent.


### Necessary steps


1. The SSG is added to the pixi environment
2. The `Renderer` trait is implemented in `crate::render::formats` and covered by unit tests
3. The expected output of rendering with the test package is in a publicly available repository and added as a git submodule so it can be used for integration testing. A link to this repo should also be included in [usage/static-site-generators.md]
4. There is atleast one theme for the SSG that is compatible with the output of snakedown. This can be either an existing if available or custom made, but it must work out of the box. (you can use the output in `tests/rendered_full` and `tests/rendered_notebooks` to test this)
5. The format is added to the config and cli options as appropriate (see `src/congi.rs` and `src/cli/mod.rs` respectively )

Please include at least one screenshot of the example website in your PR.
