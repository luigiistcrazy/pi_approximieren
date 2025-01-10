use rand::Rng; // Bibliothek zur Generierung von Zufallszahlen
use rayon::prelude::*; // Bibliothek für das parallele Rechnen
use std::time::Instant; // Bibliothek für Timer
use std::io::{self, Write}; // Bibliothek für shell Interaktion

fn approximiere_pi(tropfenzahl: u64) -> f64 {
    // Erhalte die Anzahl der verfügbaren Threads
    let threads = rayon::current_num_threads() as u64;

    // Verteile die Tropfen gleichmäßig auf die Threads.
    let tropfen_per_thread = tropfenzahl / threads;

    // Nutze einen parallelen Iterator, um die Arbeit auf Threads zu verteilen.
    let innerhalb: u64 = (0..threads)
        .into_par_iter() // Erstelle einen parallelen Iterator für die Thread-Verteilung.
        .map(|_| {
            // Initialisiere einen lokalen Zufallszahlengenerator.
            let mut rng = rand::thread_rng();
            let mut thread_innerhalb = 0; // Zähler für Punkte innerhalb des Kreises.

            // Schleife, die für jeden Tropfen ausgeführt wird.
            for _ in 0..tropfen_per_thread {
                // Generiere zufällige Koordinaten (x, y) im Bereich [0, 1).
                let x: f64 = rng.gen_range(0.0..1.0);
                let y: f64 = rng.gen_range(0.0..1.0);

                // Prüfe, ob der Punkt innerhalb des Einheitskreises liegt.
                if x * x + y * y <= 1.0 {
                    thread_innerhalb += 1; // Inkrementiere den Zähler, wenn der Punkt innerhalb liegt.
                }
            }

            // Gib die Anzahl der Punkte innerhalb des Kreises für diesen Thread zurück.
            thread_innerhalb
        })
        .sum(); // Summiere die Ergebnisse aller Threads (Punkte innerhalb des Kreises).

    // Berechne die Approximation von π.
    // Formel: π ≈ 4 * (Anzahl der Punkte innerhalb des Kreises / Gesamtanzahl der Punkte).
    4.0 * (innerhalb as f64) / (tropfenzahl as f64)
}

fn main() {

    print!("\nZusammengekleistert von Luis Weitl für den Matheunterricht mit der Programmiersprache Rust. Annäherung von Pi (π) durch den Monte-Carlo-Algorithmus.\nVerwendung der Rayon Bibliothek für das parallele Rechnen (Ermöglicht die Verwendung einer sehr hohen Tropfenzahl).\nDer Quellcode kann hier gefunden werden: https://github.com/luigiistcrazy/pi_approximieren");

    // Ermittel zu verwendende Threads
    let threads = rayon::current_num_threads() as u64;
    print!("\n\nZu verwendende Threads: {}", threads);

    // Frage den Benutzer nach der Anzahl der Tropfen
    print!("\nGib die Anzahl der Tropfen (Punkte) ein: ");
    io::stdout().flush().unwrap(); // Damit die shell sofort erscheint

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    let tropfenzahl: u64 = input.trim().parse().unwrap_or(1000000); // Falls die Eingabe ungültig ist, wird auf 1 Million gesetzt
    
    // Frage den Benutzer, ob er mit der Berechnung fortfahren möchte
    print!("\nMöchtest du mit der Berechnung fortfahren? (y/n): ");
    io::stdout().flush().unwrap(); // Damit die shell sofort erscheint

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    
    if input.trim().to_lowercase() == "n" {
        println!("Berechnung abgebrochen.");
        return; // Beende das Programm, wenn der Benutzer "n" eingibt, ansonsten, fahre fort
    }

    // Starte den Timer
    let start = Instant::now();

    // Berechne die Approximation von π
    let pi_approx = approximiere_pi(tropfenzahl);

    // Stoppe den Timer und berechne die vergangene Zeit
    let dauer = start.elapsed();

    // Gib die berechnete Näherung und die Dauer der Berechnung aus
    println!("Annährung von π mit {} Tropfen: {}", tropfenzahl, pi_approx);
    println!("Berechnung abgeschlossen in {:.2?}.", dauer);
}
