# Rapport de lacunes - Module Diagnostics

**Date :** 2026-01-14
**Identifié par :** Développeur (James), complété par PO (Sarah)
**Contexte :** Revue post-implémentation Story 3.4 (Export Buttons)

---

# 1. Lacune d'implémentation : Métadonnées média manquantes

## Constat

Les rapports de diagnostics exportés ne contiennent **aucune information sur les médias** consultés par l'utilisateur. Les événements `MediaLoadingStarted`, `MediaLoaded` et `MediaFailed` ne capturent que :
- `media_type` (image/video)
- `size_category` (small/medium/large)

## Informations manquantes

| Information | Valeur diagnostique |
|-------------|---------------------|
| Extension du fichier | Identifier les formats problématiques (ex: HEIC, AVIF, formats exotiques) |
| Type de stockage (local/réseau) | Corréler les lenteurs avec l'accès réseau (NAS, partages SMB) |
| Hash anonymisé du chemin | Tracer les opérations sur un même fichier à travers le temps |

## Analyse de la cause

- **Story 2.1** a implémenté `PathAnonymizer` avec préservation des extensions
- **Story 1.4** spécifie explicitement "paths excluded" pour les événements
- Le `PathAnonymizer` est utilisé pour nettoyer les messages d'erreur, mais **pas pour enrichir** les événements média

C'est une **incohérence entre les spécifications** : l'outil d'anonymisation existe mais n'est pas exploité là où il serait utile.

## Impact

Sans ces informations, un rapport de diagnostic ne permet pas de :
- Distinguer un problème de format d'un problème de performance
- Identifier si les ralentissements sont liés au réseau
- Corréler plusieurs événements concernant le même fichier

## Recommandation

Créer une story corrective pour enrichir les événements média avec :
1. `extension: Option<String>` - extension du fichier
2. `storage_type: StorageType` - enum `Local | Network | Unknown`
3. `path_hash: String` - hash anonymisé via `PathAnonymizer`

## Questions pour le PO

1. Cette correction doit-elle être intégrée à l'Epic 3 en cours ou faire l'objet d'un Epic/Story séparé ?
2. Priorité par rapport aux autres stories de l'Epic 3 ?
3. Le scope actuel est-il suffisant ou faut-il capturer d'autres métadonnées (dimensions image, codec vidéo, etc.) ?

---

# 2. Clarification requise : Story 3.4 critère d'acceptation #3

## Écart identifié

**Critère 3 de la Story 3.4 :**
> "Buttons disabled when collection is disabled OR buffer is empty"

**Implémentation actuelle :**
Les boutons sont désactivés uniquement quand `event_count == 0` (buffer vide).

## Contexte technique

L'application a deux niveaux de collection :
- **Collection légère** (toujours active) : actions utilisateur, états applicatifs
- **Collection ressources** (activable via toggle) : CPU, RAM, disque

Le buffer peut contenir des événements de la collection légère même quand le toggle "Resource Collection" est désactivé.

## Question pour le PO

4. Quelle est l'intention du critère 3 ?
   - **Option A** : Désactiver les boutons uniquement quand le buffer est vide (implémentation actuelle)
   - **Option B** : Désactiver les boutons quand le toggle "Resource Collection" est OFF, même si le buffer contient des événements

**Recommandation développeur** : L'option A semble plus utile car elle permet d'exporter les événements légers même sans la collection de ressources.

---

# 3. Lacune d'implémentation : SystemInfo incomplet (identifié par PO)

## Constat

L'audit des spécifications révèle un écart entre la Story 2.3 et l'implémentation actuelle de `SystemInfo`.

**Story 2.3 spécifie :**
> "System info: os, cpu_model, ram_total_mb, disk_type (without identifying info)"

**Implémentation actuelle (`src/diagnostics/report.rs`) :**
```rust
pub struct SystemInfo {
    pub os: String,
    pub os_version: String,
    pub cpu_cores: usize,
    pub ram_total_mb: u64,
}
```

## Écarts identifiés

