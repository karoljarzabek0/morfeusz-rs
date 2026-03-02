// Libraries for the file reader
use std::fs::{File};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;

// Hashmap
use std::collections::HashMap;

// Time
use std::time;

// CLI args
use std::env;


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

fn read_dictionary(path: &str) -> HashMap<String, String> {
    let start = time::Instant::now();
    let n_lines = count_lines(path).expect("Failed to count lines");

    let mut full_dictionary: HashMap<String, String> = HashMap::with_capacity(n_lines);

    if let Ok(lines) = read_lines(path) {
        let mut i = 1;
        for line in lines.flatten() {
            let record: Vec<&str> = line.split("\t").collect();
            if record.len() > 1 {
            //println!("{:?}", record);
            full_dictionary.insert(record[0].to_string(), record[1].to_string());
            i += 1;
            if i % 10_000 == 0 {
                println!("{}", i);
            }
            }
        }
    }
    let elapsed_time = start.elapsed();
    println!("Full dictionary loaded in {} seconds", elapsed_time.as_secs_f32());
    full_dictionary
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() !=2 {
        eprintln!("Usage: {} <dic_path>", args[0]);
        panic!("Wrong");
    }
    let dictionary_path = &args[1];
    let dictionary = read_dictionary(dictionary_path);
    
    let mut term = String::new();
    loop {
        term.clear();
        println!("-----------------------------------");
        println!("Wpisz słowo, które chcesz wyszukać:");

        io::stdin().read_line(&mut term).expect("Błąd w odczytywaniu słowa");
        let start = time::Instant::now();
        let term = term.trim();

        let lemma = dictionary.get(term).map_or("Brak", |v| v.as_str());

        println!("Słowo: \"{}\", Lemat: \"{}\"", term, lemma);

        let elapsed_time = start.elapsed();
        println!("Finiding lemma took {} seconds", elapsed_time.as_secs_f32())
    }
}
