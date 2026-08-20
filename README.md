# SnakeDown 🐍🕵️‍♂️

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![codecov](https://codecov.io/gh/aslowwriter/SnakeDown/branch/main/graph/badge.svg)](https://codecov.io/gh/aslowwriter/SnakeDown)
[![crates.io](https://img.shields.io/crates/v/snakedown)](https://crates.io/crates/snakedown)
[![Docs.rs](https://docs.rs/snakedown/badge.svg)](https://docs.rs/snakedown)
[![Pixi Badge](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/prefix-dev/pixi/main/assets/badge/v0.json)](https://pixi.sh)


A Python API extractor written in Rust, with the goal of allowing you to host python documentation with your favourite static site generator.

I got frustrated with the standard tools used for documentation generation used in Python, namely that all of the ones I've worked with have at least one of the following to characteristics:

1. slow
2. poor developer experience (e.g. [Sphinx](https://www.sphinx-doc.org/en/master/) lacks a `serve` functionality)

Number 1 is particularly a concern for packages that are slow to import, or have a convoluted structure (which is the case for some packages I work with)

There are many excellent Static Site Generators like [Hugo](https://gohugo.io) or [Zola](https://www.getzola.org) that are fast and offer a much better experience when developing. Typically, documentation are static pages so they should be a perfect fit, except for one issue: automatic API reference generation. This was a big showstopper... until now!

SnakeDown is an experiment to get to a place where using Hugo or Zola for python documentation generation is a viable option.


## Design goals:

While the project might not cover all of these goals yet, these are some of the things I try to keep in mind when working on SnakeDown (in loose order of precedence):

1. **Sensible defaults over infinite configurability**. Infinite configuration comes with a lot of complexity both for user and developer, and in my opinion it is not worth it, especially at the stage SnakeDown is at now. I'd rather do 10 things right 80% of the time, then 2 things right 99.999% of the time.
2. **Correctness over speed** While speed is an explicit goal of this project, correctness should take precedent. Doing the wrong thing extremely fast is useless.
3. **Be fast (enough)** We only *parse* the source code to extract the docstrings from it, but we don't actually have to do any execution. While this might mean we can't always fully expand everything, type signatures are constants or names in the vast majority of cases, so this is a decent alternative. This by itself should make us Fast Enough :tm:
4. **Be fun** This is a personal project for the time being, and I like to have fun with those.

## Status

Currently, SnakeDown is solidly in the MVP state. As of `v0.2.0` it should be usable. However, as it has not seen much real world use yet, there may still be bugs. While I am quite happy with it so far, you use it at your own risk. That said, I always welcome bug reports and feature requests if you do decide to try it! Below is a loose planning of the features I want to work on


## Loose Roadmap:
(subject to change)

- [x] Walk file tree to find python objects
- [x] Parse them to extract documentation
- [x] Dump documentation in similar file structure to original package
- [x] Fill out the CLI
- [ ] Test output in SSGs (please submit a feature request if you want others included):
    - [x] plain markdown
    - [x] zola
    - [ ] hugo
- [x] Logging at appropriate levels
- [x] Configuration file
- [x] Do reference linking inside the docs
- [x] Do reference linking to external docs
- [x] Benchmarking & optimisation
- [x] Support for pre-executed Jupyter Notebooks
- [ ] QoL features like:
    - [ ] a file watcher
    - [x] `snakedown init` to setup `snakedown.toml` etc.
    - [x] fuzzy search to suggest close references in case of typos
    - [ ] A `view on <SCM>` button
    - [ ] possibly bibtex integration for pandoc style citations

## FAQ

### Why is my Jupyter notebook output not showing up?

The Jupyter format can actually output a surprising amount of different kinds of output, many of those we didn't have an example case for, and as a general rule we don't implement things we can't verify the use of. However, if you have something that produces output we don't support and are willing to share (a simplified version) so we can make sure it works properly, please open a feature request. The media types that our dependency support are listed [here](https://docs.rs/jupyter-protocol/1.0.0/jupyter_protocol/media/enum.MediaType.html).

### Can I use whatever theme I like?

Yes. All renderers should have a mode that will generate output without any decorations that out themes will use to style it a certain way so that you can use it in other themes. However, if you do this we cannot guarantee that it will look correct.

### What about `.rst` conversion?

As it is the native language of Sphinx, most python docstrings/documentation is written in ReStructuredText (RST). Sphinx does support markdown through the [MyST](https://myst-parser.readthedocs.io/en/latest/intro.html) extension, which converts the markdown to RST before sphinx uses it. However, most of the target SSGs either primarily or exclusively support markdown, therefore the only way that would make sense to offer RST support in snakedown would be to do the opposite and translate the RST to Markdown. This would be a huge maintenance burden for snakedown. 

Snakedown is means as an alternative Sphinx, not an alternative implementation and thus compatibility with Sphinx is a non-goal. As such, we do not plan to add RST support to snakedown. Users wanting to migrate to snakedown from something that uses RST will have to translate it to markdown themselves. There are tools that can do this, but the user will remain responsible for doing so.

### Is SnakeDown dead?

No. It is a personal project for the time being so while there may be persiods of inactivity, I intend to keep working on it until it serves my personal needs. (Poe's law dictates that I mention this is mostly a joke. It's a common question for FOSS projects that don't have the high velocity of larger projects and this is a tongue in cheek way of beating this question to the punch)

### When is the next release?

To quote one of the giants in our field BurtnSushi:

> ripgrep is a project whose contributors are volunteers. A release schedule adds undue stress to said volunteers. Therefore, releases are made on a best effort basis and no dates will ever be given.

> An exception to this can be high impact bugs. If a ripgrep release contains a significant regression, then there will generally be a strong push to get a patch release out with a fix. However, no promises are made.

(source: [ripgrep](https://github.com/BurntSushi/ripgrep/blob/94305125ef33b86151b6cd2ce2b33d641f6b6ac3/FAQ.md#release))


The same applies to SnakeDown.

### Why is it called SnakeDown?

Three main reasons:

1. I was first going to call it "SnakeOil" for extraing things from snakes of dubious value but that name was already taken
2. it's about shaking down your python code for docs
3. I find it funny

### Did people actually ask these questions?

Not yet, but I suspect they will.


## Template

This repo was initially setup using [`cargo-generate`](https://github.com/cargo-generate/cargo-generate) and [this template](https://github.com/aslowwriter/rust-template)
