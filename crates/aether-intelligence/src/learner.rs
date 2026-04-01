//! # Pattern Learner - Estrae Convenzioni da Codebase Esistente
//!
//! Questo modulo implementa l'analisi automatica delle convenzioni di codice
//! da un progetto esistente, permettendo ad Aether di adattarsi allo stile del progetto.
//!
//! ## Panoramica
//!
//! Il `PatternLearner` analizza il codice sorgente per estrarre:
//!
//! - **Convenzioni di naming**: Suffissi comuni per struct/enum (Config, Builder, Error, ecc.)
//! - **Derive patterns**: Combinazioni frequenti di trait derivati (Debug, Clone, PartialEq)
//! - **Stile documentazione**: Percentuale di copertura, formato commenti (/// vs //!)
//! - **Pattern di importazione**: Crate comuni, uso di wildcard imports
//!
//! ## Output
//!
//! Produce un file TOML (`.aether/learned.toml`) con tutte le convenzioni scoperte:
//!
//! ```toml
//! project = "my-project"
//! language = "rust"
//! files_analyzed = 42
//!
//! [naming]
//! struct_suffixes = { Config = 15, Builder = 8, Error = 5 }
//! function_prefixes = { get_ = 25, set_ = 12, is_ = 8 }
//!
//! [derives]
//! debug_percentage = 95.0
//! clone_percentage = 88.0
//! common_combinations = { "Debug,Clone" = 45 }
//!
//! [confidence]
//! naming = 0.85
//! derives = 0.92
//! documentation = 0.78
//! ```
//!
//! ## Esempio di Utilizzo
//!
//! ```rust,ignore
//! use aether_intelligence::learner::PatternLearner;
//!
//! // Crea un learner per il progetto
//! let mut learner = PatternLearner::new("my-project");
//!
//! // Analizza uno o più file sorgente
//! let source = r#"
//!     /// Configuration for the application
//!     #[derive(Debug, Clone, PartialEq)]
//!     pub struct AppConfig {
//!         name: String,
//!     }
//!
//!     pub fn get_config() -> AppConfig {
//!         AppConfig { name: "default".into() }
//!     }
//! "#;
//!
//! learner.analyze_file(source)?;
//!
//! // Finalizza l'analisi e ottieni i pattern
//! let patterns = learner.finalize();
//!
//! // Esporta in formato TOML
//! let toml = learner.to_toml()?;
//! println!("{}", toml);
//! ```
//!
//! ## Integrazione con Aether
//!
//! Questo modulo è utilizzato dal comando `aether learn` per personalizzare
//! la validazione in base alle convenzioni specifiche del progetto.
//! I pattern estratti possono essere usati per:
//!
//! - Generare regole di validazione personalizzate
//! - Suggerire naming conventions durante lo sviluppo
//! - Identificare deviazioni dallo stile del progetto
//!
//! ## Confidence Scores
//!
//! I punteggi di confidenza (0.0 - 1.0) indicano l'affidabilità dei pattern estratti:
//!
//! - **Naming**: Basato su numero di campioni struct/enum (minimo 5 per confidenza piena)
//! - **Derives**: Basato su numero di derive analizzati (minimo 10 per confidenza piena)
//! - **Documentation**: Basato su numero di file analizzati (massimo a ~20 file)
//!
//! ## Limitazioni Attuali
//!
//! - **Parsing regex-based**: Usa regex invece di tree-sitter (può avere falsi positivi)
//! - **Solo Rust**: Attualmente supporta solo il linguaggio Rust
//! - **Nessuna cache**: Ogni esecuzione analizza da zero
//!
//! Vedi anche: [`LearnedPatterns`], [`NamingPatterns`], [`DerivePatterns`], [`PatternLearner`]

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Convenzioni apprese dall'analisi di un codebase.
///
/// Questo struct è il contenitore principale per tutti i pattern estratti
/// durante l'analisi. Viene serializzato in TOML e salvato in `.aether/learned.toml`.
///
/// ## Campi Principali
///
/// - `project`: Identificatore del progetto analizzato
/// - `language`: Linguaggio rilevato (attualmente solo "rust")
/// - `files_analyzed`: Numero di file processati
/// - `naming`: Convenzioni di naming scoperte
/// - `derives`: Pattern di derive traits
/// - `documentation`: Pattern di documentazione
/// - `confidence`: Punteggi di affidabilità per ogni categoria
///
/// ## Esempio di Output TOML
///
/// ```toml
/// project = "my-rust-project"
/// language = "rust"
/// analyzed_at = "2024-01-15T10:30:00Z"
/// files_analyzed = 25
///
/// [naming]
/// struct_suffixes = { Config = 12, Builder = 8 }
///
/// [confidence]
/// naming = 0.85
/// derives = 0.92
/// ```
///
/// ## Utilizzo
///
/// ```rust,ignore
/// use aether_intelligence::learner::{PatternLearner, LearnedPatterns};
///
/// let mut learner = PatternLearner::new("my-project");
/// learner.analyze_file(source)?;
/// let patterns: LearnedPatterns = learner.finalize();
///
/// // Verifica confidence prima di usare i pattern
/// if patterns.confidence.naming > 0.7 {
///     println!("Naming patterns affidabili: {:?}", patterns.naming.struct_suffixes);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LearnedPatterns {
    /// Nome o identificatore del progetto analizzato.
    ///
    /// Usato per identificare la provenienza dei pattern
    /// quando vengono condivisi tra progetti o esportati come preset.
    pub project: String,
    
    /// Linguaggio di programmazione rilevato.
    ///
    /// Attualmente supportato solo "rust".
    /// Futuro: "python", "typescript", "go", ecc.
    pub language: String,
    
    /// Timestamp dell'analisi in formato ISO 8601/RFC 3339.
    ///
    /// Generato automaticamente alla creazione del learner.
    /// Utile per determinare se i pattern necessitano di aggiornamento.
    pub analyzed_at: String,
    
    /// Numero totale di file sorgente analizzati.
    ///
    /// Influenza i confidence scores: più file = maggiore affidabilità.
    /// Il learner raggiunge confidenza massima a ~20 file.
    pub files_analyzed: usize,
    
    /// Convenzioni di naming scoperte.
    ///
    /// Include suffissi di struct/enum, prefissi di funzioni,
    /// e stile di naming delle variabili.
    pub naming: NamingPatterns,
    
    /// Pattern di derive traits (specifico per Rust).
    ///
    /// Traccia combinazioni comuni di derive e percentuali
    /// di utilizzo per Debug, Clone, Default.
    pub derives: DerivePatterns,
    
    /// Pattern di documentazione.
    ///
    /// Include percentuale di elementi pubblici documentati,
    /// stile di commento preferito, e lunghezza media.
    pub documentation: DocPatterns,
    
    /// Pattern di importazione moduli.
    ///
    /// Traccia crate esterne comuni, stile di raggruppamento,
    /// e uso di wildcard imports.
    pub imports: ImportPatterns,
    
    /// Punteggi di confidenza per ogni categoria (0.0 - 1.0).
    ///
    /// Indica quanto sono affidabili i pattern estratti.
    /// Valori bassi suggeriscono di raccogliere più campioni.
    pub confidence: ConfidenceScores,
}

