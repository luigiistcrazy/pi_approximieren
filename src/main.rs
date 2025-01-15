use rayon::prelude::*;                          // Parallelisierungsbibliothek Rayon für Multithreading.
use std::io::{self, Write};                     // Eingabe-/Ausgabefunktionen mit Flush zum Schreiben in den Output-Buffer.
use std::sync::atomic::{AtomicUsize, Ordering}; // Atomare Variablen für thread-sichere Operationen.
use std::sync::Arc;                             // Atomics sind einfach teilbar zwischen Threads.
use std::time::{Duration, Instant};             // Für Zeitmessung wie für Effizienzberechnung.

/// Spinner-Anzeige, die sich kontinuierlich ändert.
struct Spinner {
    zustände: Vec<&'static str>, // Verschiedene Zustände des Spinners.
    aktuell: AtomicUsize,        // Aktueller Zustand des Spinners (atomar für Thread-Sicherheit).
}

impl Spinner {
    // Initialisiert einen neuen Spinner mit vorgegebenen Zuständen.
    fn new() -> Self {
        Spinner {
            zustände: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            aktuell: AtomicUsize::new(0), // Startet beim ersten Zustand.
        }
    }

    // Gibt den nächsten Spinner-Zustand zurück, indem wir den Index inkrementieren und mod `länge` nehmen.
    fn nächster(&self) -> &'static str {
        let aktuell = self.aktuell.fetch_add(1, Ordering::Relaxed) % self.zustände.len();
        self.zustände[aktuell]
    }
}

/// Enthält den individuellen Fortschritt eines Threads.
struct ThreadFortschritt {
    fortschritt: AtomicUsize, // Atomare Variable für Punktanzahl, die ein Thread verarbeitet hat.
}

/// Die zentrale Struktur für die Monte-Carlo-Simulation.
struct PiRechner {
    punkte_innen: AtomicUsize,                   // Punkte, die innerhalb des Einheitskreises liegen.
    punkte_gesamt: AtomicUsize,                  // Gesamtanzahl an Punkten (innerhalb und außerhalb des Kreises).
    thread_fortschritte: Vec<ThreadFortschritt>, // Pro-Thread Fortschritt.
    spinner: Spinner,                            // Spinner.
}

impl PiRechner {
    /// Konstruktor: Erstellt eine neue Instanz und initialisiert atomare Variablen.
    fn new(thread_anzahl: usize) -> Self {
        let thread_fortschritte = (0..thread_anzahl)
            .map(|_| ThreadFortschritt {
                fortschritt: AtomicUsize::new(0),
            })
            .collect();

        PiRechner {
            punkte_innen: AtomicUsize::new(0), // Am Anfang sind keine Punkte gezählt.
            punkte_gesamt: AtomicUsize::new(0),
            thread_fortschritte,
            spinner: Spinner::new(),
        }
    }

    /// Berechnet die Annäherung an π mit der Formel: π ≈ 4 * (Punkte_innen / Punkte_gesamt).
    fn berechne_pi(&self) -> f64 {
        let innen = self.punkte_innen.load(Ordering::Relaxed) as f64;
        let gesamt = self.punkte_gesamt.load(Ordering::Relaxed) as f64;
        4.0 * innen / gesamt
    }

    /// Verarbeitet einen Batch von Zufallspunkten und prüft, ob sie im Einheitskreis liegen.
    fn verarbeite_batch(&self, thread_id: usize, batch_größe: usize) {
        use rand::Rng;
        let mut rng = rand::thread_rng(); // Zufallszahlen-Generator.
        let mut lokale_treffer = 0;       // Zähler für Punkte innerhalb des Kreises.

        // Generiere `batch_größe` Punkte und prüfe, ob sie im Kreis liegen.
        for _ in 0..batch_größe {
            let x: f64 = rng.gen(); // Zufällige x-Koordinate zwischen 0 und 1.
            let y: f64 = rng.gen(); // Zufällige y-Koordinate zwischen 0 und 1.

            // Prüfe, ob (x, y) innerhalb des Kreises liegt:
            // Ein Punkt liegt innerhalb, wenn: x² + y² ≤ 1
            if x * x + y * y <= 1.0 {
                lokale_treffer += 1; // Zähle Treffer innerhalb des Kreises.
            }
        }

        // Atomar lokales Ergebnis zur globalen Trefferanzahl hinzufügen.
        self.punkte_innen
            .fetch_add(lokale_treffer, Ordering::Relaxed);
        self.punkte_gesamt.fetch_add(batch_größe, Ordering::Relaxed);

        // Aktualisiere den Fortschritt dieses Threads mit der Anzahl der verarbeiteten Punkte.
        let aktuell = self.thread_fortschritte[thread_id]
            .fortschritt
            .load(Ordering::Relaxed)
            + batch_größe;
        self.thread_fortschritte[thread_id]
            .fortschritt
            .store(aktuell, Ordering::Relaxed); // Fortschritt-Update für Balkenanpassung.
    }
}

