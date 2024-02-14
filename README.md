# hired
## The ed rewrite for daily use

After tiring of emacs freezing up on me one time too many and concluding that vim isn't any less of a monstrosity I made the reasonable choice and started using ed.
It was a bit frustrating at the start but grew to be quite nice. The only missing feature really being syntax highlighting. After some research I couldn't find any ed fork or clone with syntax highlighting, so I once again made the reasonable choice and wrote my own.
And so here it is, with all its flaws and deficiencies. Any and all pull requests welcome.

## Core concepts:
### The selection:
The original 'ed' keeps track of one line that you recently interacted with and defaults to working on that if no
lines are given with a command. This is an extension of that logic, making it a span of lines. I find that this
is more intuitive.
(To avoid unpleasantries some commands don't default to the selection, such as 'w'. If you want to modify
the selection behavior for any command create an issue, I may well have missed one.)

### Usability:
The original 'ed' has very few features in its input editing. To improve on that this rewrite adds features
for moving both within lines and between lines in input. Similar capabilities exist for command input and
more are planned

## Commands:
For details on commands instead look at the
[add-ed repository](https://github.com/sidju/add-ed), which houses the library
that parses and runs the commands.

## Attributions:
This project has of course greatly benefited from all the crates it depends on. Especially I'd like to thank regex and syntect for helping me through my, to various degrees badly though out, issues.

Then I have also gotten a hand up from 'bat', which I also consider an excellent companion to this application, from their handling of 16-color terminals. My theme is currently copied from their repo and will probably always be based on theirs.

## Build:

```sh
git clone --recurse-submodules https://github.com/sidju/hired
cd hired
cargo build
```

Things to note:
- run `gh repo clone sidju/hired -- --recurse-submodules` instead of `git clone` in case using [GitHub's CLI](https://github.com/cli/cli)
- if `--recurse-submodules` was omitted, no highlights would be available

## Backing library

`hired` is built on the `ed` runtime of
[add-ed](https://github.com/sidju/add-ed).

If you wish to build your own editor using `ed` syntax you
are advised to give it a look. It allows replacing the IO
(file and shell interaction) as well as UI implementations
to better suit your needs.
