# Guide de test de l'indicateur de chargement

## Option 1 : Tests unitaires automatisés ✅

Les tests suivants vérifient la logique de l'indicateur de chargement :

```bash
# Exécuter tous les tests de loading state
cargo test loading_state

# Résultat attendu :
# ✅ loading_state_timeout_converts_to_error
# ✅ loading_state_timeout_does_not_trigger_before_timeout
# ✅ loading_state_resets_on_successful_load
```

Ces tests vérifient :
- ✅ Le timeout de 10 secondes déclenche une erreur
- ✅ Avant 10 secondes, pas d'erreur
- ✅ Le chargement réussi réinitialise l'état

## Option 2 : Test manuel avec délai artificiel ⭐ RECOMMANDÉ

### Méthode simple avec script

```bash
# 1. Ajouter le délai de test (2 secondes)
./scripts/add_test_delay.sh

# 2. Rebuild
cargo build --release

# 3. Tester au démarrage
./target/release/iced_lens tests/data/sample.png
# ➜ Le spinner devrait apparaître pendant 2 secondes

# 4. Tester pendant la navigation
# Ouvre l'app avec plusieurs images, puis navigue avec les flèches ← →
./target/release/iced_lens tests/data/

# 5. Retirer le délai
./scripts/remove_test_delay.sh
# Ou: git checkout src/media/mod.rs
```

### Méthode manuelle (si les scripts ne marchent pas)

Ajoute temporairement ces lignes dans `src/media/mod.rs` après la ligne 183 :

```rust
    // TEMPORARY: Artificial delay to test loading indicator
    // Remove this before committing!
    std::thread::sleep(std::time::Duration::from_secs(2));
```

**Rebuild et teste :**
```bash
cargo build --release
./target/release/iced_lens tests/data/sample.png
```

**Ce que tu devrais voir :**
1. Le spinner apparaît immédiatement avec le texte "Loading..." (ou "Chargement..." en français)
2. Après 2 secondes, l'image s'affiche normalement
3. Pas d'erreur (le timeout est à 10 secondes)

**Pour tester le timeout (erreur après 10s) :**
```rust
std::thread::sleep(std::time::Duration::from_secs(11));  // > 10 secondes
```

**Résultat attendu :**
- Message d'erreur : "Loading timed out. The file may be too large or inaccessible."

### Méthode avec patch (recommandée)

```bash
# Appliquer le patch de test
git apply test_loading_delay.patch

# Rebuild
cargo build --release

# Tester
./target/release/iced_lens tests/data/sample.png

# Retirer le patch
git apply -R test_loading_delay.patch
```

## Option 3 : Test avec fichier réseau lent

Si tu as accès à un NAS ou un partage réseau lent :

```bash
# Exemple avec un fichier sur NAS
./target/release/iced_lens /mnt/nas/huge_video.mp4
```

Le spinner devrait apparaître naturellement si le chargement prend plus de 100ms.

## Vérification visuelle

L'indicateur de chargement affiche :
- ✅ Un spinner SVG animé (cercle avec trait)
- ✅ Texte "Loading..." (EN) ou "Chargement..." (FR)
- ✅ Centré à l'écran sur fond blanc/gris (selon le thème)
- ✅ **Nouveau** : Remplace le message "Hello, world!" au démarrage

**Comportements selon le contexte :**
1. **Au démarrage** (pas encore de media) : Spinner centré plein écran
2. **Navigation entre images** (avec media déjà chargé) : Spinner en overlay semi-transparent sur l'image précédente
3. **Après timeout (>10s)** : Message d'erreur avec détails

## Notes importantes

⚠️ **N'oublie pas de retirer le délai artificiel avant de commit !**

```bash
# Vérifier qu'il n'y a pas de std::thread::sleep dans load_media
git diff src/media/mod.rs
```
