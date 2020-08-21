# hired
## The ed rewrite for daily use

After tiring of emacs freezing up on me one time too many and concluding that vim isn't any less of a monstrosity I made the reasonable choice and started using ed.
It was a bit frustrating at the start but grew to be quite nice. The only missing feature really being syntax highlighting. After some research I couldn't find any ed fork or clone with syntax highlighting, so I once again made the reasonable choice and wrote my own.
And so here it is, with all its flaws and deficiencies. Any and all pull requests welcome.

## Commands:
### Lone commands:
Commands that take no input or selection.
- q: Quit. Returns error if you have unsaved changes.
- Q: Force Quit. Ignores unsaved changes.
- h: Print Error. Print the last occured error.
- H: Toggle Error Printing. Toggle printing errors as they occur.

### File errors:
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
