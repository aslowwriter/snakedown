# Getting Started

As an example we will go through how to setup a docs site with snakedown using [zola](https://www.getzola.org/documentation/getting-started/installation/) and the [snakedown theme](https://github.com/aslowwriter/zola-snakedown-theme). In addition to snakedown being installed (see [installing](installation.md))

to follow along with this example you'll also need `zola` and `git` installed. These are not strictly required to use snakedown as you can use it without them
but it is the easiest way.

Finally you'll also need a python package. For the purposes of this tutorial we will use the following file:

```python
"""A python module to demonstrate the functionality of snakedown."""

def fun():
    """this is a free function called fun!"""
    print("I'm having fun!")

Class Foo:
    """this is the docstring for the Foo class."""

    def bar(self) -> int:
        """this is a method called bar on the [[ test_pkg.Foo ]] class"""
        return 42
```

## Setup

First make a new folder for your python project. We'll call both the project and the package `pkg` as this is common for python projects:

```bash
mkdir pkg
```

now first create the folders for the python module and the documentation:

```bash
mkdir pkg/{pkg,docs}
```

After that create a file called `__init__.py` in the `pkg/pkg` folder to mark it as a package, and paste
the above python code into it.

the folder structure of your new directory should look like this:

```ignore
❯ eza --tree -L 2 --git-ignore -A pkg
pkg
├── docs
└── pkg
    └── __init__.py
```

> [!NOTE]
> `eza` is just an `ls` alternative we use here to print the directory structure, you don't need it for this tutorial.



## Zola site

We'll now first setup the site that the output will be added to. For this tutorial we will use Zola, but you can use any compatible SSG here, just make sure to setup the site however your ssg expects it.

Move into the directly and initialise the site:

```bash
cd pkg/docs
zola init
```

`zola` will now ask you a few questions about how you'd like to set up your site. For the purposes of this tutorial you can just accept all of the defaults.

after this the structure of your project should look like this:

```ignore
❯ eza --tree -L 2 --git-ignore -A pkg
pkg
├── docs
│   ├── config.toml
│   ├── content
│   ├── sass
│   ├── static
│   ├── templates
│   └── themes
└── pkg
    └── __init__.py
```

From the root of your zola site (the `docs` folder) add the snakedown theme to your site:

```bash
git clone https://github.com/aslowwriter/zola-snakedown-theme.git themes/snakedown
```

Also make sure to enable the theme in zola's `config.toml` by adding the following line:

```toml
theme = "snakedown"
```


## Snakedown

Now that the zola site is setup, we just need to setup snakedown itself. In this case all you have to do is create a `snakedown.toml` file in the root of the project (the top most `pkg` folder). You can also create a `pyproject.toml` file there if you prefer.

the `snakedown.toml` file should have the following contents:

```toml
api_content_path = "api/"
site_root = "docs"
pkg_path = "pkg"
ssg = "Zola"
```

Because we now have setup snakedown with all the information you don't even have to provide snakedown with any arguments, since it will take the information from the config file. You can just run it like this:

```bash
snakedown
```

Now you should see files being created:

```ignore
❯ eza --tree -L 4 --git-ignore -A
.
├── docs
│   ├── config.toml
│   ├── content
│   │   └── api
│   │       ├── pkg.foo.Foo._private.md
│   │       ├── pkg.foo.Foo.bar.md
│   │       ├── pkg.foo.Foo.md
│   │       ├── pkg.foo.fun.md
│   │       └── pkg.foo.md
│   ├── sass
│   ├── static
│   ├── templates
│   └── themes
├── pkg
│   ├── __init__.py
│   └── foo.py
└── snakedown.toml
```

If you want you can still override configurations in your config files by providing cli arguments:

```bash
snakedown pkg docs tmp
```

after which it should look like this:

```ignore
❯ eza --tree -L 4 --git-ignore -A
.
├── docs
│   ├── config.toml
│   ├── content
│   │   ├── api
│   │   │   ├── pkg.foo.Foo.bar.md
│   │   │   ├── pkg.foo.Foo.md
│   │   │   ├── pkg.foo.fun.md
│   │   │   └── pkg.foo.md
│   │   └── tmp
│   │       ├── pkg.foo.Foo.bar.md
│   │       ├── pkg.foo.Foo.md
│   │       ├── pkg.foo.fun.md
│   │       └── pkg.foo.md
│   ├── sass
│   ├── static
│   ├── templates
│   └── themes
├── pkg
│   ├── __init__.py
│   └── foo.py
└── snakedown.toml
```
