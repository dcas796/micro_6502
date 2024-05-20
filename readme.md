# micro_6502

A functioning 6502 microprocessor emulator.

This crate contains a library and an executable that also serves as an example.

## Usage (library)

The main component of this library is the `Emulator` struct. This struct contains all the logic that runs the virtual CPU. To initialize a new instance of this struct, you will need a `Decoder` struct, a struct that implements the `ReadWritable` trait, and a `Regs` struct.

The `ReadWritable` trait provides a simple interface that allows the user to implement their own buses and connect the virtual CPU to peripherals. This library comes built in with a default `ReadWritable` struct—the `Memory` struct—that delivers a blank buffer that the CPU can access.

The following code instantiates a new virtual 6502 processor that runs the user-specified program:

```rust
use std::{fs::read, path::PathBuf};

use micro_6502::{
    decoder::Decoder, emulator::Emulator, instruction::InstructionRegistry, mem::Memory, regs::Regs,
};

fn main() {
    assert!(
        std::env::args().len() == 2,
        "You need to provide a binary to run."
    );
    let program_bytes = {
        let args: Vec<String> = std::env::args().collect();
        let program_path: PathBuf = args[1].clone().try_into().expect("Cannot parse file path.");
        read(&program_path)
            .expect(format!("Cannot access the file at '{}'", program_path.display()).as_str())
    };

    let registry = InstructionRegistry::new();
    let mut decoder = Decoder::new(registry, program_bytes);
    let mut memory = Memory::new();
    let mut regs = Regs::new();
    let mut emulator = Emulator::new(&mut decoder, &mut memory, &mut regs);
    emulator.run_until_completion();

    println!("Registers: {}", emulator.get_regs());
}

```

## Usage (executable)

To run a program using the emulator, run:

```
cargo run --features build-binary -- path/to/prog.bin
```

You can also specify the registers and memory buffer to initialize the program with:

```
cargo run --features build-binary -- path/to/prog.bin --regs x=3,y=2 --memory path/to/mem.bin
```

## Examples

There is an `examples/` directory that contain some example programs.

### Fibonacci

Computes the nth fibonacci number contained in the `x` register, outputting the result in the `y` register.

Note: due to the limitations of the architecture of the 6502 processor, the largest number you can generate is the 13th number in the sequence, i.e. 233.

```
cargo run --features build-binary -- examples/fibonacci.bin --regs x=(whatever)
```

---

Made by [dcas796](https://dcas796.github.com/)
