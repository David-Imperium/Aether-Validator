# ADR-002: Aether Neural Native — Architettura Neuro-Symbolic

**Data**: 2026-03-30
**Stato**: Approved
**Decisione**: Aether evolve a sistema neuro-symbolic nativo con reti neurali custom

---

## Contesto

Aether Intelligence è un sistema rule-based con 5 layer. Layer 1-2 completati (validazione, code graph, decision log), Layer 3 parziale (pattern discovery statistico), Layer 4-5 solo design (intent inference, drift detection).

Il layer "intelligente" attuale dipende da regole deterministiche e statistica classica (Jaccard, TF-IDF). Non ha capacità di generalizzazione, reasoning, o apprendimento neurale.

## Decisione

Aether diventa un sistema **neuro-symbolic nativo** con tre reti neurali custom addestrate per compiti specifici di validazione codice.

### Reti

| Rete | Architettura | Scopo | Parametri |
|---|---|---|---|
| Code Reasoner | GNN (GGNN/GAT) su CPG | Classificazione, spiegazione, fix | ~8M |
| Pattern Memory | TreeFFN + Hopfield | Pattern matching, esperienza | ~5M |
| Drift Predictor | Temporal GNN + SSM | Predizione drift | ~5M |

### Stack Tecnico

- **Training**: NexusTrain (Python/PyTorch) esteso per architetture custom
- **Inference**: Burn (Rust) con modelli ONNX
- **Feature**: Code Property Graph (AST + CFG + DFG) via tree-sitter
- **Integration**: Estensione del crate `aether-intelligence` esistente

### Implementazione

5 fasi incrementali, ogni fase produce valore:
1. GNN su CPG (classificazione)
2. Pattern Memory (ricordo + similarità)
3. Drift Prediction (predizione temporale)
4. Orchestrator (ensemble + spiegazioni + auto-training)
5. Neuro-Symbolic Unification (sistema integrato)

## Alternative Valutate

### 1. GNN solo (Quick Win)

Aggiungere solo un GNN al sistema esistente.

**Pro**: Implementazione rapida, meno rischioso.
**Contro**: Non scala a reasoning completo. Rimarrebbe un "addon" invece che un sistema nativo.

**Scartato**: Non ambizioso abbastanza per la visione del progetto.

### 2. LLM Fine-Tuning (via NexusTrain)

Fine-tunare un piccolo LLM (es. Qwen 1.5B) per validazione.

**Pro**: NexusTrain lo supporta già, generazione testo naturale.
**Contro**: Dipende da un modello locale (contraddice "senza dipendere da LLM esterni"), inference più lenta, meno interpretabile.

**Scartato**: Va contro il requisito di autonomia senza LLM.

### 3. Python ML standalone

Sistema ML separato in Python con API.

**Pro**: Ecosistema PyTorch maturo.
**Contro**: Dipendenza runtime Python, latenza IPC, due stack da mantenere.

**Scartato**: Aether è Rust-native, Python solo per training (offline).

## Conseguenze

### Positive
- Aether ragiona sul codice senza LLM esterni
- Migliora autonomamente con l'uso (auto-training)
- Pattern memory persistente e incrementale
- Predizione drift preventiva
- System unico nel panorama dei validatori

### Negative
- Complessità aggiunta: 3 reti + orchestrator
- Training data iniziale limitato (mitigato con corpus esterni)
- Burn meno maturo di PyTorch per inference GNN (mitigato con ONNX fallback)
- ~18M parametri totali = overhead memoria e storage

## Decisioni correlate

- ADR-001: Hybrid Memory Architecture (riutilizzata per model versioning)
- Documento di design: [AETHER_NEURAL.md](../AETHER_NEURAL.md)