/// Convenzioni di naming scoperte dall'analisi.
///
/// Traccia i pattern di denominazione utilizzati nel progetto,
/// permettendo di identificare lo stile preferito dai sviluppatori.
///
/// ## Pattern Tracciati
///
/// | Categoria | Esempi |
/// |-----------|--------|
/// | Struct suffixes | Config, Builder, Error, Handler, Manager |
/// | Enum suffixes | Kind, Type, Mode, State, Action |
/// | Function prefixes | get_, set_, is_, has_, try_, build_ |
/// | Variable style | snake_case (Rust), camelCase |
///
/// ## Esempio di Output
///
/// ```toml
/// [naming]
/// struct_suffixes = { Config = 15, Builder = 8, Error = 5 }
/// enum_suffixes = { Kind = 10, Type = 7 }
/// function_prefixes = { get_ = 25, set_ = 12, is_ = 8 }
/// variable_style = "snake_case"
/// private_field_prefix = "_"
/// ```
///
/// ## Utilizzo nei Controlli
///
/// Quando un nuovo struct viene creato, Aether può suggerire
/// suffissi basati su questi pattern:
///
/// ```rust
/// // Se "Config" è il suffisso più comune
/// // Aether suggerisce: AppConfig, DatabaseConfig, ecc.
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NamingPatterns {
    /// Suffissi comuni nei nomi di struct con relativa frequenza.
    ///
    /// Esempio: `{"Config": 15, "Builder": 8, "Error": 5}`
    ///
    /// I suffissi cercati includono: Config, Builder, Error, Result,
    /// Context, State, Options, Settings, Info, Data, Handler, Manager.
    pub struct_suffixes: HashMap<String, usize>,
    
    /// Suffissi comuni nei nomi di enum con relativa frequenza.
    ///
    /// Esempio: `{"Kind": 10, "Type": 7, "Mode": 5}`
    ///
    /// I suffissi cercati includono: Kind, Type, Mode, State,
    /// Error, Result, Action, Event.
    pub enum_suffixes: HashMap<String, usize>,
    
    /// Prefissi comuni nei nomi di funzioni con relativa frequenza.
    ///
    /// Esempio: `{"get_": 25, "set_": 12, "is_": 8}`
    ///
    /// I prefissi cercati includono: get_, set_, is_, has_, can_,
    /// should_, try_, parse_, validate_, build_, create_, from_, into_, as_.
    pub function_prefixes: HashMap<String, usize>,
    
    /// Stile di naming delle variabili preferito.
    ///
    /// Valori possibili: "snake_case" (standard Rust), "camelCase"
    ///
    /// Attualmente non calcolato automaticamente - lasciato come None.
    pub variable_style: Option<String>,
    
    /// Prefisso usato per campi privati.
    ///
    /// Esempi: `"m_"` (C++ style), `"_"` (Python style), `None` (Rust standard)
    ///
    /// Attualmente non calcolato automaticamente.
    pub private_field_prefix: Option<String>,
}

