use rand::Rng;
use rayon::prelude::*;
use std::time::Instant;
use std::io::{self, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

// Konfigurationskonstanten
const MIN_TROPFEN: u64 = 1000;
const DEFAULT_TROPFEN: u64 = 1_000_000;
const CHUNK_SIZE: u64 = 10_000;

#[derive(Debug)]
struct Ergebnis {
    pi_approx: f64,
    dauer: std::time::Duration,
    tropfenzahl: u64,
    threads: usize,
}

fn approximiere_pi(tropfenzahl: u64) -> f64 {
    let threads = rayon::current_num_threads();
    let counter = Arc::new(AtomicU64::new(0));
    
    // Berechne die optimale Chunk-Größe
    let chunk_size = (tropfenzahl / (threads as u64 * 10))
        .max(CHUNK_SIZE)
        .min(tropfenzahl);
    
    // Erstelle einen Vektor von Chunk-Indizes
    let chunks: Vec<u64> = (0..tropfenzahl)
        .step_by(chunk_size as usize)
        .collect();
    
    // Parallele Verarbeitung der Chunks
    chunks.par_iter()
        .for_each(|&start| {
            let mut rng = rand::thread_rng();
            let mut local_count = 0;
            let end = (start + chunk_size).min(tropfenzahl);
            
            for _ in start..end {
                let x: f64 = rng.gen();
                let y: f64 = rng.gen();
                
                if x * x + y * y <= 1.0 {
                    local_count += 1;
                }
            }
            
            counter.fetch_add(local_count, Ordering::Relaxed);
        });

    4.0 * (counter.load(Ordering::Relaxed) as f64) / (tropfenzahl as f64)
}

fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or_default();
    input.trim().to_string()
}

fn validate_tropfenzahl(input: &str) -> Result<u64, &'static str> {
    match input.parse::<u64>() {
        Ok(n) if n >= MIN_TROPFEN => Ok(n),
        Ok(_) => Err("Tropfenzahl muss mindestens 1000 sein."),
        Err(_) => Err("Ungültige Eingabe. Bitte geben Sie eine ganze Zahl ein."),
    }
}

fn berechne_pi() -> Result<Ergebnis, &'static str> {
    let input = get_user_input("\nGib die Anzahl der Tropfen (Punkte) ein: ");
    
    let tropfenzahl = validate_tropfenzahl(&input).unwrap_or_else(|_| {
        println!("Verwende Standardwert von {} Tropfen.", DEFAULT_TROPFEN);
        DEFAULT_TROPFEN
    });
    
    if get_user_input("\nMöchtest du mit der Berechnung fortfahren? (Y/n): ")
        .to_lowercase()
        .starts_with('n') {
        return Err("Berechnung abgebrochen.");
    }

    let threads = rayon::current_num_threads();
    let start = Instant::now();
    let pi_approx = approximiere_pi(tropfenzahl);
    let dauer = start.elapsed();

    Ok(Ergebnis {
        pi_approx,
        dauer,
        tropfenzahl,
        threads,
    })
}

fn main() {
    println!("\nZusammengekleistert von Luis Weitl für den Matheunterricht mit der Programmiersprache Rust.");
    println!("Annäherung von Pi (π) durch den Monte-Carlo-Algorithmus.");
    println!("Implementiert in Rust mit der Rayon Bibliothek für die parallele Berechnung");
    println!("Der Quellcode kann hier gefunden werden: https://github.com/luigiistcrazy/pi_approximieren");
    println!("\nVerfügbare Threads: {}", rayon::current_num_threads());

    match berechne_pi() {
        Ok(ergebnis) => {
            println!("\nErgebnis:");
            println!("π ≈ {:.10}", ergebnis.pi_approx);
            println!("Verwendete Tropfen: {}", ergebnis.tropfenzahl);
            println!("Berechnungsdauer: {:.2?}", ergebnis.dauer);
            println!("Verwendete Threads: {}", ergebnis.threads);
        }
        Err(e) => println!("\n{}", e),
    }
}
