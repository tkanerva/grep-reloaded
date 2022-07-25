# grep-reloaded
Grep Reloaded (grr) is a GNU grep replacement written in Rust

# objective
The main idea is to replace the old grep written in C with a modern high level language. Like Rust.
Using high level languages allows one to rethink the structure of code, to allow pluggable modules etc.
The funky thing about Rust is the ability to run a global optimizer on the whole codebase.
Possibly not having to sacrifice much if any performance while going from C to a higher level language is a boon.

# todo
- Switch to BufReader to avoid reading the file in one bit (also allows stdin streaming).
- Develop a nice plugin interface to make it easier to extend the command with custom functions.
- Implement some of the more useful GNU grep flags.
- make the code more DRY.