| Spécification | Implémentation | Statut |
|---------------|----------------|--------|
| os | os ✓ | OK |
| cpu_model | cpu_cores | **Différent** |
| ram_total_mb | ram_total_mb ✓ | OK |
| disk_type | (absent) | **Manquant** |
| (non spécifié) | os_version | Ajout |

## Analyse

1. **cpu_model vs cpu_cores** : La spec demandait le modèle de CPU (ex: "Intel i7-12700K", "Apple M2") mais l'implémentation capture uniquement le nombre de cœurs. Le modèle serait plus utile pour identifier des problèmes spécifiques à certains processeurs.

2. **disk_type manquant** : Information utile pour corréler les performances I/O avec le type de stockage (SSD, HDD, NVMe). La spec précise "without identifying info" donc seul le type générique est attendu.

3. **os_version ajouté** : Cet ajout non spécifié est pertinent et bienvenu.

## Impact

Sans `cpu_model` et `disk_type`, les rapports manquent de contexte pour :
- Identifier des problèmes liés à des architectures CPU spécifiques (x86 vs ARM)
- Corréler les performances avec le type de stockage
- Diagnostiquer des problèmes de compatibilité matérielle

---

# Résumé des lacunes

| # | Lacune | Source | Priorité suggérée |
|---|--------|--------|-------------------|
| 1 | Métadonnées média (extension, storage_type, path_hash) | Événements média | Haute |
| 2 | Clarification critère 3 Story 3.4 | Spec ambiguë | Moyenne |
| 3 | SystemInfo (cpu_model, disk_type) | Story 2.3 non respectée | Moyenne |

---

# Décisions requises du Product Owner

## Questions existantes (du développeur)

1. La correction des métadonnées média doit-elle être intégrée à l'Epic 3 en cours ou faire l'objet d'un Epic/Story séparé ?
2. Priorité par rapport aux autres stories de l'Epic 3 ?
3. Le scope actuel est-il suffisant ou faut-il capturer d'autres métadonnées (dimensions image, codec vidéo, etc.) ?
4. Quelle est l'intention du critère 3 de la Story 3.4 ? (Option A vs Option B)

## Questions additionnelles (audit PO)

5. La lacune SystemInfo (cpu_model, disk_type) doit-elle être corrigée dans la même story que les métadonnées média ou séparément ?
6. Pour cpu_model : accepte-t-on d'exposer le nom complet du processeur ou faut-il anonymiser/généraliser (ex: "Intel x86_64", "Apple ARM64") ?
7. Ces corrections sont-elles bloquantes pour la release de l'Epic 3 ou peuvent-elles être traitées dans un Epic ultérieur ?

---

# Recommandations du Product Owner (Sarah)

## Analyse de l'impact

Après audit complet des spécifications Epic 1, Epic 2 et de l'implémentation, les lacunes identifiées sont:
- **2 lacunes d'implémentation** (métadonnées média, SystemInfo)
- **1 clarification de spec** (Story 3.4 critère 3)

## Décisions préliminaires

### Question 4 - Story 3.4 critère 3
**Décision : Option A (implémentation actuelle)**

Justification : L'option A est plus cohérente avec l'architecture à deux niveaux de collection. La collection légère (toujours active) capture des événements utiles même sans les métriques système. Empêcher l'export quand le toggle est OFF serait contre-productif.

**Action :** Clarifier le critère d'acceptation dans la spec pour refléter l'intention correcte.

### Questions 1, 2, 5 - Regroupement des corrections
**Recommandation : Créer une Story corrective unique dans l'Epic 3**

Story 3.5 proposée : "Enrich Diagnostic Report Data"
- Scope : Métadonnées média + SystemInfo manquants
- Rationale : Ces corrections sont complémentaires et augmentent la valeur diagnostique des rapports

### Question 3 - Scope des métadonnées média
**Recommandation : Scope minimaliste**

Capturer uniquement:
- `extension` (ex: "jpg", "mp4")
- `storage_type` (Local | Network | Unknown)
- `path_hash` (via PathAnonymizer existant)

Les métadonnées avancées (dimensions, codec) peuvent être ajoutées ultérieurement si nécessaire.

