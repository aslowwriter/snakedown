# Contributing to snakedown

Thanks for wanting to contribute! There are many ways to contribute and we
appreciate any level you're willing to do. This repository is just for snakedown itself
if the issue, request or contribution is to do with the theme, please report that in the
specific repository.

## Feature Requests

Need some new functionality? You can let us know by opening an
[issue][new issue]. Please include a good description of what you would like
and what you are hoping to accomplish with it. If you have considered alternatives
or current work arounds it is often also helpful to describe those.

In general if your feature request is well written and complete it vastly increaces the chances it will be picked up soon. In general we don't ask for information for no reason, so if you omit things that may slow things down.

## Bug Reports

Please let us know about what problems you run into, whether in behavior or
ergonomics of API. You can do this by opening an [issue][new issue].

When you do so please include the version you were using as well good descriptions
of what you were doing that may have triggered the error and what
you expect to happen instead (unless it is trivial for example when you are reporting a crash)

In general if your bug report is well written and complete it vastly increaces the chances it will be picked up soon. In general we don't ask for information for no reason, so if you omit things that may slow things down.


## Pull Requests

Looking for an idea? Check our [issues][issues]. If the issue looks open ended,
it is probably best to post on the issue how you are thinking of resolving the
issue so you can get feedback early in the process. We want you to be
successful and it can be discouraging to find out a lot of re-work is needed.
We encourage you to reach out and ask questions early in the process if you are uncertain.

Don't know where to start? check out the issues labeled [good first issue][good first issue] and [mentoring available][mentoring available]. In general these are beginner friendly with more detailed instructions.

Already have an idea?  It might be good to first [create an issue][new issue]
to propose it so we can make sure we are aligned and lower the risk of having
to re-work some of it and the discouragement that goes along with that.

### Process

