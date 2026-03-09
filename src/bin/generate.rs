use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write, Seek, SeekFrom};
use std::collections::{HashMap, BTreeSet};
use fst::MapBuilder;
use anyhow::Result;

fn get_common_prefix_len(s1: &str, s2: &str) -> usize {
    s1.chars()
        .zip(s2.chars())
        .take_while(|(c1, c2)| c1 == c2)
        .count()
}

fn get_rule(form: &str, lemma: &str) -> (usize, String) {
    let prefix_len = get_common_prefix_len(form, lemma);
    let strip = form.chars().count() - prefix_len;
    let add = lemma.chars().skip(prefix_len).collect::<String>();
    (strip, add)
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        anyhow::bail!("Usage: {} <path_to_polimorf.tab>", args[0]);
    }
    let path = &args[1];
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    println!("Loading dictionary from {}...", path);
    let mut form_to_rules: HashMap<String, BTreeSet<(usize, String)>> = HashMap::new();
    let mut total_entries = 0;
    for line in reader.lines() {
        let line = line?;
        if line.starts_with('#') || line.trim().is_empty() { continue; }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 { continue; }
        let (form, lemma) = (parts[0], parts[1]);
        form_to_rules.entry(form.to_string())
            .or_default()
            .insert(get_rule(form, lemma));
        
        total_entries += 1;
        if total_entries % 1_000_000 == 0 {
            println!("Processed {}M entries...", total_entries / 1_000_000);
        }
    }

    println!("Deduplicating rule sets...");
    let mut rule_to_id = HashMap::new();
    let mut id_to_rule = Vec::new();
    let mut rule_set_to_rule_ids = HashMap::new();

    for rules in form_to_rules.values() {
        if rule_set_to_rule_ids.contains_key(rules) { continue; }
        let mut rule_ids = Vec::new();
        for rule in rules {
            let id = *rule_to_id.entry(rule.clone()).or_insert_with(|| {
                id_to_rule.push(rule.clone());
                (id_to_rule.len() - 1) as u32
            });
            rule_ids.push(id);
        }
        rule_set_to_rule_ids.insert(rules.clone(), rule_ids);
    }

    // --- PACKING START ---
    let mut out_file = File::create("data/dictionary.bin")?;
    out_file.write_all(&[0u8; 64])?; // Placeholder for header

    // 1. Save Rules
    let r_off = out_file.stream_position()?;
    for (strip, add) in &id_to_rule {
        out_file.write_all(&[*strip as u8])?;
        out_file.write_all(add.as_bytes())?;
        out_file.write_all(&[0])?;
    }
    let r_len = out_file.stream_position()? - r_off;

    // 2. Save Rule Index
    let ri_off = out_file.stream_position()?;
    let mut current_rule_offset = 0u32;
    for (_, add) in &id_to_rule {
        out_file.write_all(&current_rule_offset.to_le_bytes())?;
        current_rule_offset += (1 + add.as_bytes().len() + 1) as u32;
    }
    let ri_len = out_file.stream_position()? - ri_off;

    // 3. Save Rule Sets
    let rs_off = out_file.stream_position()?;
    let mut rule_set_to_offset = HashMap::new();
    let mut current_set_offset = 0u64;
    for (rules, rule_ids) in &rule_set_to_rule_ids {
        rule_set_to_offset.insert(rules.clone(), current_set_offset);
        out_file.write_all(&[rule_ids.len() as u8])?;
        for &id in rule_ids {
            out_file.write_all(&id.to_le_bytes())?;
        }
        current_set_offset += (1 + rule_ids.len() * 4) as u64;
    }
    let rs_len = out_file.stream_position()? - rs_off;

    // 4. Save FST
    let fst_off = out_file.stream_position()?;
    let mut sorted_forms: Vec<_> = form_to_rules.keys().collect();
    sorted_forms.sort();
    
    // Use a scoped writer so we can flush it before getting final file position
    {
        let mut builder = MapBuilder::new(&out_file)?;
        for form in sorted_forms {
            let rules = &form_to_rules[form];
            let offset = rule_set_to_offset[rules];
            builder.insert(form, offset)?;
        }
        builder.finish()?;
    }
    let fst_len = out_file.stream_position()? - fst_off;

    // 5. Finalize Header
    println!("Finalizing header...");
    out_file.seek(SeekFrom::Start(0))?;
    out_file.write_all(&fst_off.to_le_bytes())?;
    out_file.write_all(&fst_len.to_le_bytes())?;
    out_file.write_all(&rs_off.to_le_bytes())?;
    out_file.write_all(&rs_len.to_le_bytes())?;
    out_file.write_all(&r_off.to_le_bytes())?;
    out_file.write_all(&r_len.to_le_bytes())?;
    out_file.write_all(&ri_off.to_le_bytes())?;
    out_file.write_all(&ri_len.to_le_bytes())?;

    println!("Done! Packed dictionary saved to data/dictionary.bin");
    Ok(())
}
