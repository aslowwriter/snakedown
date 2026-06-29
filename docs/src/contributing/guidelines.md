## Submitting a PR

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

When in doubt, requesting early feedback is allowed and even encouraged.

## Coverage
Coverage in Rust can be a bit fineky at times, and additionally coverage doesn't always tell the whole story, so we usually don't enforce hard limits on coverage. For example llvm-cov marking a file that only contains a `)?` when returning a `Result` that will (almost) never error is not uncommon. We don't think there's much value in enforcing that these lines be covered, so if you miss those for example, that's okay. That being said, we do like to keep our coverage high, so if you don't cover something, please have good explanation as to why!

## Organisation

For code organization, we recommend
- Grouping `impl` blocks next to their type (or trait)
- Grouping private items after the `pub` item that uses them.
  - The intent is to help people quickly find the "relevant" details, allowing them to "dig deeper" as needed.  Or put another way, the `pub` items serve as a table-of-contents.
  - The exact order is fuzzy; do what makes sense

## Troubleshooting

### My unit tests are passing but the integration test is failing, what's going on?

Make sure that you've initialized the git submodules. We use submodules to include the supported themes for the SSGs in our repo. If you've already cloned the repo, you can do so by executing the command `git submodule update --recursive --init`. If you haven't you can make sure this is done correctly by cloning with the `--recursive` option like so: `git clone --recursive https://github.com/aslowwriter/snakedown`

## General dev tips

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