/// Erstellt einen Balken.
fn erstelle_fortschrittsbalken(prozent: f64, breite: usize) -> String {
    // Breite des Balkens, die gefüllt sein sollte, proportional zu `prozent`.
    let gefüllte_breite = ((prozent / 100.0) * breite as f64) as usize;
    let leere_breite = breite - gefüllte_breite; // Restliche leere Breite.

    // Erstelle den Balken aus gefüllten und leeren Segmenten.
    let balken = "█".repeat(gefüllte_breite) + &"░".repeat(leere_breite);
    format!("[{}] {:.1}%", balken, prozent) // Rückgabe als String mit Prozenten.
}

/// Leert den Bildschirm und zeigt die Anfangsanzeige.
fn initialisiere_anzeige(thread_anzahl: usize) -> io::Result<()> {
    print!("\x1B[H\x1B[2J"); // ANSI-Escape-Code zum Löschen des Bildschirms.
    println!("Berechne... ⠋\n");
    println!("Threads:\n");

    // Initialer Balken für jeden Thread.
    for i in 0..thread_anzahl {
        println!("[Thread {}]: [{}] 0.0%", i, "░".repeat(50)); // Anfangsbalken.
        println!();
    }

    io::stdout().flush() // Sicherstellen, dass die Ausgabe gepusht wird.
}

/// Aktualisiert die Anzeige während der Berechnung.
fn aktualisiere_anzeige(
    rechner: &PiRechner,
    thread_anzahl: usize,
    punkte_pro_thread: usize,
) -> io::Result<()> {
    print!("\x1B[H");                                         // ANSI-Escape-Code, um den Cursor zu bewegen.
    println!("Berechne... {}\n", rechner.spinner.nächster()); // Spinner Zustand.
    println!("Threads:\n");

    // Aktualisiere für jeden Thread den Fortschritt.
    for i in 0..thread_anzahl {
        let fortschritt = rechner.thread_fortschritte[i]
            .fortschritt
            .load(Ordering::Relaxed);
        let prozent = (fortschritt as f64 / punkte_pro_thread as f64) * 100.0;
        let balken = erstelle_fortschrittsbalken(prozent.min(100.0), 50); // Fortschritt als Balken.
        println!("[Thread {}]: {}", i, balken);
        println!();
    }

    io::stdout().flush() // Flush der Ausgabe, um Aktualisierung anzuzeigen.
}


// Hilfsfunktion für Benutzereingabe
fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or_default();
    input.trim().to_string()
}

