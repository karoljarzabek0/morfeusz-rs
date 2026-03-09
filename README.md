# morfeusz-rs

A highly optimized, memory-efficient morphological analyzer for the Polish language, written in Rust. 

This project is a clean-room implementation inspired by the original [Morfeusz](https://git.nlp.ipipan.waw.pl/SGJP/Morfeusz) tool by the Polish Academy of Sciences. It focuses on extreme memory compression and blazing-fast lookup speeds.

## Key Features

- **Massive Compression:** Compresses the 470MB Polish morphological dictionary (`.tab` format) down to a single **~12.8MB** binary file.
- **Lightning Fast:** Achieves ~60µs lookup times, benchmarking **~4x faster** than the original C++ implementation.
- **Zero-Copy Loading:** Uses memory-mapped files (`mmap`) for instant startup and minimal RAM footprint.
- **Rule-Based Deduplication:** Replaces explicit lemma storage with "strip and add" inflection rules, heavily deduplicated across millions of word forms.
- **Single Binary Dictionary:** All FST indices, rule sets, and string data are packed into one cohesive `dictionary.bin` file.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (edition 2024)
- A raw morphological dictionary file (e.g., `polimorf-20251116.tab`)

### 1. Generate the Optimized Dictionary

Before you can analyze words, you must compile the raw text dictionary into the optimized binary format.

```bash
cargo run --release --bin generate <path/to/polimorf.tab>
```

This will parse the dictionary, calculate the inflection rules, build the Finite State Transducer (FST), and pack everything into `data/dictionary.bin`.

### 2. Run the Analyzer

Once the dictionary is built, you can run the interactive CLI analyzer:

```bash
cargo run --release --bin morfeusz-rs
```

You will be prompted to enter a word. The tool will instantly return its base lemmas.

```text
-----------------------------------
Wpisz słowo, które chcesz wyszukać:
bankowi
Słowo: "bankowi", Lematy: ["bankowy", "bank"]
Lookup took 58.1µs seconds
```

## How It Works

Instead of storing `(form, lemma)` pairs like `(bankowi, bank)`, the generator calculates a transformation rule: "Strip 3 characters from the end, add nothing". 

1. **FST (Finite State Transducer):** Maps the inflected word to a rule set ID.
2. **Rule Sets:** Words with multiple meanings (like "domowi" -> "domowić", "dom", "domowy") point to a set of rule IDs.
3. **Rules:** The actual instructions (`strip_count`, `suffix_string`), deduplicated across the entire language.

All these components are packed into a single binary file with a custom header, allowing the runtime to memory-map exact segments without deserialization overhead.

## Benchmarks

Benchmarked against the original `morfeusz_analyzer.exe` performing 1000 lookups on Windows:

| Implementation | Time (1000 lookups) | Speedup |
| :--- | :--- | :--- |
| **Morfeusz (Original)** | 0.0617s | 1.0x |
| **morfeusz-rs (Rust)** | **0.0158s** | **3.92x** |

*(Tested on typical desktop hardware. Results may vary depending on OS and disk speeds).*