/// Pattern di derive traits scoperti dall'analisi.
///
/// Traccia le combinazioni di derive più utilizzate nel progetto,
/// permettendo di identificare le convenzioni di derivazione.
///
/// ## Derive Tracciati
///
/// I trait più comuni monitorati:
/// - **Debug**: Per debugging e logging
/// - **Clone**: Per clonazione esplicita
/// - **PartialEq**: Per confronti di uguaglianza
/// - **Default**: Per valori di default
///
/// ## Esempio di Output
///
/// ```toml
/// [derives]
/// common_combinations = { "Debug,Clone" = 45, "Debug,Clone,PartialEq" = 30 }
/// debug_percentage = 95.0
/// clone_percentage = 88.0
/// default_percentage = 42.0
/// ```
///
/// ## Utilizzo nei Controlli
///
/// Se il progetto usa `Debug,Clone` nel 90% degli struct,
/// Aether può avvertire quando un nuovo struct non li deriva.
///
/// ```rust
/// // Avvertimento se Debug non è derivato ma è usato nel 95% dei casi
/// struct MyStruct { /* ... */ } // Warning: Debug non derivato
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DerivePatterns {
    /// Combinazioni comuni di derive con relativa frequenza.
    ///
    /// Le combinazioni sono normalizzate in ordine alfabetico.
    ///
    /// Esempio: `{"Clone,Debug": 45, "Clone,Debug,PartialEq": 30}`
    ///
    /// Nota: Le trait sono ordinate alfabeticamente per evitare duplicati
    /// (es. "Debug,Clone" diventa "Clone,Debug").
    pub common_combinations: HashMap<String, usize>,
    
    /// Percentuale di struct con Debug derivato (0.0 - 100.0).
    ///
    /// Valore alto (> 80%) suggerisce che Debug è una convenzione del progetto.
    /// Aether può avvertire se un nuovo struct pubblico non lo deriva.
    pub debug_percentage: f64,
    
    /// Percentuale di struct con Clone derivato (0.0 - 100.0).
    ///
    /// Utile per identificare se la clonazione è comune nel progetto.
    pub clone_percentage: f64,
    
    /// Percentuale di struct con Default derivato (0.0 - 100.0).
    ///
    /// Utile per suggerire l'uso di Default::default() come convenzione.
    pub default_percentage: f64,
}

/// Pattern di documentazione scoperti dall'analisi.
///
/// Traccia le convenzioni di documentazione del progetto,
/// permettendo di valutare la qualità della documentazione.
///
/// ## Metriche Tracciate
///
/// - **Percentuale di documentazione**: Quanti elementi pubblici sono documentati
/// - **Stile di commento**: Formato preferito (/// vs //! vs /* */)
/// - **Lunghezza media**: Righe di documentazione per elemento
///
/// ## Esempio di Output
///
/// ```toml
/// [documentation]
/// public_doc_percentage = 72.0
/// comment_style = "///"
/// avg_doc_length = 3.5
/// ```
///
/// ## Utilizzo nei Controlli
///
/// Se `public_doc_percentage` è alto (> 80%), Aether può avvertire
/// quando un nuovo elemento pubblico non è documentato.
///
/// ```rust
/// // Avvertimento se la percentuale è alta
/// pub fn new_function() {} // Warning: elemento pubblico non documentato
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocPatterns {
    /// Percentuale di elementi pubblici con documentazione (0.0 - 100.0).
    ///
    /// Calcolata come: (elementi pubblici documentati / totali) * 100
    ///
    /// Valore alto indica un progetto ben documentato.
    /// Utile per enforce della documentazione.
    pub public_doc_percentage: f64,
    
    /// Stile di commento preferito per la documentazione.
    ///
    /// Valori possibili:
    /// - `"///"` - Line-by-line (standard Rust)
    /// - `"//!" - Inner doc comments (per moduli/crates)
    /// - `"/* */"` - Block comments (meno comune in Rust)
    ///
    /// Attualmente non calcolato automaticamente.
    pub comment_style: Option<String>,
    
    /// Lunghezza media della documentazione in righe.
    ///
    /// Calcolata come: righe totali di doc / elementi documentati
    ///
    /// Utile per identificare se il progetto preferisce
    /// documentazione concisa (1-2 righe) o estesa (5+ righe).
    pub avg_doc_length: f64,
}

