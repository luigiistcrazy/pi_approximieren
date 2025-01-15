# Pi approximieren mit Hilfe des Monte-Carlo-Algorithmus

## Ausführung

### 1. Binärdatei herunterladen

[Windows](https://github.com/luigiistcrazy/pi_approximieren/releases/download/release/Pi-approximieren.exe)

[Linux](https://github.com/luigiistcrazy/pi_approximieren/releases/download/release/Pi-approximieren)

### 2. Ausführen

- Windows:
In Powershell o.ä. ausführen
```
PS> cd "C:\pfad\zum\download\"
PS> .\Pi-approximieren.exe
```
- Linux:
In beliebiger Shell ausführen
```
$ cd $HOME/Downloads/
$ chmod +x Pi-approximieren
$ ./Pi-approximieren
```

## Build it yourself

### 1. Anforderungen

- [Rust installieren](https://www.rust-lang.org/tools/install)
- [git installieren](https://github.com/git-guides/install-git) (optional, empfohlen)

### 2. Herunterladen

Mit git
```
git clone https://github.com/luigiistcrazy/pi_approximieren
cd pi_approximieren
```

### 3. Build mit cargo

```
cargo build -r
```

### 4. Ausführen

Windows
```
PS> .\target\release\pi_approximieren.exe
```

Linux
```
./target/release/pi_approximieren
```
