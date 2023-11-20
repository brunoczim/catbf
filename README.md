# catbf

CatBF is a dialect of [Brainfuck](https://en.wikipedia.org/wiki/Brainfuck) and
its implementation. It is a complete brainfuck implementation, featuring an
interpreter, an AOT compiler and a JIT compiler  The tape is "infinite" both
forwards and backwards. Cells are 8-bit. Reading from stdin writes to two cells:
the first one is a "boolean" indicating whether a byte was read (false = EOF),
the second one is the byte read.

Currently, compilation is only supported for Linux x86-64.

# help

```
A complete brainfuck implementation: interpreter, Ahead-Of-Time (AOT) compiler and Just-In-Time (JIT) compiler.

The tape is "infinite" both forwards and backwards. Cells are 8-bit. Get from stdin writes to two cells: the first is a "boolean" indicating whether a byte was read (false = EOF), the second one is the byte read.

Usage: catbf [OPTIONS] <PATH>

Arguments:
  <PATH>
          Source file path

Options:
  -p, --print-ir
          Print intermediate representation

  -o, --compile-to <COMPILE_AOT>
          Compile the program Ahead-Of-Time (AOT) and place the artifacts into the directory indetified by the given path

  -j, --jit
          Compile the program Just-In-Time (JIT) and run it. If the target platform is not supported, this will fallback to an interpreted execution

  -J, --force-jit
          Force Just-In-Time (JIT) compilation of the program and run it. If the target platform is not supported, this will fail and the program will not be executed

  -h, --help
          Print help (see a summary with '-h')
```