/// Pattern di importazione moduli scoperti dall'analisi.
///
/// Traccia le convenzioni di import del progetto,
/// permettendo di identificare dipendenze comuni e stili.
///
/// ## Metriche Tracciate
///
/// - **Stile di raggruppamento**: Come sono organizzati gli import
/// - **Crate comuni**: Dipendenze esterne più usate
/// - **Wildcard usage**: Frequenza di `use ...::*;`
///
/// ## Esempio di Output
///
/// ```toml
/// [imports]
/// grouping_style = "by_crate"
/// common_crates = { serde = 45, tokio = 38, anyhow = 25 }
/// wildcard_usage_percentage = 5.0
/// ```
///
/// ## Utilizzo nei Controlli
///
/// Se `wildcard_usage_percentage` è basso (< 10%), Aether può
/// avvertire quando viene usato un import wildcard.
///
/// ```rust,ignore
/// // Avvertimento se il progetto evita wildcards
/// use crate::module::*; // Warning: import wildcard sconsigliato
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportPatterns {
    /// Stile di raggruppamento degli import.
    ///
    /// Valori possibili:
    /// - `"alphabetical"` - Import ordinati alfabeticamente
    /// - `"by_crate"` - Raggruppati per crate (std, esterni, interni)
    /// - `"custom"` - Altro stile non standard
    ///
    /// Attualmente non calcolato automaticamente.
    pub grouping_style: Option<String>,
    
    /// Crate esterne più utilizzate con relativa frequenza.
    ///
    /// Esempio: `{serde = 45, tokio = 38, anyhow = 25}`
    ///
    /// Utile per identificare le dipendenze principali del progetto
    /// e suggerire import appropriati.
    ///
    /// Attualmente non calcolato automaticamente.
    pub common_crates: HashMap<String, usize>,
    
    /// Percentuale di uso di import wildcard (0.0 - 100.0).
    ///
    /// Calcolata come: (wildcard imports / totali) * 100
    ///
    /// Valore basso (< 10%) suggerisce che il progetto preferisce
    /// import espliciti. Aether può avvertire sui wildcard.
    pub wildcard_usage_percentage: f64,
}

/// Punteggi di confidenza per i pattern estratti.
///
/// Indica quanto sono affidabili i pattern scoperti,
/// basati sulla dimensione del campione analizzato.
///
/// ## Interpretazione dei Valori
///
/// | Range | Significato |
/// |-------|-------------|
/// | 0.0 - 0.3 | Bassa confidenza - campioni insufficienti |
/// | 0.3 - 0.7 | Media confidenza - risultati parziali |
/// | 0.7 - 1.0 | Alta confidenza - pattern affidabili |
///
/// ## Fattori che Influenzano la Confidenza
///
/// - **Naming**: Almeno 5 struct/enum con suffissi riconosciuti
/// - **Derives**: Almeno 10 derive statements analizzati
/// - **Documentation**: Almeno 20 file analizzati per massima confidenza
///
/// ## Esempio di Output
///
/// ```toml
/// [confidence]
/// naming = 0.85
/// derives = 0.92
/// documentation = 0.45
/// ```
///
/// ## Utilizzo
///
/// Prima di usare i pattern per validazione o suggerimenti,
/// verificare che la confidenza sia sufficiente (> 0.7):
///
/// ```rust,ignore
/// let patterns = learner.finalize();
///
/// if patterns.confidence.naming > 0.7 {
///     // Sicuro usare naming patterns per validazione
///     for (suffix, count) in &patterns.naming.struct_suffixes {
///         println!("Suffix {}: {} occurrences", suffix, count);
///     }
/// } else {
///     println!("Warning: naming patterns non affidabili, analizzare più file");
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfidenceScores {
    /// Confidenza nei pattern di naming (0.0 - 1.0).
    ///
    /// Formula: `min(samples / 5.0, 1.0) * min(files / 20.0, 1.0)`
    ///
    /// Dove `samples` è il numero di struct/enum con suffissi riconosciuti.
    pub naming: f64,
    
    /// Confidenza nei pattern di derive (0.0 - 1.0).
    ///
    /// Formula: `min(samples / 10.0, 1.0) * min(files / 20.0, 1.0)`
    ///
    /// Dove `samples` è il numero di derive statements analizzati.
    pub derives: f64,
    
    /// Confidenza nei pattern di documentazione (0.0 - 1.0).
    ///
    /// Formula: `min(files / 20.0, 1.0)`
    ///
    /// Basata solo sul numero di file analizzati.
    pub documentation: f64,
}