### Question 6 - cpu_model et confidentialité
**Recommandation : Généraliser le modèle CPU**

Format suggéré: `"architecture (vendor)"` ex: "x86_64 (Intel)", "aarch64 (Apple)"
- Préserve la valeur diagnostique (architecture, vendor)
- Évite d'exposer des identifiants potentiellement uniques

### Question 7 - Priorité et release
**Décision : Non-bloquant pour Epic 3 release**

Les rapports sont fonctionnels dans leur état actuel. Les enrichissements proposés augmentent la qualité mais ne sont pas critiques.

**Plan proposé :**
1. Terminer les stories restantes de l'Epic 3 (3.5 serait ajoutée)
2. La Story 3.5 devient la dernière story avant release
3. Si urgence, l'Epic 3 peut être release sans 3.5

---

# Prochaines étapes

1. [x] Valider ces recommandations avec les stakeholders
2. [ ] Créer la Story 3.5 avec les acceptance criteria détaillés
3. [ ] Mettre à jour le critère 3 de la Story 3.4 dans la spec
4. [ ] Assigner la Story 3.5 au développeur

---

# Validation technique (Architecte - Winston)

**Date :** 2026-01-14

## Verdict global : ✅ VALIDÉ avec ajustements

Les recommandations de Sarah sont techniquement solides. Quelques précisions et une contre-proposition ci-dessous.

## Analyse détaillée

### 1. Métadonnées média — ✅ Validé

| Champ | Verdict | Détail technique |
|-------|---------|------------------|
| `extension` | ✅ Trivial | `Path::extension()` suffit |
| `path_hash` | ✅ Prêt | Réutiliser `PathAnonymizer` existant (blake3, 8 chars hex) |
| `storage_type` | ⚠️ Avec réserve | Voir ci-dessous |

**Concernant `storage_type`** :