As a heads up, we'll be running your PR through the following gauntlet:
- warnings turned to compile errors
- `cargo test`
- `rustfmt`
- `clippy`
- `rustdoc`
- `taplo` (toml formatter)
- `codecov`
- [`committed`](https://github.com/crate-ci/committed) as we use [Conventional](https://www.conventionalcommits.org) commit style. Ideally the commit message shouldn't just say what was done but also why.
- [`typos`](https://github.com/crate-ci/typos) to check spelling

In generally you can make sure these are okay by installing the `pre-commit` hooks in this repository. Not everything can be checked automatically though.

We also don't allow "TODO" comments in the code unless they also link to an issue, since TODO comments usually get forgotten and overlooked.

We request that the commit history gets cleaned up so that that commits are atomic, meaning they are complete and have a single responsibility. A complete commit should build, pass tests, update documentation and tests, and not have dead or commented out code.

PRs should tell a cohesive story, with refactor and test commits that keep the
fix or feature commits simple and clear.

We understand having a clean history requires more advanced git skills;
feel free to ask us for help! We might even suggest where it would work to be lax.

We also understand that editing some early commits may cause a lot of churn with merge conflicts which can make it not worth editing all of the history. One way to do this is to just keep one big temporary commit (or a bunch of temporary commits) while you prototype until things are the way you want them to be, soft reset all the commits (or move all changes to a new branch) and then re-commit things in atomic commits one by one. This also gives you a good opportunity to do a self review!

#### Coverage
Coverage in Rust can be a bit fineky at times, and additionally coverage doesn't always tell the whole story, so we usually don't enforce hard limits on coverage. For example llvm-cov marking a file that only contains a `)?` when returning a `Result` that will (almost) never error is not uncommon. We don't think there's much value in enforcing that these lines be covered, so if you miss those for example, that's okay. That being said, we do like to keep our coverage high, so if you don't cover something, please have good explanation as to why!

#### Organisation

For code organization, we recommend
- Grouping `impl` blocks next to their type (or trait)
- Grouping private items after the `pub` item that uses them.
  - The intent is to help people quickly find the "relevant" details, allowing them to "dig deeper" as needed.  Or put another way, the `pub` items serve as a table-of-contents.
  - The exact order is fuzzy; do what makes sense

### Benchmarking

Benchmarking/profilling to improve performance is tricky, because so many smart and complicated tricks are already done for you by compilers/operating systems/hardware etc. meaning that you have to make sure you work together with them instead of against them. Performance measuring and improving is part science, part art. To accommodate for this we have designed our benchmarks with the following considerations:

- Results of the benchmarks are intentionally not saved because there is an inherent amount of irreproducibility to them
- While the results are not saved, the _setup_ is. All benchmarks should use exactly the config used in `benchmark-config.toml` for the snakedown configuration and the `bench` profile for compilation (unless you are benchmarking different profiles). If you modify any of these you'll have to run new baselines!
- Remember that benchmarks are inherently hardware bound, so do not pay too much attention to the individual numbers, and if you are bench marking yourself, remember to always run the baselines yourself.
- There are currently two benchmark commands in the `pixi.toml` we use:
    - `bench` this runs the citereon benchmark. Use this if you want to see if your changes had an impact on performance
    - `flamegraph` Runs the benchmark config under `cargo flamegraph` which will produce a graph that can help you gain insight into which parts of the code need optimising. Knowing how to read this graph can be tricky at first. See the [flamegrpah repo](https://github.com/flamegraph-rs/flamegraph) for more information on this

### Dev tips

We use [`pixi`](https://pixi.prefix.dev/latest/) to manage installation of external tools, and as a cross platform task runner. Though originally developed for Python, it has access to `conda-forge` where many tools are already available, as well as making sure we use the correct versions through it's lockfile.

Though not necessary, as you can install the tools yourself and run the commands listed in the `pixi.toml` file yourself, we recommend accessing our workflows through `pixi`. You can install all the dependencies with the command `pixi install`. The default environment contains everything you might need and as a dev this is almost certainly what you want.

You can see which tasks you can run with the command `pixi task run`. This should give you an overview of all common workflows. Do note that if you install everything through pixi, then you will need to either activate the environment with `pixi shell` or run tasks (including things like `cargo run`) through pixi like so: `pixi run test` or `pixi run cargo run`.

In case you would like to install the tools yourself, below is a list of tools we use:

- [`pre-commit`](https://pre-commit.com) This will run lints when you try to commit so you don't fail CI tasks unnecessarily. Make sure to activate the hooks by running `pre-commit install` after you clone the repo.
- [`bacon`](https://github.com/Canop/bacon) A runner that will watch your files and run checks, tests, linting etc. when they change. Very useful while developing for fast feedback cycles.
- [`gh`](https://github.com/cli/cli) Can be used this to quickly open PRs when done working locally and make sure they aren't duplicated. Quite convenient, but not necessary.
- While our CI will test the outputs with all generators we support, and therefore you don't technically have to install them, it makes local development a lot easier. Currently we support the following formats:
    - [`zola`](https://github.com/getzola/zola)

Also keep in mind not all our rules have to be met at every single stage. It is totally allowed to iterate/prototype until you are happy with things, and then clean up after!

### Troubleshooting

#### My unit tests are passing but the integration test is failing, what's going on?

Make sure that you've initialized the git submodules. We use submodules to include the supported themes for the SSGs in our repo. If you've already cloned the repo, you can do so by executing the command `git submodule update --recursive --init`. If you haven't you can make sure this is done correctly by cloning with the `--recursive` option like so: `git clone --recursive https://github.com/aslowwriter/snakedown`

##  Publishing

For this project we have a [`release-plz`](https://release-plz.dev/docs) action setup. This gets updated automatically, and to release to all the places we distribute too all you have to do is edit and merge that release PR.

[issues]: https://github.com/aslowwriter/snakedown/issues
[new issue]: https://github.com/aslowwriter/snakedown/issues/new
[good first issue]: https://github.com/aslowwriter/snakedown/issues?q=state%3Aopen%20label%3A%22good%20first%20issue%22
[mentoring available]: https://github.com/aslowwriter/snakedown/issues?q=state%3Aopen%20label%3Amentoring-available

## Acknolwegements
- thank you to [Ed Page](https://github.com/epage). These guidelines were adapted from [this](https://github.com/epage/_rust) template which he wrote.