/// Analizzatore di pattern per codebase Rust.
///
/// Il learner principale che analizza il codice sorgente per estrarre
/// convenzioni e pattern utilizzati nel progetto.
///
/// ## Workflow
///
/// 1. **Creazione**: `PatternLearner::new("project-name")`
/// 2. **Analisi**: `learner.analyze_file(source)` per ogni file
/// 3. **Finalizzazione**: `learner.finalize()` per ottenere i pattern
/// 4. **Esportazione**: `learner.to_toml()` per salvare in formato TOML
///
/// ## Esempio Completo
///
/// ```rust,ignore
/// use aether_intelligence::learner::PatternLearner;
///
/// // Crea il learner
/// let mut learner = PatternLearner::new("my-rust-project");
///
/// // Analizza più file
/// let files = vec![
///     include_str!("config.rs"),
///     include_str!("builder.rs"),
///     include_str!("error.rs"),
/// ];
///
/// for source in files {
///     learner.analyze_file(source)?;
/// }
///
/// // Finalizza e ottieni i pattern
/// let patterns = learner.finalize();
///
/// // Verifica confidenza
/// if patterns.confidence.naming > 0.7 {
///     println!("Pattern affidabili estratti!");
///     println!("Struct suffixes: {:?}", patterns.naming.struct_suffixes);
/// }
///
/// // Esporta per uso futuro
/// let toml = learner.to_toml()?;
/// std::fs::write(".aether/learned.toml", toml)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ## Thread Safety
///
/// `PatternLearner` non è thread-safe. Per analizzare file in parallelo,
/// creare un learner per ogni thread e mergiare i risultati manualmente.
///
/// ## Integrazione CLI
///
/// Usato dal comando `aether learn`:
///
/// ```bash
/// aether learn --project ./my-project --output .aether/learned.toml
/// ```
pub struct PatternLearner {
    /// Statistiche accumulate durante l'analisi.
    stats: LearnedPatterns,
}

impl PatternLearner {
    /// Crea un nuovo learner per il progetto specificato.
    ///
    /// Inizializza le statistiche vuote e imposta il timestamp di analisi.
    ///
    /// # Argomenti
    ///
    /// * `project` - Nome o identificatore del progetto da analizzare
    ///
    /// # Returns
    ///
    /// Un nuovo `PatternLearner` pronto per l'analisi.
    ///
    /// # Esempio
    ///
    /// ```rust
    /// use aether_intelligence::learner::PatternLearner;
    ///
    /// let learner = PatternLearner::new("my-awesome-project");
    /// // Timestamp impostato automaticamente
    /// // Language = "rust" di default
    /// ```
    ///
    /// # Nota
    ///
    /// Attualmente supporta solo Rust. In futuro sarà possibile
    /// specificare il linguaggio: `PatternLearner::new("proj", Lang::Python)`.
    pub fn new(project: &str) -> Self {
        Self {
            stats: LearnedPatterns {
                project: project.to_string(),
                language: "rust".to_string(),
                analyzed_at: chrono::Utc::now().to_rfc3339(),
                ..Default::default()
            },
        }
    }
    
    /// Analizza un singolo file sorgente Rust.
    ///
    /// Estrae tutti i pattern dal codice e aggiorna le statistiche accumulate.
    /// Può essere chiamato più volte per analizzare un intero progetto.
    ///
    /// # Argomenti
    ///
    /// * `source` - Codice sorgente Rust come stringa
    ///
    /// # Returns
    ///
    /// `Ok(())` se l'analisi è riuscita, `Err(Error)` in caso di errore.
    ///
    /// # Pattern Estratti
    ///
    /// Per ogni file vengono analizzati:
    /// - Struct e enum con i loro suffissi
    /// - Derive attributes e combinazioni
    /// - Documentazione di elementi pubblici
    /// - Funzioni e loro prefissi comuni
    ///
    /// # Esempio
    ///
    /// ```rust,ignore
    /// let mut learner = PatternLearner::new("my-project");
    ///
    /// let source = r#"
    ///     /// Configuration struct
    ///     #[derive(Debug, Clone)]
    ///     pub struct AppConfig {
    ///         name: String,
    ///     }
    ///
    ///     pub fn get_config() -> AppConfig {
    ///         AppConfig { name: "default".into() }
    ///     }
    /// "#;
    ///
    /// learner.analyze_file(source)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errori
    ///
    /// Attualmente non restituisce errori, ma il tipo `Result` è mantenuto
    /// per future estensioni (es. parsing con tree-sitter).
    ///
    /// # Performance
    ///
    /// - O(n) dove n è la lunghezza del codice sorgente
    /// - Ogni chiamata aggiorna le statistiche in modo incrementale
    /// - Le percentuali sono calcolate come running average
    pub fn analyze_file(&mut self, source: &str) -> Result<()> {
        // Extract struct definitions
        self.extract_struct_patterns(source);
        
        // Extract derive patterns
        self.extract_derive_patterns(source);
        
        // Extract documentation patterns
        self.extract_doc_patterns(source);
        
        // Extract function patterns
        self.extract_function_patterns(source);
        
        self.stats.files_analyzed += 1;
        
        Ok(())
    }
    
