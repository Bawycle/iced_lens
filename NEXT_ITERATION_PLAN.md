# Plan d'itération : UX/UI Améliorations et Nouvelles Fonctionnalités

Ce plan couvre 4 chantiers principaux pour la prochaine itération d'IcedLens.

## Vue d'ensemble

| Chantier | Branche | Dépendances | Estimé |
|----------|---------|-------------|--------|
| A. Documentation styles | `docs/style-architecture` | Aucune | Petit |
| B. UX/UI Settings | `feature/settings-redesign` | Aucune | Moyen |
| C. UX/UI Erreurs | `feature/error-ux` | Aucune | Moyen |
| D. Rotation viewer | `feature/viewer-rotation` | Aucune | Moyen |
| E. Export de frames | `feature/frame-export` | Aucune | Moyen-Grand |

**Principe transversal : Responsive Design**
Tous les chantiers UI doivent respecter les principes de responsive design :
- Les interfaces s'adaptent aux différentes tailles de fenêtre
- Les contrôles restent accessibles et utilisables à toutes les tailles
- Les textes et icônes sont lisibles sur petits et grands écrans

**Parallélisation possible :**
- A, B, C peuvent être développés en parallèle (indépendants)
- D et E peuvent être développés en parallèle (indépendants)
- Tous les chantiers sont indépendants les uns des autres

**Stratégie de merge :**
1. Chaque branche est créée depuis `dev`
2. Chaque branche est mergée dans `dev` via squash merge une fois terminée
3. Les branches terminées sont supprimées après merge

---

## Responsive Design Guidelines

### Breakpoints

Pour une application desktop avec possibilité de redimensionnement :

| Breakpoint | Largeur | Usage |
|------------|---------|-------|
| Compact | < 600px | Fenêtre très petite, layout vertical privilégié |
| Medium | 600-900px | Fenêtre moyenne, layout adaptatif |
| Expanded | > 900px | Fenêtre large, layout complet |

### Principes d'adaptation

1. **Layout fluide** : Utiliser des pourcentages et `Length::Fill` plutôt que des tailles fixes
2. **Contrôles adaptatifs** :
   - Compact : Icônes seules, tooltips obligatoires
   - Medium/Expanded : Icônes + labels si l'espace le permet
3. **Hiérarchie visuelle** :
   - Les éléments critiques (play, pause, fermer) restent toujours visibles
   - Les éléments secondaires peuvent être masqués derrière un menu "..." en mode compact
4. **Tailles minimales** :
   - Cibles tactiles/clic : minimum 44x44px
   - Textes : minimum 12px, préférer 14px+
   - Icônes : minimum 20x20px pour les actions

### Implémentation dans Iced

```rust
// Pattern pour layout responsive
fn adaptive_layout(width: u16, content: impl Into<Element>) -> Element {
    if width < 600 {
        // Layout compact
        Column::new().push(content)
    } else {
        // Layout expanded
        Row::new().push(content)
    }
}
```

### Tests responsive

Pour chaque chantier UI, tester avec :
- Fenêtre 400x300 (compact)
- Fenêtre 800x600 (medium)
- Fenêtre 1920x1080 (expanded)
- Redimensionnement dynamique

---

## Chantier A : Documentation de l'architecture des styles

**Branche :** `docs/style-architecture`

### Contexte

L'architecture des styles (`src/ui/styles/`, `theme.rs`, `theming.rs`, `design_tokens.rs`) n'est pas documentée dans CONTRIBUTING.md, ce qui rend difficile pour les contributeurs de comprendre comment modifier ou étendre les styles.

### Tâches

#### A.1 Analyser l'architecture existante
- [ ] Documenter la responsabilité de chaque module :
  - `design_tokens.rs` : Tokens de base (couleurs, espacements, tailles)
  - `theming.rs` : Système de thèmes (ColorScheme, AppTheme, ThemeMode)
  - `theme.rs` : Fonctions utilitaires de couleurs pour le viewer/editor
  - `styles/*.rs` : Styles de composants spécifiques (boutons, containers, overlays)
- [ ] Identifier les patterns d'utilisation recommandés

