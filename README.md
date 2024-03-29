# *RSSG*
## A rusty static-site generator

### Contents

 - [Command-line usage](#command-line-usage)
 - [File structure](#file-structure)
 - [`rules.toml`](#rulestoml)
    - [Filters](#filters)
    - [Templates](#templates)
    - [Rules](#rules)
 - [Contributing](#contributing)
 - [Copyright](#copyright)

### Usage

**An example website can be found in the `example-site` directory.**

Use `rssg -i path-to-new-site` to create a new site.

---

### Command-line usage

```
rssg [options]
     -h |    --help : print this help dialog
     -c | --compile : compile the site
     -i |    --init : create a new site
     -v | --verbose : include debug output
     -f |   --force : force recompilation
                      rebuilds cache
          --content : set source directory
                      defaults to `content`
           --output : set output directory
                      defaults to `output`
           --public : set public directory
                      defaults to `public`
            --clean : cleans the `output`
                      and `temp` directories
```

### File structure
Websites use a structure to make compilation simpler. The structure looks like
this:
```
my_site/
├── content/
│   └── ...
├── templates/
│   └── ...
├── public/
│   └── ...
├── output/
│   └── ...
└── rules.toml
```

All your pages go into the `content` directory. These files will be processed
by the `rules.toml` (more about that below) which might run them through
templates in the `templates` directory. The results are placed into the
`output` directory. Any and all files in the `public` directory get copied into
the output without any modifications, preserving directory structure.

---

### `rules.toml`

The core of the generator is the `rules.toml`. It dictates what happens to
everything in the `content` directory. It's made up of two things: filters and
rules.

### Filters

Filters take an input file, run a command (probably to change the file), and
output a file. A command might look like `pandoc {full} -o {outfile}`. In this
example, two substitutions are made: first, `{full}` gets replaced with the
input files' full path; second, `{outfile}` gets replaced with the output file
of the filter. After substitution, it might look something like this:
`pandoc content/index.md -o temp/<hash>/index.html`.

All filters require an output file. This gets substituted, then substituted for
`{outfile}` in the command. The valid substitutions are as follows:
 - `{full}`: The full path to the input file.
 - `{dir}`: The parent directories of the input file.
 - `{name}`: The filename of the input file (minus extension).
 - `{ext}`: The extension of the input file.
 - `{parent}`: The direct parent of the input file.

In the `rules.toml`, filters can be in a list at the top-level of the file.
Here's an example that runs the file through `pandoc`, then outputs it without
changing it's path (but updating the extension):

```toml
[[filters]]
# The name of the filter, to use in rules
name = "markdown"

# The command to run, with substitutions
# example/path.html -> `pandoc example/path.html -o temp/<..>/path.html`
command = "pandoc {full} -o {outfile}"

# The resulting file, with substitutions
# !!! NOT neccessarily where the file will be in the final output
# example/path.html -> <hash of command + filepath>/path.html
outfile = "{dir}/{name}.html"
```

You can also specify "inline" filters inside of a rule specification (see
below), like so:
```toml
# ...
filters = [
    {command = "pandoc {full} -o {outfile}", outfile = "{dir}/{name}.html"}
]
# ...
```

*NOTE*: Filter outfiles are stored in the `temp` directory during generation,
with unique directory names. This is irrelevant for site development.

Filters can also omit the `outfile` property. Filters like this do not output
any information; as far as the other filters are concerned, they never existed.
It is possible for such a filter to directly mutate the output from a previous
filter, but this is inadvisable. These filters are intended for things like
logging and generating sitemaps.

Filters, by default, never see the raw source file. Even first-layer filters
only ever see a YAML-filtered version. However, if you have a filter without an
outfile, you can specify `give_original = true` in order to get the unchanged
source file path. ***Never*** use this to modify the source file, unless you
have an exceptional reason.

### Templates

Templates are files that you can use encapsulate other files. For example, you
might have a `default.html` template that contains a header and footer to wrap
your page content in. They reside in the `templates` directory, and have some
substitution rules of their own:
 - `{{data}}`: The full data of the page you are embedding.
 - `{{version}}`: The version of `rssg` used to compile the page.
 - `{{data.<key>}}`: Data from the content file's frontmatter.

Note that, unlike command substitutions, these are enclosed in double brackets.
Content files can have YAML frontmatter, to use in these substitutions.
For example, you might have a `title` key in each page, and a `title` element
in the template that uses the key. Frontmatter is enclosed on both sides by
triple-dashes (`---`) and *must* be at the start. Here's an example (assuming a
very basic markdown-to-html filter):

`input.md`
```md
---
title: Homepage
---

# Hello, World!
```

`template.html`
```html
<html>
    <head>
        <title>{{data.title}}</title>
    </head>
    <body>
        {{data}}
    </body>
</html>
```

`output.html`
```html
<html>
    <head>
        <title>Homepage</title>
    </head>
    <body>
        <h1>Hello, World!</h1>
    </body>
</html>
```

### Rules

Filters do nothing on their own; they have to be used inside of rules. Rules
are composed of four components: a regex pattern (`rule`), a list of filters,
(`filters`), a list of templates (`templates`), and an output (`output`). When
they are applied to a file, first they apply each filter to it in sequence.
Then they apply each template to it in sequence. The result is stored in the
output path, prefixed with `output`. Once one rule has matched a file, no other
rule can.

*NOTE*: Rules, unlike filters, store their output files directly in the output
directory.

This rule matches all files ending in `.md` or `.markdown`, translates them to
HTML, applies a template, then saves it in a directory named like it but with
the filename `index.html` (this turns the url `example.com/contact.html` into
`example.com/contact/`):
```toml
[[rules]]
# starts with any characters, ends in .md or .markdown
rule = ".*\\.(md|markdown)"

# Can also use inline filters, see [filters](#filters)
filters = ["markdown"]

# -> templates/default.html
templates = ["default.html"]

# example/path.html -> output/example/path/index.html
output = "{dir}/{name}/index.html"
```

Rules, like filters, can omit the `output` property. In this case, no templates
will be applied, and no files/directories created. The same warning goes for
rules as for filters; you really shouldn't mutate data from inside a no-output
rule.

These are just the recommended style guidelines. Any other way to create a TOML
list called `rules`, or `filters`, will work. This is just the cleanest way. If
you need to change it up for whatever reason, check out the official
[TOML website](https://toml.io).

### Pre- and post-commands

In your `rules.toml`, you can add arbitrary commands to run before and after
building your site. You can list your pre-commands in the root-level
`pre_commands` list, and your post-commands in the `post_commands` list.
Neither pre- nor post-commands undergo any substitution, they are run as-is and
will cause a build to fail on a non-zero exit code.

### Contributing

First, thank you for even considering contributing to the project!

There are several things that need improving right now. First of all, unit
tests should really get made. Several other little changes would be nice as
well; a watch-mode for development (watch the files and recompile when
something changes), a more configurable log system, better logging,
documentation, and general style improvements. This readme is indicative of the
rest of the project; functional, but flawed.

**Whatever you do, TEST it first!** Use the provided example (or your own) to
ensure correctness. If you add a feature, add a test for it.

Please run `cargo clippy` and `cargo fmt` before making any pull requests.
These not only help with style and performance issues, but clippy can also
inadvertently catch some bad bugs in your code. However, as long as your
contributions are helpful and functional, I won't be a stickler for formatting.

Unless you specifically state otherwise, all contributions are licensed under
the project license (MIT).

***Never submit incomplete code (`todo!()`, `unimplemented!()`, etc.)***

### Copyright

Copyright (c) 2023 Kyllingene, MIT license.