    /// Estrae pattern di naming da struct ed enum.
    ///
    /// # Pattern Cercati
    ///
    /// **Struct suffixes**: Config, Builder, Error, Result, Context, State,
    /// Options, Settings, Info, Data, Handler, Manager
    ///
    /// **Enum suffixes**: Kind, Type, Mode, State, Error, Result, Action, Event
    ///
    /// # Implementazione
    ///
    /// Usa regex-based matching. In futuro sarà sostituito con tree-sitter
    /// per parsing più accurato e supporto macro complesse.
    ///
    /// # Limitazioni
    ///
    /// - Non riconosce struct in macro (es. `bitflags!`, `derive_builder!`)
    /// - Può avere falsi positivi in stringhe letterali
    /// - Non distingue tra struct pubblici e privati
    fn extract_struct_patterns(&mut self, source: &str) {
        // Simple regex-based extraction (tree-sitter would be better but this is prototype)
        let struct_pattern = regex::Regex::new(r"struct\s+([A-Z][a-zA-Z0-9]*)").unwrap();
        
        for cap in struct_pattern.captures_iter(source) {
            if let Some(name) = cap.get(1) {
                let name = name.as_str();
                
                // Extract suffix (last capitalized word or common suffixes)
                for suffix in &["Config", "Builder", "Error", "Result", "Context", "State", "Options", "Settings", "Info", "Data", "Handler", "Manager"] {
                    if name.ends_with(suffix) {
                        *self.stats.naming.struct_suffixes.entry(suffix.to_string()).or_insert(0) += 1;
                        break;
                    }
                }
            }
        }
        
        // Extract enum definitions
        let enum_pattern = regex::Regex::new(r"enum\s+([A-Z][a-zA-Z0-9]*)").unwrap();
        for cap in enum_pattern.captures_iter(source) {
            if let Some(name) = cap.get(1) {
                let name = name.as_str();
                for suffix in &["Kind", "Type", "Mode", "State", "Error", "Result", "Action", "Event"] {
                    if name.ends_with(suffix) {
                        *self.stats.naming.enum_suffixes.entry(suffix.to_string()).or_insert(0) += 1;
                        break;
                    }
                }
            }
        }
    }
    
    /// Estrae pattern di derive traits dal codice.
    ///
    /// # Pattern Cercati
    ///
    /// Analizza `#[derive(...)]` attributes per identificare:
    /// - Combinazioni comuni di traits (es. Debug+Clone)
    /// - Percentuale di utilizzo di Debug, Clone, Default
    ///
    /// # Normalizzazione
    ///
    /// Le combinazioni sono normalizzate in ordine alfabetico
    /// per evitare duplicati (es. "Debug,Clone" e "Clone,Debug"
    /// sono considerati uguali e diventano "Clone,Debug").
    ///
    /// # Running Average
    ///
    /// Le percentuali sono calcolate come running average su tutti
    /// i file analizzati, permettendo di aggiornare le statistiche
    /// incrementalmente.
    ///
    /// # Limitazioni
    ///
    /// - Non riconosce derive da macro (es. `#[derive(Debug)]` in `macro_rules!`)
    /// - Non distingue derive su struct vs enum
    fn extract_derive_patterns(&mut self, source: &str) {
        // Match #[derive(...)]
        let derive_pattern = regex::Regex::new(r"#\[derive\(([^)]+)\)]").unwrap();
        
        let mut total_structs = 0usize;
        let mut structs_with_debug = 0usize;
        let mut structs_with_clone = 0usize;
        let mut structs_with_default = 0usize;
        
        for cap in derive_pattern.captures_iter(source) {
            if let Some(derives) = cap.get(1) {
                let derives_str = derives.as_str();
                
                // Normalize derive order for combination tracking
                let mut traits: Vec<&str> = derives_str.split(',').map(|s| s.trim()).collect();
                traits.sort();
                let combination = traits.join(",");
                
                *self.stats.derives.common_combinations.entry(combination).or_insert(0) += 1;
                
                total_structs += 1;
                if traits.contains(&"Debug") {
                    structs_with_debug += 1;
                }
                if traits.contains(&"Clone") {
                    structs_with_clone += 1;
                }
                if traits.contains(&"Default") {
                    structs_with_default += 1;
                }
            }
        }
        
        // Update percentages (running average)
        if total_structs > 0 {
            let n = self.stats.files_analyzed as f64;
            let new_debug = (structs_with_debug as f64 / total_structs as f64) * 100.0;
            let new_clone = (structs_with_clone as f64 / total_structs as f64) * 100.0;
            let new_default = (structs_with_default as f64 / total_structs as f64) * 100.0;
            
            self.stats.derives.debug_percentage = 
                (self.stats.derives.debug_percentage * n + new_debug) / (n + 1.0);
            self.stats.derives.clone_percentage = 
                (self.stats.derives.clone_percentage * n + new_clone) / (n + 1.0);
            self.stats.derives.default_percentage = 
                (self.stats.derives.default_percentage * n + new_default) / (n + 1.0);
        }
    }
    
