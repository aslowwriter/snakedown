# Using supported Static Site Generators

Snakedown knows how to produce output that some static site generators can work with, and provides at least one compatible theme to make customisation easier. Bellow are the static sites we can work with at the moment and some information on how to customise them.

## Plain Markdown

Not technically a site generator, but in case you just want the output of snakedown for whatever reason without having to scaffold a static site around it, you can use this option. As Markdown doesn't support themes, or other types of customisation, this option is pretty basic and doesn't come with a theme.

## Zola


[Zola](https://www.getzola.org/documentation/getting-started/overview/) is a static site generator written in rust and snakedown was originally written with Zola in mind. Snakedown will put the output in the `content` section of the site you provide, so once the output is generated you can develop and distribute your site like any other zola site.

Many themes allow customisation though the `config.toml` file of the zola site, or by modifying or extending the templates they provide. Please see the [zola documentation](https://www.getzola.org/documentation/themes/extending-a-theme/) or the theme documentation for more detail.

Currently snakedown only dumps the content and doesn't do much pre-processing, however in future we may provide output containing shortcodes so that the site can more easily customise them.

### Supported themes

Because the output of snakedown is just content that get's added to an existing site, and we don't yet produce shortcodes, you can currently use snakedown with any theme. However, the following themes are made to work with the output of snakedown with minimal configuration and should be kept up to date with new snakedown features.

- [Snakedown](https://github.com/aslowwriter/zola-snakedown-theme) The default snakedown theme for Zola, developed by the Snakedown devs. Based on the PyData Sphinx Theme.

Other compatible themes may be added here. If you know of one, please submit a PR to add it to this list!
