# Aether Learner — User Learning System

**Version:** 0.1.0
**Status:** Planned
**Related:** [AETHER_RAG.md](./AETHER_RAG.md), [AETHER_PRE_GUIDANCE.md](./AETHER_PRE_GUIDANCE.md)

---

## Overview

Aether Learner è il sistema di apprendimento che permette ad Aether di:

1. **Imparare dalle preferenze dell'utente** — Linguaggi, pattern, stili
2. **Ricordare errori comuni** — Cosa l'utente ha corretto in passato
3. **Ottimizzare il flusso** — Ridurre iterazioni anticipando le necessità

---

## Architettura

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        AETHER LEARNER ARCHITECTURE                          │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                        INPUT SOURCES                                  │  │
│  │  • Validation results (errors/warnings)                              │  │
│  │  • User corrections (when user fixes Aether's suggestion)           │  │
│  │  • Prompt patterns (how user phrases requests)                       │  │
│  │  • Code preferences (style, patterns)                                │  │
│  └────────────────────────────────┬─────────────────────────────────────┘  │
│                                   │                                          │
│                                   ▼                                          │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                      LEARNING ENGINE                                  │  │
│  │                                                                       │  │
│  │   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐               │  │
│  │   │   Profile   │   │    Stats    │   │   Memory    │               │  │
│  │   │   Manager   │   │   Tracker   │   │   Store     │               │  │
│  │   └──────┬──────┘   └──────┬──────┘   └──────┬──────┘               │  │
│  │          │                  │                  │                      │  │
│  │          └──────────────────┴──────────────────┘                      │  │
│  │                             │                                          │  │
│  │   ┌─────────────────────────────────────────────────────────────┐   │  │
│  │   │                    PATTERN EXTRACTOR                        │   │  │
│  │   │  • Code patterns                                            │   │  │
│  │   │  • Error patterns                                           │   │  │
│  │   │  • Style patterns                                           │   │  │
│  │   └─────────────────────────────────────────────────────────────┘   │  │
│  └────────────────────────────────┬─────────────────────────────────────┘  │
│                                   │                                          │
│                                   ▼                                          │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                        OUTPUT                                         │  │
│  │  • User Profile (preferences, expertise)                             │  │
│  │  • Pattern Library (learned patterns)                                │  │
│  │  • Pre-Guidance hints (what to warn about)                           │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Componenti

### 1. User Profile

```rust
// crates/aether-learner/src/profile.rs
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Profilo utente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// ID univoco utente.
    pub id: String,
    
    /// Linguaggi preferiti con frequenza.
    pub languages: HashMap<String, LanguageStats>,
    
    /// Pattern di codice preferiti.
    pub preferred_patterns: Vec<String>,
    
    /// Regole spesso violate.
    pub common_violations: Vec<ViolationRecord>,
    
    /// Stile di codice.
    pub code_style: CodeStyle,
    
    /// Data creazione profilo.
    pub created_at: DateTime<Utc>,
    
    /// Ultimo aggiornamento.
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStats {
    /// Numero di validazioni per questo linguaggio.
    pub validation_count: usize,
    
    /// Percentuale di successo (codice che passa senza modifiche).
    pub success_rate: f32,
    
    /// Ultima volta usato.
    pub last_used: DateTime<Utc>,
    
    /// Violazioni più comuni per questo linguaggio.
    pub common_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationRecord {
    /// ID della violazione.
    pub rule_id: String,
    
    /// Numero di volte violata.
    pub count: usize,
    
    /// Ultima violazione.
    pub last_occurred: DateTime<Utc>,
    
    /// Se l'utente ha corretto seguendo il suggerimento.
    pub followed_suggestion: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeStyle {
    /// Preferenza naming (snake_case, camelCase, PascalCase).
    pub naming_convention: String,
    
    /// Lunghezza massima linea preferita.
    pub max_line_length: usize,
    
    /// Preferenza commenti.
    pub comment_style: CommentStyle,
    
    /// Indentazione (2 o 4 spazi, tabs).
    pub indentation: IndentationStyle,
}

impl UserProfile {
    /// Crea un nuovo profilo.
    pub fn new(id: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            languages: HashMap::new(),
            preferred_patterns: Vec::new(),
            common_violations: Vec::new(),
            code_style: CodeStyle::default(),
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Registra una validazione.
    pub fn record_validation(&mut self, language: &str, result: &ValidationResult) {
        // Aggiorna statistiche linguaggio
        // Registra violazioni
        // Aggiorna timestamp
    }
    
    /// Ottieni suggerimenti basati sul profilo.
    pub fn get_guidance_hints(&self, language: &str) -> Vec<GuidanceHint> {
        let mut hints = Vec::new();
        
        // Avvisa su errori comuni
        if let Some(stats) = self.languages.get(language) {
            for error in &stats.common_errors {
                hints.push(GuidanceHint {
                    hint_type: HintType::CommonError,
                    message: format!("Attenzione: spesso commetti errore '{}'", error),
                    severity: Severity::Warning,
                });
            }
        }
        
        // Avvisa su regole spesso violate
        for violation in &self.common_violations {
            hints.push(GuidanceHint {
                hint_type: HintType::ViolatedRule,
                message: format!("Ricorda: la regola {} è stata violata {} volte", 
                    violation.rule_id, violation.count),
                severity: Severity::Info,
            });
        }
        
        hints
    }
}
```

### 2. Stats Tracker

```rust
// crates/aether-learner/src/stats.rs
use std::collections::HashMap;

/// Traccia statistiche di utilizzo.
pub struct StatsTracker {
    /// Validazioni per linguaggio.
    language_stats: HashMap<String, LanguageStats>,
    
    /// Validazioni per dominio.
    domain_stats: HashMap<String, DomainStats>,
    
    /// Validazioni per tipo di intent.
    intent_stats: HashMap<String, IntentStats>,
    
    /// Tempi di validazione.
    timing_stats: TimingStats,
}

#[derive(Debug, Clone)]
pub struct LanguageStats {
    pub total_validations: usize,
    pub successful_validations: usize,
    pub avg_iterations: f32,
    pub common_violations: Vec<(String, usize)>,
}

impl StatsTracker {
    /// Registra una validazione completata.
    pub fn record(&mut self, record: ValidationRecord) {
        // Aggiorna statistiche
    }
    
    /// Ottieni statistiche per linguaggio.
    pub fn get_language_stats(&self, language: &str) -> Option<&LanguageStats> {
        self.language_stats.get(language)
    }
    
    /// Predici probabili errori per un linguaggio.
    pub fn predict_common_errors(&self, language: &str) -> Vec<String> {
        // Basato su storico, quali errori sono più probabili
    }
}
```

### 3. Memory Store

```rust
// crates/aether-learner/src/memory.rs
use std::collections::HashMap;
use std::path::PathBuf;

/// Tipo di memoria.
#[derive(Debug, Clone, Copy)]
pub enum MemoryType {
    /// Correzione fatta dall'utente.
    UserCorrection,
    /// Pattern imparato.
    LearnedPattern,
    /// Preferenza esplicita.
    ExplicitPreference,
    /// Lezione da errore.
    ErrorLesson,
}

/// Record di memoria.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: String,
    pub memory_type: MemoryType,
    pub content: String,
    pub context: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub relevance: f32,
}

/// Storage per le memorie.
pub struct MemoryStore {
    /// Memorie per tipo.
    memories: HashMap<MemoryType, Vec<MemoryRecord>>,
    
    /// Indice per ricerca rapida.
    index: HashMap<String, usize>,
    
    /// Path del file di storage.
    storage_path: PathBuf,
}

impl MemoryStore {
    /// Carica memorie da disco.
    pub fn load(path: &Path) -> Result<Self, LearnerError> {
        // Carica da JSON
    }
    
    /// Salva memorie su disco.
    pub fn save(&self) -> Result<(), LearnerError> {
        // Salva in JSON
    }
    
    /// Aggiungi una memoria.
    pub fn add(&mut self, memory: MemoryRecord) {
        // Aggiungi e indicizza
    }
    
    /// Cerca memorie rilevanti.
    pub fn search(&self, query: &str, limit: usize) -> Vec<&MemoryRecord> {
        // Ricerca keyword nel contenuto
    }
    
    /// Ottieni memorie per tipo.
    pub fn get_by_type(&self, memory_type: MemoryType) -> &[MemoryRecord] {
        self.memories.get(&memory_type).map(|v| v.as_slice()).unwrap_or(&[])
    }
}
```

### 4. Pattern Extractor

```rust
// crates/aether-learner/src/pattern.rs

/// Estrae pattern dal codice e dalle correzioni.
pub struct PatternExtractor {
    /// Pattern estratti.
    patterns: Vec<ExtractedPattern>,
}

#[derive(Debug, Clone)]
pub struct ExtractedPattern {
    /// Tipo di pattern.
    pub pattern_type: PatternType,
    
    /// Codice originale (se correzione).
    pub original: Option<String>,
    
    /// Codice corretto.
    pub corrected: String,
    
    /// Descrizione del pattern.
    pub description: String,
    
    /// Contesto (linguaggio, dominio).
    pub context: PatternContext,
    
    /// Frequenza di occorrenza.
    pub frequency: usize,
    
    /// Confidenza del pattern.
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub enum PatternType {
    /// Pattern di codice preferito.
    PreferredCode,
    
    /// Errore comune da evitare.
    ErrorAvoidance,
    
    /// Stile di naming.
    NamingStyle,
    
    /// Pattern architetturale.
    Architectural,
    
    /// Ottimizzazione applicata.
    Optimization,
}

impl PatternExtractor {
    /// Estrai pattern da una correzione.
    pub fn extract_from_correction(
        original: &str,
        corrected: &str,
        violations: &[Violation],
    ) -> Option<ExtractedPattern> {
        // Analizza differenze
        // Identifica pattern
        // Calcola confidenza
    }
    
    /// Estrai pattern dal codice valido.
    pub fn extract_from_code(code: &str, language: &str) -> Vec<ExtractedPattern> {
        // Identifica pattern di stile
        // Identifica pattern architetturali
        // Calcola frequenza
    }
}
```

---

## Flusso di Apprendimento

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        LEARNING FLOW                                         │
│                                                                              │
│  Scenario 1: Errore Corretto dall'Utente                                    │
│  ─────────────────────────────────────                                      │
│                                                                              │
│  1. Aether valida codice                                                    │
│     → Trova violazione RUST001 (unwrap senza contesto)                      │
│     → Suggerisce: "Usa expect() con messaggio"                              │
│                                                                              │
│  2. Utente corregge diversamente                                            │
│     → Utente scrive: "Usa ? operator con Result"                            │
│     → Questo è DIVERSO dal suggerimento di Aether                           │
│                                                                              │
│  3. Aether impara                                                           │
│     → Learner.record_correction(                                            │
│         original: "x.unwrap()",                                             │
│         suggested: "x.expect(\"ctx\")",                                     │
│         actual: "x?"                                                        │
│       )                                                                      │
│     → Pattern estratto: "Per questo utente, preferisce ? su expect"         │
│     → Memorizzato come PreferredPattern                                     │
│                                                                              │
│  4. Prossima volta                                                          │
│     → Aether suggerisce "x?" invece di "x.expect()"                         │
│     → Utente più soddisfatto, meno iterazioni                              │
│                                                                              │
│  ────────────────────────────────────────────────────────────────────────  │
│                                                                              │
│  Scenario 2: Violazione Ricorrente                                          │
│  ─────────────────────────────────────                                      │
│                                                                              │
│  1. Utente violola RUST003 (clone non necessario) 5 volte                  │
│                                                                              │
│  2. Learner rileva pattern                                                  │
│     → stats.record_violation("RUST003")                                     │
│     → count = 5                                                             │
│     → threshold = 3                                                         │
│                                                                              │
│  3. Pre-Guidance attivo                                                     │
│     → Quando utente scrive codice con .clone()                              │
│     → Aether AVVISA prima:                                                  │
│       "Attenzione: spesso usi .clone() quando non necessario"               │
│       "Vuoi usare & invece?"                                                │
│                                                                              │
│  4. Utente corregge PRIMA di validazione                                    │
│     → Zero iterazioni sprecate                                              │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Integrazione con Pre-Guidance

```rust
// crates/aether-wrapper/src/pre_guidance.rs
impl PreGuidance {
    /// Genera contesto per l'agente prima che scriva codice.
    pub fn generate_agent_context(&self, prompt: &str) -> AgentContext {
        // 1. Analizza prompt
        let analysis = self.analyzer.analyze(prompt);
        
        // 2. Ottieni profilo utente
        let profile = self.learner.get_profile();
        
        // 3. Ottieni suggerimenti dal profilo
        let hints = profile.get_guidance_hints(&analysis.domain.primary);
        
        // 4. Cerca nella documentazione
        let docs = self.rag.search(&analysis.keywords.join(" "), 5);
        
        // 5. Ottieni pattern rilevanti
        let patterns = self.learner.get_patterns_for_domain(&analysis.domain.primary);
        
        // 6. Genera contesto
        AgentContext {
            prompt_analysis: analysis,
            user_hints: hints,
            documentation: docs,
            learned_patterns: patterns,
            warnings: self.generate_warnings(&hints),
        }
    }
    
    fn generate_warnings(&self, hints: &[GuidanceHint]) -> Vec<String> {
        hints.iter()
            .filter(|h| h.severity == Severity::Warning)
            .map(|h| h.message.clone())
            .collect()
    }
}
```

---

## Storage

```
.aether/
├── learner/
│   ├── profile.json          # Profilo utente
│   ├── stats.json            # Statistiche aggregate
│   ├── memories.json         # Memorie e correzioni
│   └── patterns.json         # Pattern estratti
└── config/
    └── learner.yaml          # Configurazione
```

### Configurazione

```yaml
# .aether/config/learner.yaml
version: "1.0"

# Apprendimento
learning:
  enabled: true
  # Minimo occorrenze per imparare un pattern
  pattern_threshold: 3
  # Minimo violazioni per attivare warning preventivo
  violation_threshold: 3
  
# Memoria
memory:
  max_memories: 1000
  # Giorni prima di dimenticare
  retention_days: 365
  # Salvataggio automatico
  auto_save: true
  save_interval_seconds: 60
  
# Privacy
privacy:
  # Non salvare codice sensibile
  exclude_patterns:
    - "*password*"
    - "*secret*"
    - "*key*"
  # Anonimizza percorsi file
  anonymize_paths: false
```

---

## API

### Rust API

```rust
use aether_learner::{Learner, UserProfile, GuidanceHint};

// Inizializza learner
let learner = Learner::new("./project/.aether/learner")?;

// Registra validazione
learner.record_validation("rust", &result);

// Registra correzione utente
learner.record_correction(
    "x.unwrap()",
    "x?",
    &[Violation::new("RUST001", "unwrap without context")]
);

// Ottieni suggerimenti
let hints = learner.get_guidance_hints("rust");
for hint in hints {
    println!("[{}] {}", hint.severity, hint.message);
}
```

### MCP Tool

```json
{
  "name": "aether_learn",
  "description": "Record a user correction or preference for Aether to learn",
  "inputSchema": {
    "type": "object",
    "properties": {
      "type": {
        "type": "string",
        "enum": ["correction", "preference", "lesson"]
      },
      "content": {
        "type": "string",
        "description": "The content to learn"
      },
      "context": {
        "type": "string",
        "description": "Optional context (language, domain)"
      }
    },
    "required": ["type", "content"]
  }
}
```

---

## Benefici

| Beneficio | Senza Learner | Con Learner |
|-----------|---------------|-------------|
| Iterazioni medie | 2-3 | 1-1.5 |
| Errori ricorrenti | Spesso | Rari |
| Suggerimenti rilevanti | 50% | 85% |
| Tempo per validazione | 100% | 40% |

---

## Prossimi Passi

> **Vedi [ROADMAP_INDEX.md](./ROADMAP_INDEX.md)** per la roadmap consolidata.

---

## Note Commerciali

| Feature | Community | Commercial |
|---------|-----------|------------|
| Basic Profile | ✅ | ✅ |
| Stats Tracking | ✅ | ✅ |
| Pattern Learning | ❌ | ✅ |
| Memory Store | ❌ | ✅ |
| Cloud Sync | ❌ | ✅ |
| Team Learning | ❌ | ✅ |