    /// Estrae pattern di documentazione dal codice.
    ///
    /// # Pattern Cercati
    ///
    /// - Percentuale di elementi pubblici documentati
    /// - Presenza di `///` commenti prima di elementi `pub`
    ///
    /// # Metriche
    ///
    /// Calcola la percentuale di elementi pubblici (fn, struct, enum, trait,
    /// const, static) che hanno documentazione (/// commento immediatamente prima).
    ///
    /// # Limitazioni
    ///
    /// - Richiede che il doc comment sia sulla riga immediatamente prima
    /// - Non riconosce doc comments inline o dopo l'elemento
    /// - Non distingue tra /// e //! (assumono tutti come ///)
    fn extract_doc_patterns(&mut self, source: &str) {
        // Count public items
        let pub_pattern = regex::Regex::new(r"pub\s+(fn|struct|enum|trait|const|static)\s+").unwrap();
        let total_public = pub_pattern.find_iter(source).count();
        
        // Count documented public items (/// or //! before pub)
        let doc_pattern = regex::Regex::new(r"(?m)^\s*///[^\n]*\n\s*pub").unwrap();
        let documented = doc_pattern.find_iter(source).count();
        
        if total_public > 0 {
            let new_percentage = (documented as f64 / total_public as f64) * 100.0;
            let n = self.stats.files_analyzed as f64;
            
            self.stats.documentation.public_doc_percentage = 
                (self.stats.documentation.public_doc_percentage * n + new_percentage) / (n + 1.0);
        }
    }
    
    /// Estrae pattern di naming delle funzioni.
    ///
    /// # Pattern Cercati
    ///
    /// Prefissi comuni: get_, set_, is_, has_, can_, should_, try_,
    /// parse_, validate_, build_, create_, from_, into_, as_
    ///
    /// # Convenzioni Rust
    ///
    /// Questi prefissi seguono le convenzioni idiomatiche Rust:
    /// - `get_`, `set_` per accessors
    /// - `is_`, `has_`, `can_`, `should_` per predicates (restituiscono bool)
    /// - `try_` per operazioni che possono fallire (restituiscono Result)
    /// - `from_`, `into_`, `as_` per conversioni
    /// - `build_`, `create_` per factory methods
    ///
    /// # Limitazioni
    ///
    /// - Non distingue tra metodi e funzioni standalone
    /// - Non riconosce funzioni in macro
    fn extract_function_patterns(&mut self, source: &str) {
        // Extract function names to find common prefixes
        let fn_pattern = regex::Regex::new(r"fn\s+([a-z_][a-z0-9_]*)").unwrap();
        
        for cap in fn_pattern.captures_iter(source) {
            if let Some(name) = cap.get(1) {
                let name = name.as_str();
                
                // Check for common prefixes
                for prefix in &["get_", "set_", "is_", "has_", "can_", "should_", "try_", "parse_", "validate_", "build_", "create_", "from_", "into_", "as_"] {
                    if name.starts_with(prefix) {
                        *self.stats.naming.function_prefixes.entry(prefix.to_string()).or_insert(0) += 1;
                        break;
                    }
                }
            }
        }
    }
    
    /// Finalizza l'analisi e calcola i punteggi di confidenza.
    ///
    /// Questo metodo consuma il learner e restituisce i pattern finali.
    /// Deve essere chiamato dopo aver analizzato tutti i file desiderati.
    ///
    /// # Returns
    ///
    /// `LearnedPatterns` con tutti i pattern estratti e confidence scores calcolati.
    ///
    /// # Algoritmo di Confidenza
    ///
    /// La confidenza è calcolata in base alla dimensione del campione:
    ///
    /// ```rust,ignore
    /// base_confidence = min(files_analyzed / 20.0, 1.0)
    ///
    /// naming_confidence = if naming_samples >= 5 { base_confidence }
    ///                     else { naming_samples / 5.0 }
    ///
    /// derives_confidence = if derive_samples >= 10 { base_confidence }
    ///                      else { derive_samples / 10.0 }
    ///
    /// doc_confidence = base_confidence
    /// ```
    ///
    /// # Esempio
    ///
    /// ```rust,ignore
    /// let mut learner = PatternLearner::new("my-project");
    ///
    /// for file in source_files {
    ///     learner.analyze_file(&file)?;
    /// }
    ///
    /// let patterns = learner.finalize(); // Consuma il learner
    ///
    /// println!("Confidenza naming: {:.2}", patterns.confidence.naming);
    /// println!("Pattern trovati: {:?}", patterns.naming.struct_suffixes);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Nota
    ///
    /// Dopo `finalize()`, il learner non può più essere usato.
    /// Per ottenere statistiche senza consumare il learner, usa `stats()`.
    pub fn finalize(mut self) -> LearnedPatterns {
        let files = self.stats.files_analyzed as f64;
        
        // Confidence based on sample size
        // More files = more confidence, up to ~20 files for 1.0
        let base_confidence = (files / 20.0).min(1.0);
        
        // Naming confidence: need at least 5 struct samples
        let naming_samples = self.stats.naming.struct_suffixes.values().sum::<usize>() as f64;
        self.stats.confidence.naming = if naming_samples >= 5.0 { base_confidence } else { naming_samples / 5.0 };
        
        // Derive confidence: need at least 10 derive samples
        let derive_samples = self.stats.derives.common_combinations.values().sum::<usize>() as f64;
        self.stats.confidence.derives = if derive_samples >= 10.0 { base_confidence } else { derive_samples / 10.0 };
        
        // Doc confidence: based on files analyzed
        self.stats.confidence.documentation = base_confidence;
        
        self.stats
    }
    
