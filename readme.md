# micro_6502

A functioning 6502 microprocessor emulator.

## Usage

To run a program using the emulator, run:

```
cargo run -- path/to/prog.bin
```

You can also specify the registers and memory buffer to initialize the program with:

```
cargo run -- path/to/prog.bin --regs x=3,y=2 --memory path/to/mem.bin
```

## Examples

There is an `examples/` directory that contain some example programs.

### Fibonacci

Computes the nth fibonacci number contained in the `x` register, outputting the result in the `y` register.

Note: due to the limitations of the architecture of the 6502 processor, the largest number you can generate is the 13th number in the sequence, i.e. 233.

```
cargo run -- examples/fibonacci.bin --regs x=(whatever)
```

---

Made by [dcas796](https://dcas796.github.com/)
