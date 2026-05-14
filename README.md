# tqec-rs

This project is a re-implementation of the [TQEC](https://github.com/tqec/tqec)
compiler in Rust.

## Why rust?

Over time, larger and larger quantum circuits will be used as input to
tqec, particuarly as more user-facing tools like [Piper Draw](https://github.com/Walrus-Computing/piper-draw) are added.
TQEC is written in Python, and will likely struggle to keep up with larger
circuits. As it stands, the compiler is not tested on large circuits.
For example, tqec takes ~2s to compile a logical CNOT gate, and ~9s
to compile a Steane circuit.

