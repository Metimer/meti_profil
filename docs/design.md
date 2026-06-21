# Spécification de design — meti_profil

## Objectif

Recréer une alternative moderne et performante à `ydata-profiling` pour l'exploration rapide de datasets tabulaires.

**Cibles du MVP :**
- Moteur de profilage écrit en Rust pour la performance.
- Binding Python installable via `pip install meti_profil`.
- Génération d'un rapport Markdown hybride : lisible par un humain et structuré pour être consommé par des agents de code.
- Génération d'un rapport HTML interactif auto-contenu, et rendu inline dans Jupyter / VSCode.
- Support de CSV, Parquet et Excel en entrée.
- Support de datasets jusqu'à quelques millions de lignes en mémoire.

**Hors scope du MVP :**
- Serveur web / application live (exploration dynamique côté serveur).
- Streaming sur des datasets dépassant la RAM.
- Moteurs distribués ou calculs sur des milliards de lignes.

## Rapport HTML interactif

En plus du Markdown, le moteur produit un document HTML autonome destiné aux
data scientists / analysts :

- **Auto-contenu** : CSS et JavaScript embarqués, aucune ressource externe ni
  CDN — le fichier fonctionne hors-ligne et se partage tel quel.
- **Visualisations** rendues en SVG interactif par un petit script vanilla
  (pas de dépendance JS lourde) : histogrammes (numérique), barres de
  fréquences (catégoriel), aperçu des valeurs manquantes, heatmap de
  corrélation Pearson — avec tooltips au survol.
- **Données** : le `Report` est sérialisé en JSON et embarqué dans la page ;
  le script dessine les graphiques à partir de ce payload.
- **API** : `report.to_html(path)` écrit le fichier, `report.to_html()`
  retourne la chaîne, et `report._repr_html_()` encapsule le document dans une
  `<iframe sandbox>` pour un rendu isolé en notebook.
- **Implémentation** : `report/html.rs` (`HtmlRenderer`) +
  `report/assets/report.{css,js}` inclus à la compilation via `include_str!`.

## Architecture globale

Le projet est organisé en un workspace Cargo avec un packaging Python via Maturin.

```
meti_profil/
├── pyproject.toml
├── Cargo.toml          # workspace
├── crates/
│   ├── meti_profil_core/   # Moteur Rust pur
│   └── meti_profil_py/     # Binding PyO3
├── python/
│   └── meti_profil/
│       └── __init__.py     # Facade Python optionnelle
└── tests/
    ├── rust/               # Tests unitaires Rust
    ├── python/             # Tests d'intégration Python
    └── fixtures/           # Datasets synthétiques
```

### `meti_profil_core`

Crate Rust contenant :

- **`io/`** : lecteurs de fichiers.
  - `csv.rs` : lecture CSV via `csv` / `arrow-csv` → `RecordBatch`.
  - `parquet.rs` : lecture Parquet via `parquet` d'arrow-rs → `RecordBatch`.
  - `excel.rs` : lecture Excel via `calamine` → conversion manuelle en `RecordBatch`.
- **`dataframe/`** : couche interne minimaliste au-dessus d'`arrow-rs`.
  - `DataFrame` : conteneur de `RecordBatch` + schéma.
  - `Column` : abstraction sur une `Array` Arrow typée.
- **`analysis/`** : modules d'analyse indépendants.
  - `schema.rs` : types détectés, cardinalités, constantes.
  - `numeric.rs` : statistiques descriptives (min, max, mean, median, std, skewness, kurtosis) + histogramme.
  - `categorical.rs` : valeurs uniques, top values, fréquences.
  - `missing.rs` : nulls, vides, patterns de missingness.
  - `duplicate.rs` : détection de lignes dupliquées.
  - `correlation.rs` : corrélations Pearson entre colonnes numériques.
- **`report/`** : génération du rapport.
  - `model.rs` : structures aggrégées des résultats.
  - `markdown.rs` : rendu Markdown hybride.

### `meti_profil_py`

Crate Rust utilisant PyO3 pour exposer :

- `ProfileReport` : classe Python.
  - `__new__(source, *, title="Dataset Profile", minimal=False, explorative=True)`
  - `to_file(path)`
  - `to_markdown()` -> `str`
  - `get_summary()` -> `dict`
  - `get_column_info(name)` -> `dict`

Le binding accepte en entrée :
- `str` / `Path` : chemin vers un fichier CSV, Parquet ou Excel.
- `pandas.DataFrame` : converti via `pyarrow` en `RecordBatch`.
- `polars.DataFrame` : converti directement en `RecordBatch`.

## Flux de données

1. **Entrée Python** : `ProfileReport(source)` reçoit un DataFrame ou un chemin.
2. **Conversion Arrow** : la source est transformée en un ou plusieurs `RecordBatch` Arrow.
3. **Analyses** : chaque module d'analyse travaille sur le `RecordBatch` et produit une structure de résultat.
4. **Agrégation** : un `Report` centralise tous les résultats.
5. **Rendu** : `MarkdownRenderer` écrit le rapport dans un fichier ou une chaîne.

## API Python

```python
import meti_profil as mp

# Source fichier
report = mp.ProfileReport("data.csv")

# Source DataFrame
import pandas as pd
df = pd.read_csv("data.csv")
report = mp.ProfileReport(df, title="Profilage clients")

# Exports
report.to_file("rapport.md")
md = report.to_markdown()

# Accès programmatique
summary = report.get_summary()
age_info = report.get_column_info("age")
```

