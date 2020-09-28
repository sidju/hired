# hired
## The ed rewrite for daily use

After tiring of emacs freezing up on me one time too many and concluding that vim isn't any less of a monstrosity I made the reasonable choice and started using ed.
It was a bit frustrating at the start but grew to be quite nice. The only missing feature really being syntax highlighting. After some research I couldn't find any ed fork or clone with syntax highlighting, so I once again made the reasonable choice and wrote my own.
And so here it is, with all its flaws and deficiencies. Any and all pull requests welcome.

## Core concepts:
### The selection:
The original 'ed' keeps track of one line that you recently interacted with and defaults to working on that if no lines are given with a command. This is an extension of that logic, making it a span of lines. I find that this is more intuitive.
(To avoid unpleasantries some commands don't default to the selection, such as 'w'. If you want to modify the selection behavior for any command create an issue, I may well have missed one.)

### The view:
The original 'ed' allows you to set the 'p' flag on most commands, causing the affected lines to be printed after being changed. On a modern terminal there is no real reason not to do this by default on any text change, so I have written in the view.
It is intended to print as much as fits on the screen from 5 lines before the start of the current selection. Currently it prints unless a print command printed, but this will be adjusted to only print if an editing command was called.

## Commands:
### Lone commands:
Commands that take no input or selection.
- q: Quit. Returns error if you have unsaved changes.
- Q: Force Quit. Ignores unsaved changes.
- h: Print Error. Print the last occured error.
- H: Toggle Error Printing. Toggle printing errors as they occur.

### File commands:
- f: If no filepath is given, prints filepath. Else sets the filepath to given path.
- e: Closes current file and opens the given path. Returns error if you have unsaved changes.
- E: Closes current file and opens the given path, ignoring unsaved changes.
- r: Append the data from given path to given selection.

- w: Write the given selection (default whole buffer) to given path (default filepath). Waring, silently overwrites the file.
- W: Append the given selection (default whole buffer) to given path (default filepath).

### Print commands:
- p: Print the given selection.
- n: Print the given selection with numbered lines.
- l: Print the given selection with character escaping. (NOT IMPLEMENTED)

### Basic editing commands:
- a: Append. Append lines given after the command to given selection. Stop entry with only '.' on a line.
- i: Insert. Insert lines given after the command before given selection. Stop entry with only '.' on a line.
- c: Change. Replace given selection with lines given after the command. Stop entry with only '.' on a line.
- d: Delete. Delete the given selection.

### Advanced editing commands:
- m: Move. Move given selection to given index.
- t: Transfer. Copy given selection to given index.
- j: Join. Append together given selection into one line.

### Regex commands:
- s: Substitute. Regex replace, just like 'sed'.

### Special cases:
- no command: Takes the given selection and sets it to the default selection.

### Commands not yet implemented:
Note that these aren't set in stone (nor are the others). If a better idea comes up or people believe something would be confusing I'll skip or adjust it.

#### Easy to implement:
- A: Append Inline. Appends to the same line rather that creating a new line after.
- I: Insert Inline. Inserts at the start of the same line rather that creating a new line before. (Perfect for commenting out)
- J: Re-Join. Join all the lines in the selection and then split them (following word boundaries) at the given number of columns. Will not handle adding // before comments or anything, but a macro could probably see you through when those come...
- P: Toggle View. Toggle wether the View is printed after each editing command.

#### Hard to implement:
- C: Change Inline. Change the given line by opening it in a one-line editing buffer. If multiple lines enter submits current and moves to next one. '\n' in a line is substituted with newline. (Requires small editing buffer)
- g: Group. A macro command that executes a given list of commands on all lines in the selection that match a given regex. Will require considerable work in the command logic and I have yet to feel a need for it myself.
- !: Run. A way to do ANYTHING, if I can manage it. The given selection of lines is piped into your shell with the given command. Some way of flagging what to do with the output is pending desing. Flags 'c','a','i' and 'p' would require custom parsing, but not too much....
- :: Macro. Intended to allow the user to define macros of commands in the eventually created config file. This would allow ':push' to be the same as 'w' and '!git push', or whatever the user wishes.

## Attributions:
This project has of course greatly benefited from all the crates it depends on. Especially I'd like to thank regex and syntect for helping me through my, to various degrees badly though out, issues.

Then I have also gotten a hand up from 'bat', which I also consider an excellent companion to this application, from their handling of 16-color terminals. My theme is currently copied from their repo and will probably always be based on theirs.
