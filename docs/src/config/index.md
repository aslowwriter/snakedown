# Configuration

There are a few was that you can configure the behaviour of snakedown. The main way you'll probably want to do this
is via one of the configuration file formats detailed below to document what arguments need to be passed and save you a
lot of typing. However, most things that can be configured via a file, can also be configured (and overridden) by
the cli.

In general options can be provided and get resolved at run time according to the following precedence from highest to lowest:
- cli arguments
- `snakedown.toml`
- `[tools.snakedown]` table in a `pyproject.toml` file
- program defaults

For a complete list of all options and their explanation see [the options page](config/options.md).

## Formats
### snakedown.toml

This is a toml file with configuration options that snakedown will use. It can be located either in the current working directory or in `$HOME/.config/snakedown/snakedown.toml`

And example file filled with all the defaults can be found [here](https://github.com/aslowwriter/snakedown/blob/main/snakedown.example.toml)

The possible values are documented below.

### pyproject.toml

This one will only be found if it is in the current working directory. It has the same options and behavior as `snakedown.toml` but the tables should be prefixed with `tool.snakedown` like so:

```toml
[tool.snakedown.externals]
builtins = {name = "Python", url = "https://docs.python.org/3/"}
```
