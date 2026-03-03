// Libraries for the file reader
use std::fs::{File};
use std::io::{self, BufRead, BufReader, Read, BufWriter};
use std::path::Path;

// Hashmap
use std::collections::{HashMap, BTreeMap};

// Time
use std::{time, vec};

// CLI args
use std::env;

// FST
use fst::{MapBuilder, Map};

use memmap2::Mmap;

use anyhow;

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn count_lines(path: &str) -> io::Result<usize> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut count = 0;
    let mut buffer = [0; 16384]; // 16KB buffer

    while let Ok(n) = reader.read(&mut buffer) {
        if n == 0 { break; }
        count += buffer[..n].iter().filter(|&&b| b == b'\n').count();
    }
    Ok(count)
}

fn load_btree(path: &str) -> BTreeMap<String, Vec<String>> {
    let mut btree: BTreeMap<String, Vec<String>> = BTreeMap::new();

    if let Ok(lines) = read_lines(path) {
        let mut i = 1;

        for line in lines.flatten() {
            let record: Vec<&str> = line.split('\t').collect();
            if record.len() > 1 {
                let key = record[0].to_string();
                let val = record[1].to_string();

                btree.entry(key)
                    .and_modify(|vec| {
                        if !vec.contains(&val) {
                            vec.push(val.clone());
                        }
                    })
                    .or_insert(vec![val]);

                i += 1;
                if i % 100_000 == 0 {
                    println!("{}", i);
                }
            }
        }
    }
    btree
}

fn generate_fst(btree: &BTreeMap<String, Vec<String>>) {
    let fst_file = File::create("data/dict.fst").unwrap();
    let fst_writer = BufWriter::new(fst_file);
    let mut fst_builder = MapBuilder::new(fst_writer).unwrap();

    let data_file = File::create("data/dict.data").unwrap();
    let mut data_writer = BufWriter::new(data_file);
    
    let mut current_offset = 0u64;

    for (word, lemmas) in btree {
        // 1. Write lemmas to the data file
        let start_offset = current_offset;
        for lemma in lemmas {
            let bytes = lemma.as_bytes();
            std::io::Write::write_all(&mut data_writer, bytes).unwrap();
            std::io::Write::write_all(&mut data_writer, &[0]).unwrap(); // Null terminator
            current_offset += (bytes.len() + 1) as u64;
        }

        // 2. Pack (Offset: 48 bits | Count: 16 bits)
        let packed = (start_offset << 16) | (lemmas.len() as u64 & 0xFFFF);
        
        // 3. Insert into FST (Keys MUST be sorted)
        fst_builder.insert(word, packed).unwrap();
    }

    fst_builder.finish().unwrap();
}

struct Dictionary {
    index: Map<Mmap>,
    data: Mmap,
}

impl Dictionary {
    fn open() -> anyhow::Result<Self> {
        let index_file = File::open("data/dict.fst")?;
        let data_file = File::open("data/dict.data")?;
        
        Ok(Self {
            index: Map::new(unsafe { Mmap::map(&index_file)? })?,
            data: unsafe { Mmap::map(&data_file)? },
        })
    }

    fn lookup(&self, word: &str) -> Vec<&str> {
        let Some(packed) = self.index.get(word) else { return vec![]; };
        
        let offset = (packed >> 16) as usize;
        let count = (packed & 0xFFFF) as usize;
        
        let mut results = Vec::with_capacity(count);
        let mut ptr = offset;

        for _ in 0..count {
            // Find null terminator in the mmap'd data
            let slice = &self.data[ptr..];
            let len = slice.iter().position(|&b| b == 0).unwrap();
            results.push(std::str::from_utf8(&slice[..len]).unwrap());
            ptr += len + 1;
        }
        results
    }
}



fn prompt_loop(dictionary: &BTreeMap<String, Vec<String>>) {
    let mut term = String::new();
    let default_vec = vec!["Brak".to_string()];
    loop {
        term.clear();
        println!("-----------------------------------");
        println!("Wpisz słowo, które chcesz wyszukać:");

        io::stdin().read_line(&mut term).expect("Błąd w odczytywaniu słowa");

        let term = term.trim();

        if term == "print" {
            for (word, lemma_vec) in dictionary.iter() {
                println!("{}: {:?}", word, lemma_vec);
            }
        } else {

            let start = time::Instant::now();
            let lemma = dictionary.get(term).unwrap_or(&default_vec);

            println!("Słowo: \"{}\", Lemat: \"{:?}\"", term, lemma);

            let elapsed_time = start.elapsed();
            println!("Finiding lemma took {} seconds", elapsed_time.as_secs_f32())
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() !=2 {
        eprintln!("Usage: {} <dic_path>", args[0]);
        panic!("Wrong");
    }
    let dictionary_path = &args[1];
    //let dictionary = read_dictionary(dictionary_path);
    let dictionary = load_btree(dictionary_path);
    
    generate_fst(&dictionary);

    let dict = Dictionary::open()?;

    // 2. Perform a lookup
    let start = time::Instant::now();
    let word = "bankowi";
    let lemmas = dict.lookup(word);

    if lemmas.is_empty() {
        println!("'{}' not found.", word);
    } else {
        println!("Found {} for '{}': {:?}", lemmas.len(), word, lemmas);
    }
    let elapsed_time = start.elapsed();
    println!("Lookup took {} seconds", elapsed_time.as_secs_f32());

    prompt_loop(&dictionary);

    Ok(())

}
