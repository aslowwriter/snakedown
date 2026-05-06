# Options


## pkg_path

The relative path from the current working directory to where the python package you want to document is located. This is the folder that snakedown will try to crawl and extract all documentation from.

Default value: `.`

## skip_undoc

Whether to skip undocumented objects (meaning ones that don't have a docstring). If you set this to true, no page will be generated for these objects. If it is set to false, an empty page will be generated with just the signature.

Default Value: `true`

## api_content_path
The relative path from the site root (see [site_root](#siteroot)) to where the api docs are located. This is tracked separately because this needs to be reflected in the generated links (they need to be relative to the site root, not the current working directory for example).

The full place where the docs will get placed is determined by joining `api_content_path` to `site_root` along with whatever intermediate path is required for the SSG (e.g. for zola, this is `content`). Meaning that by default the docs will be placed in `./docs/api/` for markdown and `./docs/content/api/` for zola.

Default value: `api`

## notebook_path

A path from the current working directory to a folder where notebooks are located that you want to be included.

Default value: `examples`

## notebook_content_path


The relative path from the site root (see [site_root](#siteroot)) to where the output of the jupyter notebooks should be located, similar to (api_content_path)[#apicontentpath]

Default value: `user-guide`

## site_root

The relative path from the current working directory where root of the docs site is located. For example when using a
Zola site, this should be the path to the folder containing your `contents` folder and `config.toml` file.

Default value: `docs`

## skip_private
Whether to skip private objects (meaning ones whose name starts with an `_` e.g. `_foo`) If you set this to true, no page will be generated for these objects. If it is set to false, an empty page will be generated with just the signature.

Default Value: `true`

## ssg

Which static site the output should be compatible with.

Default value:`Markdown`

Possible values:
- `Markdown`
- `Zola`

## exclude

A list of paths that should be explicitly not documented by snakedown. Paths in this list will be skipped regardless of the values of [skip_undoc](#skipundoc) and [skip_private](#skipprivate) and can be either relative or absolute.

## externals
Similar to Sphinx, snakedown can parse references to external documentation by parsing a file called `objects.inv` which sphinx produces. External references mentioned in this table will be retriefec, cached and parsed so that you can refer to them in your docstrings the same way you can to internal objects. The key (in the example below that would be `builtins`) is not used for anything other than defining the table. The url should point to the location on the internet where the `objects.inv` file is located.

Default value:

```toml
builtins = {name = "Python", url = "https://docs.python.org/3/"}
```

## render

Not all though some renderers take parameters to modify their behavior. You can set those parameters in this table like so:

```toml
[render.zola]
use_shortcodes = true
```

The `markdown` renderer currently does not have any options.