    /// Ottiene le statistiche correnti senza consumare il learner.
    ///
    /// Utile per ispezionare i pattern accumulati durante l'analisi
    /// prima di finalizzare.
    ///
    /// # Returns
    ///
    /// Riferimento read-only alle statistiche accumulate.
    ///
    /// # Esempio
    ///
    /// ```rust,ignore
    /// let mut learner = PatternLearner::new("my-project");
    ///
    /// // Durante l'analisi, controlla i progressi
    /// for (i, file) in source_files.iter().enumerate() {
    ///     learner.analyze_file(file)?;
    ///     
    ///     if i % 10 == 0 {
    ///         let stats = learner.stats();
    ///         println!("Progresso: {} file, {} struct",
    ///             stats.files_analyzed,
    ///             stats.naming.struct_suffixes.len()
    ///         );
    ///     }
    /// }
    ///
    /// // Finalizza solo alla fine
    /// let patterns = learner.finalize();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Nota
    ///
    /// A differenza di `finalize()`, questo metodo non calcola
    /// i confidence scores. Usa `finalize()` per i punteggi finali.
    pub fn stats(&self) -> &LearnedPatterns {
        &self.stats
    }
    
    /// Esporta i pattern correnti in formato TOML.
    ///
    /// Utile per salvare i pattern estratti in un file di configurazione
    /// che può essere riutilizzato da Aether per la validazione.
    ///
    /// # Returns
    ///
    /// `Ok(String)` con il TOML formattato, o `Err(Error::Toml)` se la serializzazione fallisce.
    ///
    /// # Esempio
    ///
    /// ```rust,ignore
    /// let mut learner = PatternLearner::new("my-project");
    ///
    /// for file in source_files {
    ///     learner.analyze_file(&file)?;
    /// }
    ///
    /// // Esporta senza consumare il learner
    /// let toml = learner.to_toml()?;
    /// std::fs::write(".aether/learned.toml", toml)?;
    ///
    /// // Il learner può ancora essere usato
    /// let patterns = learner.finalize();
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Formato Output
    ///
    /// ```toml
    /// project = "my-project"
    /// language = "rust"
    /// analyzed_at = "2024-01-15T10:30:00Z"
    /// files_analyzed = 42
    ///
    /// [naming]
    /// struct_suffixes = { Config = 15, Builder = 8 }
    ///
    /// [confidence]
    /// naming = 0.85
    /// derives = 0.92
    /// ```
    ///
    /// # Nota
    ///
    /// I confidence scores nel TOML sono quelli dell'ultimo `finalize()`
    /// chiamato, o 0.0 se `finalize()` non è mai stato chiamato.
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(&self.stats).map_err(|e| Error::Toml(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_learner_basic() {
        let mut learner = PatternLearner::new("test-project");
        
        let code = r#"
/// Documentation for UserConfig
#[derive(Debug, Clone, PartialEq)]
pub struct UserConfig {
    name: String,
}

#[derive(Debug, Clone)]
struct InternalState {
    data: Vec<u8>,
}

pub fn get_config() -> UserConfig {
    UserConfig { name: "default".into() }
}

pub fn is_valid(value: &str) -> bool {
    !value.is_empty()
}
"#;
        
        learner.analyze_file(code).unwrap();
        let patterns = learner.finalize();
        
        assert_eq!(patterns.files_analyzed, 1);
        assert!(patterns.naming.struct_suffixes.contains_key("Config"));
        assert!(patterns.naming.function_prefixes.contains_key("get_"));
        assert!(patterns.naming.function_prefixes.contains_key("is_"));
        assert!(patterns.derives.debug_percentage > 0.0);
    }
}