#### A.2 Mettre à jour CONTRIBUTING.md
- [ ] Ajouter une section "## Style Architecture" après "## Project Structure"
- [ ] Expliquer la hiérarchie des modules
- [ ] Donner des exemples de modification/extension
- [ ] Lister les conventions (ex: utiliser `design_tokens::spacing::MD` plutôt que `16.0`)

#### A.3 Ajouter des commentaires dans le code
- [ ] Enrichir la documentation des modules si nécessaire
- [ ] Ajouter des exemples dans les doc-comments

### Critères de validation
- [ ] `cargo doc --open` génère une documentation claire
- [ ] Un nouveau contributeur peut comprendre où modifier les couleurs/espacements

---

## Chantier B : Refonte UX/UI des Settings

**Branche :** `feature/settings-redesign`

### Contexte

L'écran des paramètres devient désorganisé avec l'ajout des options vidéo. Selon les [meilleures pratiques UX](https://www.setproduct.com/blog/settings-ui-design), les paramètres doivent être regroupés par catégorie avec une hiérarchie visuelle claire.

### Design proposé

```
┌─────────────────────────────────────────────────────────────┐
│ ← Retour                                                    │
│                                                             │
│ ⚙️ Paramètres                                               │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ 🌐 GÉNÉRAL                                              │ │
│ │ ─────────────────────────────────────────────────────── │ │
│ │ Langue          [Français ▼]                            │ │
│ │ Thème           ( ) Système  (●) Clair  ( ) Sombre      │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ 🖼️ AFFICHAGE                                            │ │
│ │ ─────────────────────────────────────────────────────── │ │
│ │ Fond du viewer  ( ) Clair  (●) Sombre  ( ) Damier       │ │
│ │ Pas de zoom     [____10____] %                          │ │
│ │ Tri des médias  (●) Alpha  ( ) Date modif  ( ) Date créa│ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ 🎬 VIDÉO                                                │ │
│ │ ─────────────────────────────────────────────────────── │ │
│ │ Lecture auto    ( ) Désactivée  (●) Activée             │ │
│ │ Normalisation   (●) Activée  ( ) Désactivée             │ │
│ │ Cache frames    [═══════●═══] 128 Mo                    │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                             │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ 🎨 PLEIN ÉCRAN                                          │ │
│ │ ─────────────────────────────────────────────────────── │ │
│ │ Masquage auto   [═══●═════════] 3 secondes              │ │
│ └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Tâches

#### B.1 Refactorer la structure des sections
- [ ] Créer un composant `SettingsSection` réutilisable avec :
  - Icône (optionnelle)
  - Titre de section
  - Contenu (enfants)
- [ ] Définir les catégories : Général, Affichage, Vidéo, Plein écran

#### B.2 Réorganiser les paramètres par catégorie
- [ ] **Général** : Langue, Thème (mode)
- [ ] **Affichage** : Fond du viewer, Pas de zoom, Tri des médias
- [ ] **Vidéo** : Lecture auto, Normalisation audio, Cache frames
- [ ] **Plein écran** : Délai de masquage automatique

#### B.3 Améliorer la hiérarchie visuelle
- [ ] Ajouter des icônes aux titres de section (utiliser le module `icons.rs` existant ou en ajouter)
- [ ] Utiliser des séparateurs visuels entre sections
- [ ] Améliorer le contraste des titres de section

#### B.4 Ajouter des descriptions contextuelles
- [ ] Chaque paramètre doit avoir un hint explicatif (déjà partiellement fait)
- [ ] Vérifier que tous les hints sont traduits (fr + en-US)

#### B.5 Responsive Design
- [ ] **Compact (< 600px)** :
  - Sections empilées verticalement
  - Labels sur une ligne, contrôles sur la ligne suivante
  - Scrolling vertical si nécessaire
- [ ] **Medium/Expanded (> 600px)** :
  - Layout actuel avec labels et contrôles alignés horizontalement
  - Colonnes multiples possibles pour les radio buttons
- [ ] Utiliser `Length::Fill` pour les containers de section
- [ ] Padding adaptatif selon la taille de fenêtre

#### B.6 Tests
- [ ] Tests unitaires pour le nouveau composant `SettingsSection`
- [ ] Vérifier le rendu en mode clair et sombre
- [ ] Vérifier les traductions
- [ ] Tester aux 3 breakpoints (400x300, 800x600, 1920x1080)
- [ ] Vérifier le redimensionnement dynamique

### Fichiers impactés
- `src/ui/settings.rs` (principal)
- `src/ui/styles/container.rs` (nouveau style de section)
- `assets/i18n/en-US.ftl`
- `assets/i18n/fr.ftl`

### Critères de validation
- [ ] Les paramètres sont clairement regroupés par catégorie
- [ ] L'écran reste lisible avec 4+ catégories
- [ ] Tous les textes sont traduits
- [ ] `cargo clippy` sans warnings

---

## Chantier C : Amélioration UX/UI des erreurs

**Branche :** `feature/error-ux`

### Contexte

Actuellement, les erreurs sont affichées de manière basique (texte rouge). Selon [Nielsen Norman Group](https://www.nngroup.com/articles/indicators-validations-notifications/) et les [bonnes pratiques](https://www.pencilandpaper.io/articles/ux-pattern-analysis-error-feedback), les erreurs doivent :
- Être visibles et non-intrusives
- Expliquer clairement le problème
- Proposer une action si possible
- Rester à l'écran tant que l'utilisateur n'a pas agi (pas de toast pour les erreurs)

### Design proposé

**Erreur inline (dans le viewer) :**
```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│                    ⚠️                                       │
│                                                             │
│           Impossible de charger le fichier                  │
│                                                             │
│   Le format n'est pas supporté ou le fichier est corrompu   │
│                                                             │
│              [ Choisir un autre fichier ]                   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Erreur vidéo (pendant la lecture) :**
```
┌─────────────────────────────────────────────────────────────┐
│                   [thumbnail floue]                         │
│                                                             │
│         ┌─────────────────────────────────┐                 │
│         │  ⚠️ Erreur de lecture           │                 │
│         │  Codec non supporté: HEVC       │                 │
│         │  [ Réessayer ]                  │                 │
│         └─────────────────────────────────┘                 │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Tâches

#### C.1 Créer un composant ErrorDisplay réutilisable
- [ ] Props : icône, titre, message détaillé, action optionnelle
- [ ] Styles cohérents avec le design system (utiliser `design_tokens`)
- [ ] Variantes : erreur critique (rouge), warning (orange), info (bleu)

#### C.2 Améliorer les messages d'erreur
- [ ] Auditer tous les messages d'erreur existants
- [ ] S'assurer qu'ils sont explicites et actionnables
- [ ] Ajouter les traductions manquantes

#### C.3 Intégrer ErrorDisplay dans le viewer
- [ ] Remplacer l'affichage d'erreur actuel pour les images
- [ ] Remplacer l'affichage d'erreur actuel pour les vidéos
- [ ] Gérer les erreurs de chargement de fichier

#### C.4 Gérer les erreurs vidéo spécifiques
- [ ] Erreur de décodage
- [ ] Erreur audio
- [ ] Timeout de chargement

#### C.5 Responsive Design
- [ ] **ErrorDisplay adaptatif** :
  - Compact : Icône + message court, détails en tooltip
  - Expanded : Icône + titre + message détaillé + action
- [ ] **Overlay d'erreur vidéo** :
  - S'adapte à la taille de la vidéo
  - Texte tronqué avec "..." si nécessaire
  - Boutons d'action restent toujours accessibles
- [ ] Taille minimale du composant : 200x100px
- [ ] Texte d'erreur avec word-wrap

#### C.6 Tests
- [ ] Tests unitaires pour ErrorDisplay
- [ ] Tests d'intégration avec des fichiers corrompus
- [ ] Tester aux 3 breakpoints
- [ ] Vérifier la lisibilité sur petits écrans

### Fichiers impactés
- `src/ui/components/` (nouveau : `error_display.rs`)
- `src/ui/viewer/component.rs`
- `src/ui/viewer/pane.rs`
- `src/ui/styles/` (nouveau style pour erreurs si nécessaire)
- `assets/i18n/*.ftl`

### Critères de validation
- [ ] Les erreurs sont visuellement distinctes et lisibles
- [ ] Chaque erreur propose une action (si applicable)
- [ ] Les messages sont traduits
- [ ] Le composant s'adapte à différentes tailles de fenêtre
- [ ] `cargo clippy` sans warnings

---

## Chantier D : Rotation temporaire dans le viewer

**Branche :** `feature/viewer-rotation`

### Contexte

Permettre à l'utilisateur de faire pivoter temporairement une image ou vidéo sans modifier le fichier source. La rotation est persistante par session (conservée tant que l'application est ouverte ou jusqu'au changement de média).

### Spécifications
- **Incréments** : 90° uniquement (0°, 90°, 180°, 270°)
- **Persistance** : Session uniquement (reset au changement de média)
- **Média supportés** : Images et vidéos
- **Raccourcis clavier** : `R` (rotation horaire), `Shift+R` (anti-horaire)

### Design proposé

**Boutons dans la toolbar du viewer :**
```
┌─────────────────────────────────────────────────────────────┐
│ [🔄↺] [🔄↻]  |  [🔍+] [🔍-] [Fit] [1:1]  |  [⛶]  |  [⚙️] │
│  Rotation     |        Zoom              | Full |  Settings │
└─────────────────────────────────────────────────────────────┘
```

### Tâches

#### D.1 Ajouter l'état de rotation au viewer
- [ ] Ajouter `rotation_degrees: u16` (0, 90, 180, 270) dans `ViewerState`
- [ ] Réinitialiser à 0 lors du changement de média

#### D.2 Implémenter la rotation visuelle des images
- [ ] Appliquer une transformation CSS/canvas à l'image affichée
- [ ] Adapter le calcul de fit-to-window pour les dimensions pivotées

#### D.3 Implémenter la rotation visuelle des vidéos
- [ ] Appliquer la rotation au canvas vidéo
- [ ] S'assurer que les contrôles restent bien positionnés

#### D.4 Ajouter les contrôles UI
- [ ] Ajouter les icônes de rotation (`rotate_left.svg`, `rotate_right.svg` - déjà présentes)
- [ ] Ajouter les boutons dans la toolbar du viewer (mode normal et fullscreen)
- [ ] Implémenter les raccourcis clavier `R` et `Shift+R`

#### D.5 Gérer les interactions
- [ ] Le zoom et le pan doivent fonctionner correctement avec la rotation
- [ ] Le fit-to-window doit recalculer selon les dimensions pivotées

#### D.6 Responsive Design
- [ ] **Boutons de rotation dans la toolbar** :
  - Compact : Intégrés dans le menu "..." ou icônes seules
  - Expanded : Boutons visibles avec tooltips
- [ ] **Position des boutons** :
  - Reste accessible quelle que soit la taille de fenêtre
  - Ne masque pas le contenu de l'image/vidéo
- [ ] **Fullscreen** : Boutons accessibles dans l'overlay fullscreen

#### D.7 Tests
- [ ] Tests unitaires pour la logique de rotation
- [ ] Tests d'intégration rotation + zoom + pan
- [ ] Tester aux 3 breakpoints
- [ ] Vérifier en mode fullscreen

### Fichiers impactés
- `src/ui/viewer/state.rs`
- `src/ui/viewer/component.rs`
- `src/ui/viewer/pane.rs`
- `src/ui/viewer/controls.rs`
- `src/ui/widgets/video_canvas.rs`
- `assets/i18n/*.ftl` (tooltips)

### Critères de validation
- [ ] La rotation fonctionne pour images et vidéos
- [ ] Le fit-to-window s'adapte aux dimensions pivotées
- [ ] Les raccourcis clavier fonctionnent
- [ ] La rotation est réinitialisée au changement de média
- [ ] Les contrôles s'adaptent à la taille de fenêtre
- [ ] `cargo clippy` sans warnings

---

## Chantier E : Export de frames vidéo

**Branche :** `feature/frame-export`

### Contexte

Permettre d'exporter la frame actuelle d'une vidéo ou d'un GIF/WebP animé en tant qu'image. Inclut aussi un mode avancé de navigation frame par frame.

### Spécifications
- **Déclencheur** : Bouton "Capturer" accessible en lecture, pause, et preview
- **Navigation frame par frame** : Accessible en pause via des contrôles dédiés (moins proéminent)
- **Formats d'export** : PNG (défaut), JPEG, WebP
- **Dialogue** : Utiliser `rfd` pour le choix du fichier de destination

### Design proposé

**Contrôles vidéo avec bouton capture :**
```
┌─────────────────────────────────────────────────────────────┐
│  [⏮] [⏪] [▶️/⏸] [⏩] [⏭]   [═══════●═══════]   [📷] [🔊] [🔁] │
│                                 Seekbar         Capture     │
└─────────────────────────────────────────────────────────────┘
```

**Mode pause avec navigation frame par frame :**
```
┌─────────────────────────────────────────────────────────────┐
│  [⏮] [◀️] [⏸] [▶️] [⏭]   [═══════●═══════]   [📷] [🔊] [🔁] │
│        ↑frame  ↑frame                        Capture        │
│       -1      +1                                            │
└─────────────────────────────────────────────────────────────┘
```

Ou via menu contextuel / bouton "..." pour les contrôles avancés :
```
┌───────────────────┐
│ ◀️ Frame -1       │
│ ▶️ Frame +1       │
│ ───────────────── │
│ 📷 Exporter frame │
└───────────────────┘
```

### Tâches

#### E.1 Implémenter la capture de frame
- [ ] Extraire la frame RGBA actuelle du décodeur vidéo
- [ ] Convertir en format image (PNG/JPEG/WebP) via le crate `image`
- [ ] Ouvrir le dialogue de sauvegarde avec `rfd`

#### E.2 Ajouter le bouton capture dans les contrôles vidéo
- [ ] Créer l'icône `camera.svg` ou utiliser une existante
- [ ] Ajouter le bouton dans `video_controls.rs`
- [ ] Gérer le message `CaptureFrame`

#### E.3 Implémenter le dialogue d'export
- [ ] Utiliser `rfd::FileDialog::new().add_filter("PNG", &["png"])...`
- [ ] Proposer les formats : PNG, JPEG, WebP
- [ ] Nom par défaut : `{nom_video}_frame_{timestamp}.png`

#### E.4 Implémenter la navigation frame par frame
- [ ] Ajouter les commandes `StepForward` et `StepBackward` au décodeur
- [ ] Modifier le décodeur FFmpeg pour supporter le step (seek au frame suivant/précédent)
- [ ] Modifier le décodeur WebP pour supporter le step
- [ ] Afficher les contrôles frame par frame uniquement en pause

#### E.5 UI pour les contrôles avancés
- [ ] Option 1 : Boutons visibles uniquement en pause
- [ ] Option 2 : Menu "..." avec options avancées
- [ ] Raccourcis clavier : `,` (frame -1), `.` (frame +1) - comme dans VLC

#### E.6 Feedback utilisateur
- [ ] Afficher un toast/notification de succès après export
- [ ] Gérer les erreurs d'écriture (permissions, espace disque)

#### E.7 Responsive Design
- [ ] **Bouton capture** :
  - Toujours visible dans les contrôles vidéo (priorité haute)
  - Icône seule en mode compact, avec label en mode expanded
- [ ] **Contrôles frame par frame** :
  - Compact : Masqués derrière menu "..." ou accessibles uniquement par raccourcis clavier
  - Expanded : Boutons visibles en mode pause
- [ ] **Barre de contrôles vidéo** :
  - S'adapte à la largeur disponible
  - Boutons essentiels (play/pause, seekbar, volume) prioritaires
  - Boutons secondaires (capture, loop) dans overflow si nécessaire
- [ ] **Dialogue d'export** : Géré par `rfd`, responsive natif du système

#### E.8 Tests
- [ ] Tests unitaires pour l'extraction de frame
- [ ] Tests d'intégration avec différents formats vidéo
- [ ] Tests du dialogue d'export (mock si nécessaire)
- [ ] Tester les contrôles aux 3 breakpoints
- [ ] Vérifier l'accessibilité des contrôles en fullscreen

### Fichiers impactés
- `src/video_player/decoder.rs` (nouvelles commandes)
- `src/video_player/webp_decoder.rs` (nouvelles commandes)
- `src/video_player/state.rs` (gestion du step)
- `src/ui/viewer/video_controls.rs` (nouveaux boutons)
- `src/ui/viewer/component.rs` (handler capture)
- Nouveau : `src/media/frame_export.rs`
- `assets/icons/camera.svg` (si nécessaire)
- `assets/i18n/*.ftl`

### Critères de validation
- [ ] L'export fonctionne pour MP4, MKV, WebM, GIF animé, WebP animé
- [ ] Les 3 formats d'export sont disponibles (PNG, JPEG, WebP)
- [ ] La navigation frame par frame fonctionne en pause
- [ ] Le dialogue de sauvegarde propose un nom par défaut pertinent
- [ ] Les contrôles s'adaptent aux différentes tailles de fenêtre
- [ ] `cargo clippy` sans warnings

---

## Workflow Git

### Création des branches

```bash
# Depuis dev, créer chaque branche
git checkout dev
git pull

git checkout -b docs/style-architecture
# ... travail ...
git checkout dev

git checkout -b feature/settings-redesign
# ... travail ...
git checkout dev

# etc.
```

### Merge vers dev

```bash
# Une fois une branche terminée
git checkout dev
git pull
git merge --squash feature/settings-redesign
git commit -m "feat(settings): Redesign settings page with categorized sections"
git push

# Supprimer la branche locale et distante
git branch -D feature/settings-redesign
git push origin --delete feature/settings-redesign
```

### Ordre suggéré (non obligatoire)

1. **A** (Documentation) - Rapide, clarifie l'architecture pour les autres chantiers
2. **B et C** en parallèle - Indépendants, améliorent l'UX globale
3. **D et E** en parallèle - Fonctionnalités indépendantes

---

## Checklist globale avant merge

**Important** : Toutes les contributions doivent respecter les directives de [CONTRIBUTING.md](CONTRIBUTING.md), notamment :
- Test-Driven Development (TDD) : écrire les tests avant ou avec l'implémentation
- Conventional commits pour les messages de commit
- Code review et PR process

Pour chaque branche, avant le merge :

- [ ] `cargo test` passe
- [ ] `cargo clippy --all --all-targets -- -D warnings` passe
- [ ] `cargo fmt --all` appliqué
- [ ] Traductions ajoutées (en-US + fr)
- [ ] CHANGELOG.md mis à jour si nécessaire
- [ ] Documentation mise à jour si nécessaire
- [ ] **Tests responsive** : Vérifier le rendu aux 3 breakpoints (400x300, 800x600, 1920x1080)
- [ ] **Tests redimensionnement** : Vérifier le comportement lors du redimensionnement dynamique

---

## Références

### UX/UI Erreurs
- [NN/G - Indicators, Validations, and Notifications](https://www.nngroup.com/articles/indicators-validations-notifications/)
- [Error Message UX - Pencil & Paper](https://www.pencilandpaper.io/articles/ux-pattern-analysis-error-feedback)
- [Toast Notifications Best Practices - LogRocket](https://blog.logrocket.com/ux-design/toast-notifications/)

### UX/UI Settings
- [App Settings UI Design - SetProduct](https://www.setproduct.com/blog/settings-ui-design)
- [Settings UX - Toptal](https://www.toptal.com/designers/ux/settings-ux)
- [Designing Settings Screen UI - LogRocket](https://blog.logrocket.com/ux-design/designing-settings-screen-ui/)

### Video Player UX
- [Video Player Controls UX - Vidzflow](https://www.vidzflow.com/blog/mastering-video-player-controls-ux-best-practices)

### Responsive Design
- [Responsive UI Design Principles - Material Design](https://m3.material.io/foundations/layout/understanding-layout)
- [Adaptive Layouts - Microsoft Fluent](https://learn.microsoft.com/en-us/windows/apps/design/layout/responsive-design)
- [Touch Target Guidelines - WCAG](https://www.w3.org/WAI/WCAG21/Understanding/target-size.html)