// Hauptprogramm
fn main() -> io::Result<()> {
    // Ausgabe der Programminformationen
    println!("\nZusammengekleistert von Luis Weitl für den Matheunterricht mit der Programmiersprache Rust.");
    println!("Annäherung von Pi (π) durch den Monte-Carlo-Algorithmus.");
    println!("Implementiert in Rust mit der Rayon Bibliothek für die parallele Berechnung.");
    println!("Der Quellcode kann hier gefunden werden: https://github.com/luigiistcrazy/pi_approximieren");

    // Ermittelt die Anzahl verfügbarer Threads, die für die Parallelverarbeitung verwendet werden können.
    let thread_anzahl = rayon::current_num_threads(); // Rayon überprüft die verfügbaren CPU-Kerne.
    println!("\nVerfügbare Threads im System: {}\n", thread_anzahl);

    // Eingabeaufforderung für die Anzahl der Punkte, die für die Simulation verwendet werden sollen.
    let eingabe = get_user_input("Gib die Anzahl der zu verwendenden Punkte ein: ");
    let gesamt_punkte: usize = eingabe
        .trim()                                      // Entfernt unnötige Leerzeichen aus der Eingabe.
        .parse()                                     // Parst die Eingabe in eine Ganzzahl (usize).
        .expect("Bitte gebe eine gültige Zahl ein"); // Gibt bei ungültiger Eingabe eine Fehlermeldung aus.

    // Anzahl der Punkte, die jedem Thread zugewiesen werden
    let punkte_pro_thread = gesamt_punkte / thread_anzahl;

    // Fragt den Benutzer, ob die Berechnung begonnen werden soll
    if get_user_input("\nMöchtest du mit der Berechnung fortfahren? (Y/n): ")
        .to_lowercase()   // Konvertiert die Eingabe in Kleinbuchstaben für Vergleichszwecke.
        .starts_with('n') // Prüft, ob die Eingabe mit "n" beginnt (Abbruch).
    {
        // Falls der Benutzer "n" eingibt, wird die Berechnung abgebrochen.
        println!("\nBerechnung abgebrochen.");
        return Ok(()); // Das Programm wird beendet.
    }

    // Erstellt eine neue geteilte (shared) Instanz des PiRechners, um thread-sicher Punkte zu berechnen.
    let rechner = Arc::new(PiRechner::new(thread_anzahl));

    // Startet eine Zeitmessung, um die gesamte Berechnungsdauer zu erfassen.
    let start_zeit = Instant::now();

    // ANSI-Escape-Code, um den Cursor unsichtbar zu machen.
    print!("\x1B[?25l");
    io::stdout().flush()?; // Stellt sicher, dass der Cursor Status sofort geändert wird.

    // Initialisiert die Anzeige mit einem Fortschrittsbalken für jeden Thread.
    initialisiere_anzeige(thread_anzahl)?;

    // Erstellt einen geklonten Zeiger auf den PiRechner, um ihn innerhalb des Berechnungs-Threads zu verwenden.
    let rechner_klon = Arc::clone(&rechner);

    // Startet einen Thread für die eigentliche Monte-Carlo-Berechnung.
    let berechnungs_handle = std::thread::spawn(move || {
        // Parallelisiert die Berechnung auf Basis der verfügbaren Threads.
        (0..thread_anzahl).into_par_iter().for_each(|thread_id| {
            let mut verarbeitet = 0; // Anzahl der bisher vom Thread verarbeiteten Punkte.

            // Solange der Thread weniger Punkte verarbeitet hat als ihm zugewiesen wurden:
            while verarbeitet < punkte_pro_thread {
                // Bestimmt die Anzahl verbleibender Punkte für diesen Batch.
                let verbleibend = punkte_pro_thread - verarbeitet;
                let batch_größe = 10_000.min(verbleibend); // Maximale Batch-Größe ist 10.000 Punkte.

                // Ruft die Batch-Verarbeitung des Monte-Carlo-Algorithmus auf.
                rechner_klon.verarbeite_batch(thread_id, batch_größe);

                // Erhöht den Zähler für verarbeitete Punkte.
                verarbeitet += batch_größe;
            }
        });
    });

    // Startet einen weiteren Thread, um die Fortschrittsanzeige zu aktualisieren, während die Berechnung läuft.
    let rechner_klon = Arc::clone(&rechner);
    std::thread::spawn(move || {
        // Solange die Gesamtanzahl an Punkten noch nicht erreicht ist:
        while rechner_klon.punkte_gesamt.load(Ordering::SeqCst) < gesamt_punkte {
            // Aktualisiere die Fortschrittsanzeige.
            if let Err(e) = aktualisiere_anzeige(&rechner_klon, thread_anzahl, punkte_pro_thread) {
                // Gib eine Fehlermeldung aus, falls ein Fehler beim Aktualisieren der Anzeige auftritt.
                eprintln!("Fehler beim Aktualisieren der Anzeige: {}", e);
            }

            // Warte 50 Millisekunden, bevor die Anzeige erneut aktualisiert wird.
            std::thread::sleep(Duration::from_millis(50));
        }
        
    });

    // Wartet auf den Abschluss des Berechnungs-Threads.
    match berechnungs_handle.join() {
        Ok(_) => println!("Berechnungs-Thread abgeschlossen"), // Erfolgreicher Abschluss der Berechnung.
        Err(e) => eprintln!("Berechnungs-Thread ist mit einem Fehler beendet: {:?}", e), // Fehlermeldung.
    }

    // Aktualisiere die Fortschrittsanzeige ein letztes Mal nach Abschluss der Berechnung.
    aktualisiere_anzeige(&rechner, thread_anzahl, punkte_pro_thread)?;

    // ANSI-Escape-Code, um den Cursor wieder sichtbar zu machen.
    print!("\x1B[?25h");

    // Berechnet die Gesamtzeit der Simulation.
    let dauer = start_zeit.elapsed();

    // Führt die Monte-Carlo-Berechnung durch, um π zu approximieren:
    let pi_approximation = rechner.berechne_pi();

    // Lädt die Gesamtanzahl der verarbeiteten Punkte, um dies in den Ergebnissen anzuzeigen.
    let gesamt_punkte = rechner.punkte_gesamt.load(Ordering::Relaxed);

    // Gibt die berechneten Ergebnisse an die Konsole aus:
    println!("\nErgebnisse:");
    println!("π Annäherung:  {:.10}", pi_approximation);      // Die approximierte Annäherung von π.
    println!("Eigentliches π: {:.10}", std::f64::consts::PI); // Der tatsächliche Wert von π aus den Rust-Konstanten.
    println!(
        "Abweichung:      {:.10}",
        (pi_approximation - std::f64::consts::PI).abs()       // Die Differenz zwischen berechnetem und tatsächlichem π.
    );
    println!("Verwendete Punkte:    {}", gesamt_punkte);      // Gesamtanzahl der verarbeiteten Punkte.
    println!("Berechnungszeit: {:.2?}", dauer);               // Gesamtdauer der Berechnung.
    println!(
        "Punkte pro Sekunde:  {:.2e}",                        // Berechnete Leistung basierend auf Punkten pro Sekunde.
        gesamt_punkte as f64 / dauer.as_secs_f64()
    );

    Ok(()) // Beendet das Programm.
}