La détection Local vs Network n'est pas triviale cross-platform :
- Linux : `/mnt/*`, `/media/*` peut être local ou réseau (nécessite inspection de `/proc/mounts`)
- Windows : `\\server\share` est réseau, mais lettres mappées (`Z:\`) aussi
- macOS : `/Volumes/*` peut être NFS, SMB, ou disque USB local

**Recommandation** : Implémenter avec heuristique simple, accepter que `Unknown` soit fréquent :

```rust
pub enum StorageType {
    Local,      // /home, /Users, C:\Users, chemins "évidents"
    Network,    // UNC paths (\\\\), chemins avec signature SMB/NFS
    Unknown,    // Default si incertain — affiné ultérieurement si besoin
}
```

### 2. cpu_model format — ⚠️ Contre-proposition

**Proposition Sarah** : `"x86_64 (Intel)"`, `"aarch64 (Apple)"`

**Problème** : Le vendor n'est pas directement exposé par `sysinfo`. Il faudrait parser `cpu.brand()`.

**Contre-proposition architecte** : Capturer les deux informations séparément :

```rust
pub struct SystemInfo {
    pub os: String,
    pub os_version: String,
    pub cpu_arch: String,      // "x86_64", "aarch64" — via std::env::consts::ARCH
    pub cpu_brand: String,     // "Intel Core i7-12700K" — via sysinfo::Cpu::brand()
    pub cpu_cores: usize,      // Conserver — utile pour corrélation performance
    pub ram_total_mb: u64,
    pub disk_type: Option<DiskType>,
}
```

**Justification** :
- `cpu_brand` complet est plus utile pour diagnostiquer des bugs spécifiques
- Le risque vie privée est minimal (le modèle CPU n'est pas un identifiant unique)
- `cpu_arch` séparé permet filtrage facile x86 vs ARM dans les rapports

### 3. disk_type — ✅ Validé

`sysinfo::Disks::kind()` retourne déjà `DiskKind::SSD`, `HDD`, ou `Unknown`.

```rust
pub enum DiskType {
    Ssd,
    Hdd,
    Unknown,
}
```

**Note** : Capturer le type du disque contenant le répertoire home de l'utilisateur (disque principal).

### 4. Story 3.4 critère 3 (Option A) — ✅ Validé

L'Option A est architecturalement cohérente avec le design à deux niveaux de collection. L'export doit être possible dès qu'il y a des données exploitables.

## Tableau récapitulatif

| Recommandation | Verdict | Action |
|----------------|---------|--------|
| Story 3.5 unique | ✅ | Approuvé |
| `extension` | ✅ | Implémenter tel quel |
| `storage_type` enum | ✅ | Implémenter avec heuristique simple |
| `path_hash` | ✅ | Réutiliser `PathAnonymizer` |
| `cpu_model` généralisé | ⚠️ | **Modifier** : garder `cpu_brand` complet + ajouter `cpu_arch` |
| `disk_type` | ✅ | Utiliser `sysinfo::DiskKind` |
| Option A critère 3 | ✅ | Clarifier la spec |

## Impact sur l'estimation

Les ajustements proposés ne changent pas significativement la complexité. La Story 3.6 reste de taille raisonnable (estimée ~1-2 jours de développement).

---

# Validation PM (Product Manager - John)

**Date :** 2026-01-14

## Verdict global : ✅ VALIDÉ avec correction

### Conflit de numérotation corrigé

**Problème identifié** : La Story 3.5 proposée entre en conflit avec la Story 3.5 existante ("Information and Help Content").

**Correction** : La nouvelle story sera numérotée **Story 3.6** "Enrich Diagnostic Report Data".

### État actuel de l'Epic 3

| Story | Titre | Status | Implémenté |
|-------|-------|--------|------------|
| 3.1 | Diagnostics Screen Layout | Ready for Review | ✅ |
| 3.2 | Collection Status Display | Ready for Review | ✅ |
| 3.3 | Collection Toggle | Ready for Review | ✅ |
| 3.4 | Export Buttons | Approved | ✅ |
| 3.5 | Help Content | Approved | ❌ Pending |
| **3.6** | **Enrich Report Data** | **À créer** | ❌ |

### Plan de release approuvé

**Ordre de priorité décidé** : Valeur technique d'abord

```
Story 3.6 (Enrich Report Data) → Story 3.5 (Help Content) → Release Epic 3
```

**Justification** :
- Les rapports enrichis apportent une valeur diagnostique immédiate
- Le contenu d'aide est important pour l'UX mais moins critique
- Estimation totale : ~3-4 jours pour les deux stories

### Décisions validées

| Élément | Décision | Statut |
|---------|----------|--------|
| Numérotation | Story 3.6 (pas 3.5) | ✅ Corrigé |
| Regroupement | Story unique | ✅ Validé |
| Priorité | 3.6 avant 3.5 | ✅ Validé |
| Non-bloquant | Release possible sans 3.6 si urgence | ✅ Validé |
| Estimation | 1-2 jours (Story 3.6) | ✅ Validé |
| Contre-proposition cpu_brand | Adopter version architecte | ✅ Validé |

### Risques identifiés

| Risque | Probabilité | Impact | Mitigation |
|--------|-------------|--------|------------|
| `storage_type` souvent `Unknown` | Haute | Faible | Acceptable, affiné ultérieurement |
| Détection cross-platform complexe | Moyenne | Faible | Heuristique simple validée |

---

# Plan d'action final

1. [x] Valider recommandations PO (Sarah)
2. [x] Valider faisabilité technique (Winston)
3. [x] Valider priorité et plan release (John)
4. [x] **Créer Story 3.6** avec acceptance criteria intégrant les ajustements architecte
5. [x] Mettre à jour critère 3 de Story 3.4 dans la spec
6. [x] **Story 3.6 APPROVED** par PO (Sarah) — 2026-01-14
7. [ ] Implémenter Story 3.6
7. [ ] Implémenter Story 3.5 (Help Content)
8. [ ] Release Epic 3

---

# Fichiers créés/modifiés

- `docs/stories/story-3.6-enrich-report-data.md` — Story 3.6 créée
- `docs/prd/epic-3-ui-integration.md` — Story 3.6 ajoutée, critère 3.4 clarifié
