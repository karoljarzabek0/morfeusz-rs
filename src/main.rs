use std::fs::File;
use std::io::{self, Read};
use fst::Map;
use memmap2::Mmap;
use anyhow::Result;
use std::time::Instant;

struct Dictionary {
    _mmap: Mmap, // Keep the mmap alive
    fst: Map<Vec<u8>>, // fst::Map can take a sub-slice
    rule_sets_off: usize,
    rules_off: usize,
    rule_idx_off: usize,
}

impl Dictionary {
    fn open() -> Result<Self> {
        let mut file = File::open("data/dictionary.bin")?;
        
        // 1. Read Header (64 bytes)
        let mut header = [0u8; 64];
        file.read_exact(&mut header)?;

        let fst_off = u64::from_le_bytes(header[0..8].try_into()?) as usize;
        let fst_len = u64::from_le_bytes(header[8..16].try_into()?) as usize;
        let rs_off = u64::from_le_bytes(header[16..24].try_into()?) as usize;
        let r_off = u64::from_le_bytes(header[32..40].try_into()?) as usize;
        let ri_off = u64::from_le_bytes(header[48..56].try_into()?) as usize;

        // 2. Mmap the whole file
        let mmap = unsafe { Mmap::map(&file)? };

        // 3. Create sub-slices
        // fst crate Map::new() wants a &[u8] but we need to own the data if we want to store it easily.
        // Actually we can store Map<&[u8]> but then we have lifetime issues.
        // A cleaner way for this demo is to just clone the FST bytes or use a different Map constructor.
        let fst_slice = mmap[fst_off..fst_off+fst_len].to_vec();
        let fst = Map::new(fst_slice)?;

        Ok(Self {
            _mmap: mmap,
            fst,
            rule_sets_off: rs_off,
            rules_off: r_off,
            rule_idx_off: ri_off,
        })
    }

    fn lookup(&self, word: &str) -> Vec<String> {
        let Some(set_offset) = self.fst.get(word) else { return vec![]; };
        let set_offset = (self.rule_sets_off + set_offset as usize) as usize;

        let count = self._mmap[set_offset] as usize;
        let mut results = Vec::with_capacity(count);

        for i in 0..count {
            let id_ptr = set_offset + 1 + i * 4;
            let rule_id = u32::from_le_bytes(self._mmap[id_ptr..id_ptr+4].try_into().unwrap()) as usize;

            let idx_ptr = self.rule_idx_off + rule_id * 4;
            let rule_offset = u32::from_le_bytes(self._mmap[idx_ptr..idx_ptr+4].try_into().unwrap()) as usize;
            let rule_abs_ptr = self.rules_off + rule_offset;

            let strip = self._mmap[rule_abs_ptr] as usize;
            let add_ptr = rule_abs_ptr + 1;
            let add_slice = &self._mmap[add_ptr..];
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
    println!("Loading packed dictionary (mmap)...");
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