### Paramètres de `ProfileReport`

| Paramètre | Type | Défaut | Description |
|-----------|------|--------|-------------|
| `source` | `str`, `Path`, `DataFrame` | requis | Source de données. |
| `title` | `str` | `"Dataset Profile"` | Titre du rapport. |
| `minimal` | `bool` | `False` | Réduit les analyses lourdes (corrélations, histogrammes). |
| `explorative` | `bool` | `True` | Active les analyses avancées (corrélations, dépendances). |

## Format de sortie Markdown

Le fichier Markdown est hybride : lisible par un humain et structuré pour un agent.

### Frontmatter

```yaml
---
title: Profilage clients
source: data.csv
rows: 1000000
columns: 12
missing_cells: 32000
missing_cells_pct: 0.32
duplicate_rows: 1250
duplicate_rows_pct: 0.125
generated_at: 2026-06-21T11:58:57Z
meti_profil_version: 0.1.0
---
```

### Sections normalisées

Chaque section utilise un titre de niveau 2 normalisé :

- `## Overview`
- `## Schema`
- `## Numeric Columns`
- `## Categorical Columns`
- `## Missing Values`
- `## Duplicate Rows`
- `## Correlations`
- `## Recommendations`

### Exemple de section colonne

```markdown
## Numeric Columns

### age

| Statistic | Value |
|-----------|------:|
| type      | int64 |
| count     | 1000000 |
| missing   | 0 |
| mean      | 45.3 |
| std       | 16.2 |
| min       | 18 |
| 25%       | 32 |
| median    | 44 |
| 75%       | 58 |
| max       | 99 |
| skewness  | 0.12 |
| kurtosis  | -0.05 |
```

## Analyses incluses dans le MVP

### 1. Schéma et types
- Nombre de lignes et colonnes.
- Type détecté : `numeric`, `categorical`, `boolean`, `datetime`, `text`, `constant`, `unique`.
- Nombre de valeurs uniques et taux de cardinalité.

### 2. Statistiques descriptives
- Pour les colonnes numériques : count, mean, std, min, 25%, median, 75%, max, skewness, kurtosis.
- Histogramme discrétisé (10 bins par défaut).

### 3. Colonnes catégorielles
- Nombre de valeurs uniques.
- Top 10 valeurs avec fréquences.

### 4. Valeurs manquantes
- Compte et pourcentage par colonne.
- Compte global.

### 5. Lignes dupliquées
- Nombre et pourcentage de lignes dupliquées.

### 6. Corrélations
- Matrice Pearson pour les colonnes numériques.
- Signalement des paires avec corrélation absolue > 0.9.

## Stack technique

### Rust
- `arrow-rs` : format de données intermédiaire.
- `csv` + `arrow-csv` : lecture CSV.
- `parquet` : lecture Parquet.
- `calamine` : lecture Excel.
- `pyo3` : binding Python.
- `serde` : sérialisation des résultats.
- `statrs` ou implémentation maison : statistiques.

### Python
- `maturin` : build-backend pour le wheel.
- `pytest` : tests.
- `pandas`, `polars`, `pyarrow` : tests d'intégration.

### CI / CD
- GitHub Actions.
- Tests Rust sur Linux, macOS, Windows.
- Tests Python sur 3.10, 3.11, 3.12.
- Publication PyPI automatique sur tag.

## Tests

### Tests Rust
- Tests unitaires pour chaque analyse sur des petits tableaux Arrow.
- Tests de lecture pour chaque format (CSV, Parquet, Excel).
- Tests de rendu Markdown avec comparaison de snapshots.

### Tests Python
- Génération de rapport depuis pandas, polars et fichiers.
- Vérification de la présence des sections Markdown attendues.
- Comparaison des statistiques avec des datasets de référence.

### Fixtures
- `tests/fixtures/small.csv` : dataset synthétique de 100 lignes.
- `tests/fixtures/small.parquet` : version Parquet.
- `tests/fixtures/small.xlsx` : version Excel.

## Qualité du code

- Rust : `cargo clippy`, `cargo fmt`.
- Python : `ruff`, `mypy`.
- Pré-commit hooks optionnels.

## Packaging

```toml
[build-system]
requires = ["maturin>=1.0"]
build-backend = "maturin"

[project]
name = "meti_profil"
version = "0.1.0"
description = "Modern, fast data profiling in Rust with Python bindings"
requires-python = ">=3.10"
```

Installation par l'utilisateur final :

```bash
pip install meti_profil
```

## Évolutions futures

- UI web embarquée (serveur HTTP local).
- Export HTML interactif.
- Support du streaming pour fichiers plus grands que la RAM.
- Corrélations Spearman, Phi-K, Cramér's V.
- Détection automatique d'outliers.
- Profiling par chunks pour Parquet partitionné.

## Open questions / Hypothèses

1. Pour la lecture Excel, `calamine` lit `.xlsx` et `.xls`. Le support de `.ods` est hors scope MVP.
2. Les dates sont détectées soit par le schéma Arrow (Parquet), soit par inférence sur les chaînes CSV/Excel.
3. Le format Markdown est versionné implicitement via `meti_profil_version` dans le frontmatter.
