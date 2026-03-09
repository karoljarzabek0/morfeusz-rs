use std::fs::File;
use std::io;
use fst::Map;
use memmap2::Mmap;
use anyhow::Result;
use std::time::Instant;

struct Dictionary {
    fst: Map<Mmap>,
    rule_sets: Mmap,
    rules: Mmap,
    rule_idx: Mmap,
}

impl Dictionary {
    fn open() -> Result<Self> {
        let fst_file = File::open("data/dict.fst")?;
        let rule_sets_file = File::open("data/rule_sets.bin")?;
        let rules_file = File::open("data/rules.bin")?;
        let rule_idx_file = File::open("data/rules.idx")?;

        Ok(Self {
            fst: Map::new(unsafe { Mmap::map(&fst_file)? })?,
            rule_sets: unsafe { Mmap::map(&rule_sets_file)? },
            rules: unsafe { Mmap::map(&rules_file)? },
            rule_idx: unsafe { Mmap::map(&rule_idx_file)? },
        })
    }

    fn lookup(&self, word: &str) -> Vec<String> {
        let Some(set_offset) = self.fst.get(word) else { return vec![]; };
        let set_offset = set_offset as usize;

        let count = self.rule_sets[set_offset] as usize;
        let mut results = Vec::with_capacity(count);

        for i in 0..count {
            let id_ptr = set_offset + 1 + i * 4;
            let rule_id = u32::from_le_bytes(self.rule_sets[id_ptr..id_ptr+4].try_into().unwrap()) as usize;

            let idx_ptr = rule_id * 4;
            let rule_offset = u32::from_le_bytes(self.rule_idx[idx_ptr..idx_ptr+4].try_into().unwrap()) as usize;

            let strip = self.rules[rule_offset] as usize;
            let add_ptr = rule_offset + 1;
            let add_slice = &self.rules[add_ptr..];
            let add_len = add_slice.iter().position(|&b| b == 0).unwrap();
            let add_str = std::str::from_utf8(&add_slice[..add_len]).unwrap();

            let chars: Vec<char> = word.chars().collect();
            if chars.len() >= strip {
                let mut lemma: String = chars[..chars.len() - strip].iter().collect();
                lemma.push_str(add_str);
                results.push(lemma);
            }
        }
        results
    }
}

fn main() -> Result<()> {
    println!("Loading dictionary (mmap)...");
    let dict = Dictionary::open()?;
    println!("Dictionary loaded.");

    let mut term = String::new();
    loop {
        term.clear();
        println!("-----------------------------------");
        println!("Wpisz słowo, które chcesz wyszukać:");

        let bytes_read = io::stdin().read_line(&mut term)?;
        if bytes_read == 0 { break; }
        
        let term = term.trim();
        if term.is_empty() { continue; }

        let start = Instant::now();
        let lemmas = dict.lookup(term);

        if lemmas.is_empty() {
            println!("Słowo: \"{}\", Lemat: Brak", term);
        } else {
            println!("Słowo: \"{}\", Lematy: {:?}", term, lemmas);
        }

        let elapsed = start.elapsed();
        println!("Lookup took {:?} seconds", elapsed);
    }
    Ok(())
}
