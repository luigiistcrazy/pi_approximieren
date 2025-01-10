// Externe Bibliotheken
use rand::Rng;                                   // Für Zufallszahlengenerierung
use rayon::prelude::*;                           // Für parallele Berechnung
use std::time::Instant;                          // Für Zeitmessung
use std::io::{self, Write};                      // Für Ein-/Ausgabe-Operationen
use std::sync::atomic::{AtomicU64, Ordering};    // Für thread-sichere Zähler
use std::sync::Arc;                              // Für thread-sicheres Reference Counting

// Konfigurationskonstanten
const MIN_TROPFEN: u64 = 1000;           // Minimale Anzahl von Punkten für aussagekräftige Ergebnisse
const DEFAULT_TROPFEN: u64 = 1_000_000;  // Standardwert für Punktanzahl
const CHUNK_SIZE: u64 = 10_000;          // Minimale Chunk-Größe für parallele Verarbeitung

// Struktur zur Speicherung der Berechnungsergebnisse
#[derive(Debug)]
struct Ergebnis {
    pi_approx: f64,              // Angenäherter Pi Wert
    dauer: std::time::Duration,  // Berechnungsdauer
    tropfenzahl: u64,            // Anzahl verwendeter Punkte
    threads: usize,              // Anzahl verwendeter Threads
}

// Hauptfunktion zur Pi-annäherung mit der Monte-Carlo Methode
// Funktion:
// - Verhältnis der Fläche eines Viertelkreises zur Fläche eines Quadranten
// - A_Kreis / A_Quadrat = π/4
// - Daraus folgt: π ≈ 4 * (Punkte im Kreis / Gesamtpunkte)
fn approximiere_pi(tropfenzahl: u64) -> f64 {
    let threads = rayon::current_num_threads();  // Ermittle verfügbare Threads
    let counter = Arc::new(AtomicU64::new(0));   // Thread-sicherer Zähler für Treffer im Kreis
    
    // Berechne optimale Chunk Größe für Load Balancing
    // - Mindestens CHUNK_SIZE (10.000)
    // - Maximal tropfenzahl
    // - Ziel: ca. 10 Chunks pro Thread
    let chunk_size = (tropfenzahl / (threads as u64 * 10))
        .max(CHUNK_SIZE)
        .min(tropfenzahl);
    
    // Erstelle Vektor mit Start Indizes für jeden Chunk
    let chunks: Vec<u64> = (0..tropfenzahl)
        .step_by(chunk_size as usize)
        .collect();
    
    // Parallele Verarbeitung der Chunks
    chunks.par_iter()  // Parallelisierung mittels Rayon
        .for_each(|&start| {
            let mut rng = rand::thread_rng();  // Thread lokaler Zufallszahlengenerator
            let mut local_count = 0;           // Lokaler Zähler für diesen Chunk
            let end = (start + chunk_size).min(tropfenzahl);
            
            // Monte-Carlo-Simulation für diesen Chunk
            for _ in start..end {
                // Generiere zufällige Punkte im Einheitsquadrat [0,1] × [0,1]
                let x: f64 = rng.gen();  // Zufällige x-Koordinate
                let y: f64 = rng.gen();  // Zufällige y-Koordinate
                
                // Prüfe, ob Punkt im Viertelkreis liegt (x² + y² ≤ 1)
                if x * x + y * y <= 1.0 {
                    local_count += 1;
                }
            }
            
            // Atomare Addition zum Gesamtzähler
            counter.fetch_add(local_count, Ordering::Relaxed);
        });

    // Berechne Pi-Approximation
    // π ≈ 4 * (Punkte im Kreis / Gesamtpunkte)
    4.0 * (counter.load(Ordering::Relaxed) as f64) / (tropfenzahl as f64)
}

// Hilfsfunktion für Benutzereingabe
fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or_default();
    input.trim().to_string()
}

// Validierung der eingegebenen Tropfenzahl
fn validate_tropfenzahl(input: &str) -> Result<u64, &'static str> {
    match input.parse::<u64>() {
        Ok(n) if n >= MIN_TROPFEN => Ok(n),
        Ok(_) => Err("Tropfenzahl muss mindestens 1000 sein."),
        Err(_) => Err("Ungültige Eingabe. Bitte geben Sie eine ganze Zahl ein."),
    }
}

// Hauptfunktion zur Benutzerinteraktion und Berechnung
fn berechne_pi() -> Result<Ergebnis, &'static str> {
    // Benutzereingabe für Tropfenzahl
    let input = get_user_input("\nGib die Anzahl der Tropfen (Punkte) ein: ");
    
    // Validierung mit Fallback auf Standardwert
    let tropfenzahl = validate_tropfenzahl(&input).unwrap_or_else(|_| {
        println!("Verwende Standardwert von {} Tropfen.", DEFAULT_TROPFEN);
        DEFAULT_TROPFEN
    });
    
    // Bestätigung durch Benutzer
    if get_user_input("\nMöchtest du mit der Berechnung fortfahren? (Y/n): ")
        .to_lowercase()
        .starts_with('n') {
        return Err("Berechnung abgebrochen.");
    }

    // Durchführung der Berechnung mit Zeitmessung
    let threads = rayon::current_num_threads();
    let start = Instant::now();
    let pi_approx = approximiere_pi(tropfenzahl);
    let dauer = start.elapsed();

    // Rückgabe der Ergebnisse
    Ok(Ergebnis {
        pi_approx,
        dauer,
        tropfenzahl,
        threads,
    })
}

// Hauptprogramm
fn main() {
    // Ausgabe der Programminformationen
    println!("\nZusammengekleistert von Luis Weitl für den Matheunterricht mit der Programmiersprache Rust.");
    println!("Annäherung von Pi (π) durch den Monte-Carlo-Algorithmus.");
    println!("Implementiert in Rust mit der Rayon Bibliothek für die parallele Berechnung");
    println!("Der Quellcode kann hier gefunden werden: https://github.com/luigiistcrazy/pi_approximieren");
    println!("\nVerfügbare Threads: {}", rayon::current_num_threads());

    // Ausführung der Berechnung und Ausgabe der Ergebnisse
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
