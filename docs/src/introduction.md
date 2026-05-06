# Introduction

`snakedown` is a command line tool to extract API documentation from a python package. The most popular solutions like Sphinx and Quarto are fairly slow and require a lot of configuration, as well as having poor DevX.

Snakedown is designed to let you use your favourite static site generator to host your Python documentation.

Features include:
- Easily extract both type signatures and docstrings
- lightweight syntax for linking to other objects in docstrings, inspired by Obsidian
- Can link to other Sphinx sites like the `numpy` docs
- Themes in supported static site generators give you precie control over the look of your docs

It currently supports the following static site generators and themes:

- [Zola](https://getzola.org)
    - [snakedown](https://github.com/aslowwriter/zola-snakedown-theme)
