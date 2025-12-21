# Terme pour le nom de l'application - source unique pour la marque
-app-name = IcedLens

window-title = { -app-name }
new-image-title = Nouvelle image
settings-back-to-viewer-button = Retour
settings-title = Paramètres
settings-section-general = Général
settings-section-display = Affichage
settings-section-video = Vidéo
settings-section-fullscreen = Plein écran
settings-section-ai = IA / Apprentissage automatique
select-language-label = Sélectionner la langue :
language-name-en-US = Anglais
language-name-fr = Français
language-name-es = Espagnol
language-name-de = Allemand
language-name-it = Italien
error-load-image-heading = Impossible d'ouvrir l'image.
error-details-show = Afficher les détails
error-details-hide = Masquer les détails
error-details-technical-heading = Détails techniques
viewer-zoom-label = Zoom
viewer-zoom-input-placeholder = 100
viewer-zoom-reset-button = Réinitialiser
viewer-fit-to-window-toggle = Adapter à la fenêtre
viewer-zoom-input-error-invalid = Veuillez saisir un nombre valide.
viewer-zoom-step-error-invalid = L'incrément de zoom doit être un nombre.
viewer-zoom-step-error-range = L'incrément de zoom doit être compris entre 1 % et 200 %.
viewer-delete-tooltip = Supprimer l'image affichée
viewer-zoom-in-tooltip = Zoom avant
viewer-zoom-out-tooltip = Zoom arrière
viewer-fullscreen-tooltip = Basculer en plein écran
viewer-fullscreen-disabled-unsaved = Enregistrez ou annulez d'abord les modifications
viewer-double-click = Double-clic
viewer-scroll-wheel = Molette
viewer-click-drag = Clic + glisser
settings-zoom-step-label = Incrément de zoom
settings-zoom-step-placeholder = 10
settings-zoom-step-hint = Définissez la variation appliquée lors des contrôles de zoom (de 1 % à 200 %).
settings-background-label = Fond de la visionneuse
settings-background-light = Clair
settings-background-dark = Sombre
settings-background-checkerboard = Damier
settings-theme-mode-label = Thème de l'application
settings-theme-system = Suivre le système
settings-theme-light = Clair
settings-theme-dark = Sombre
help-usage-heading = UTILISATION :
help-options-heading = OPTIONS :
help-args-heading = ARGUMENTS :
help-examples-heading = EXEMPLES :
help-line-option-help = -h, --help        Afficher cette aide
help-line-option-lang =     --lang <id>    Définir la langue (ex. en-US, fr)
help-arg-image-path = <CHEMIN>      Chemin vers un fichier média ou un répertoire à ouvrir
help-example-1 = iced_lens ./photo.png
help-example-2 = iced_lens ./mes_photos/
help-example-3 = iced_lens --lang fr ./image.jpg
help-description = { -app-name } – Visionneuse d'images
help-line-option-i18n-dir =     --i18n-dir <chemin>  Charger les traductions depuis un dossier
help-line-option-data-dir =     --data-dir <chemin>  Remplacer le répertoire de données (fichiers d'état)
help-line-option-config-dir =     --config-dir <chemin>  Remplacer le répertoire de config (settings.toml)
settings-sort-order-label = Ordre de tri pour la navigation
settings-sort-alphabetical = Alphabétique
settings-sort-modified = Date de modification
settings-sort-created = Date de création
settings-max-skip-attempts-label = Ignorer les fichiers corrompus
settings-max-skip-attempts-hint = Nombre max de fichiers corrompus à ignorer lors de la navigation.
settings-overlay-timeout-label = Délai de masquage automatique en plein écran
settings-overlay-timeout-hint = Durée avant la disparition des contrôles en mode plein écran.
seconds = secondes
image-editor-title = Éditeur d'image
image-editor-back-to-viewer = Retour
image-editor-cancel = Annuler
image-editor-save = Enregistrer
image-editor-save-as = Enregistrer sous...
image-editor-tool-crop = Rogner
image-editor-tool-resize = Redimensionner
image-editor-tool-light = Lumière
image-editor-rotate-section-title = Rotation
image-editor-rotate-right-tooltip = Tourner l'image dans le sens horaire
image-editor-rotate-left-tooltip = Tourner l'image dans le sens antihoraire
image-editor-flip-section-title = Retournement
image-editor-flip-horizontal-tooltip = Retourner l'image horizontalement (miroir gauche-droite)
image-editor-flip-vertical-tooltip = Retourner l'image verticalement (miroir haut-bas)
image-editor-resize-section-title = Redimensionner
image-editor-resize-scale-label = Échelle
image-editor-resize-dimensions-label = Dimensions cibles
image-editor-resize-width-label = Largeur (px)
image-editor-resize-height-label = Hauteur (px)
image-editor-resize-lock-aspect = Conserver les proportions
image-editor-resize-presets-label = Préréglages
image-editor-resize-apply = Appliquer le redimensionnement
image-editor-resize-preview-label = Aperçu
image-editor-resize-ai-upscale = Upscaling IA (Real-ESRGAN)
image-editor-resize-ai-model-not-downloaded = Modèle IA non téléchargé
image-editor-resize-ai-model-downloading = Téléchargement du modèle IA
image-editor-resize-ai-model-validating = Validation du modèle IA
image-editor-resize-ai-model-error = Erreur du modèle IA
image-editor-resize-ai-enlargement-only = L'upscaling IA ne s'applique qu'aux agrandissements
image-editor-light-section-title = Ajustements de lumière
image-editor-light-brightness-label = Luminosité
image-editor-light-contrast-label = Contraste
image-editor-light-reset = Réinitialiser
image-editor-light-apply = Appliquer
image-editor-crop-section-title = Rogner
image-editor-crop-ratio-label = Ratio d'aspect
image-editor-crop-ratio-free = Libre
image-editor-crop-ratio-square = Carré (1:1)
image-editor-crop-ratio-landscape = Paysage (16:9)
image-editor-crop-ratio-portrait = Portrait (9:16)
image-editor-crop-ratio-photo = Photo (4:3)
image-editor-crop-ratio-photo-portrait = Photo Portrait (3:4)
image-editor-crop-apply = Appliquer le rognage
image-editor-undo-redo-section-title = Dernière modification
image-editor-undo = Annuler
image-editor-redo = Rétablir
image-editor-export-format-label = Format d'export
media-loading = Chargement...
settings-video-autoplay-label = Lecture automatique des vidéos
settings-video-autoplay-enabled = Activée
settings-video-autoplay-disabled = Désactivée
settings-video-autoplay-hint = Lorsque activée, les vidéos démarrent automatiquement à l'ouverture.
video-play-tooltip = Lecture (Espace)
video-pause-tooltip = Pause (Espace)
video-mute-tooltip = Couper le son (M)
video-unmute-tooltip = Remettre le son (M)
video-loop-tooltip = Boucle
video-capture-tooltip = Capturer l'image actuelle
video-step-forward-tooltip = Avancer d'une image (.)
video-step-backward-tooltip = Reculer d'une image (,)
video-more-tooltip = Plus d'options
video-speed-down-tooltip = Réduire la vitesse (J)
video-speed-up-tooltip = Augmenter la vitesse (L)
hud-video-no-audio = Pas de son
settings-audio-normalization-label = Normalisation du volume audio
settings-audio-normalization-enabled = Activée
settings-audio-normalization-disabled = Désactivée
settings-audio-normalization-hint = Nivelle automatiquement le volume entre les différents médias pour éviter les changements brusques de volume.
settings-frame-cache-label = Taille du cache de keyframes (pour la navigation)
settings-frame-cache-hint = Met en cache les images-clés (keyframes) pour accélérer la navigation dans la timeline et les sauts à un moment précis. Des valeurs plus élevées stockent plus de keyframes pour une navigation plus fluide. Les changements s'appliquent à l'ouverture d'une nouvelle vidéo.
settings-frame-history-label = Taille de l'historique (pour reculer image par image)
settings-frame-history-hint = Conserve les images récemment affichées pour permettre de reculer image par image. Utilisée uniquement lors du défilement manuel des images, pas pendant la lecture normale.
settings-keyboard-seek-step-label = Pas de navigation au clavier
settings-keyboard-seek-step-hint = Durée à sauter avec les touches fléchées pendant la lecture vidéo.
megabytes = Mo
error-load-video-heading = Impossible de lire cette vidéo.
error-load-video-general = Une erreur est survenue lors du chargement de la vidéo.
error-load-video-unsupported-format = Ce format de fichier n'est pas pris en charge.
error-load-video-unsupported-codec = Le codec vidéo « { $codec } » n'est pas pris en charge sur ce système.
error-load-video-corrupted = Le fichier vidéo semble corrompu ou invalide.
error-load-video-no-video-stream = Aucune piste vidéo n'a été trouvée dans ce fichier.
error-load-video-decoding-failed = Échec du décodage vidéo : { $message }
error-load-video-io = Impossible de lire ce fichier. Vérifiez qu'il existe et que vous disposez des permissions nécessaires.

# Barre de navigation
menu-settings = Paramètres
menu-help = Aide
menu-about = À propos
navbar-edit-button = Éditer

# Écran d'aide
help-title = Aide
help-back-to-viewer-button = Retour

# Libellés communs
help-tools-title = Outils disponibles
help-shortcuts-title = Raccourcis clavier
help-usage-title = Mode d'emploi

# ─────────────────────────────────────────────────────────────────────────────
# Section Visionneuse
# ─────────────────────────────────────────────────────────────────────────────
help-section-viewer = Visionneuse d'images et vidéos
help-viewer-role = Parcourez et visualisez vos images et vidéos. Naviguez entre les fichiers du même dossier et ajustez l'affichage selon vos préférences.

help-viewer-tool-navigation = Navigation
help-viewer-tool-navigation-desc = Utilisez les flèches ou le clavier pour passer d'un fichier à l'autre.
help-viewer-tool-zoom = Zoom
help-viewer-tool-zoom-desc = Molette de souris, boutons +/-, ou entrez un pourcentage directement.
help-viewer-tool-pan = Déplacement
help-viewer-tool-pan-desc = Lorsque l'image est zoomée, cliquez et faites glisser pour vous déplacer.
help-viewer-tool-fit = Adapter à la fenêtre
help-viewer-tool-fit-desc = Ajuste automatiquement l'image pour qu'elle tienne entièrement dans la fenêtre.
help-viewer-tool-fullscreen = Plein écran
help-viewer-tool-fullscreen-desc = Vue immersive avec masquage automatique des contrôles (délai configurable dans Paramètres).
help-viewer-tool-delete = Supprimer
help-viewer-tool-delete-desc = Supprime définitivement le fichier (déplacé vers la corbeille si disponible).

help-viewer-key-navigate = Passer au fichier précédent/suivant
help-viewer-key-edit = Ouvrir l'image dans l'éditeur
help-viewer-key-fullscreen = Entrer/quitter le plein écran
help-viewer-key-exit-fullscreen = Quitter le mode plein écran
help-viewer-key-info = Afficher/masquer le panneau d'informations

help-mouse-title = Interactions souris
help-viewer-mouse-doubleclick = Double-clic sur l'image/vidéo pour basculer en plein écran
help-viewer-mouse-wheel = Zoomer/dézoomer
help-viewer-mouse-drag = Déplacer l'image lorsqu'elle est zoomée

# ─────────────────────────────────────────────────────────────────────────────
# Section Lecture vidéo
# ─────────────────────────────────────────────────────────────────────────────
help-section-video = Lecture vidéo
help-video-role = Lisez vos vidéos avec des contrôles complets. Réglez le volume, naviguez dans la timeline et avancez image par image pour un positionnement précis.

help-video-tool-playback = Lecture/Pause
help-video-tool-playback-desc = Démarrez ou arrêtez la lecture avec le bouton ou la touche Espace.
help-video-tool-timeline = Timeline
help-video-tool-timeline-desc = Cliquez n'importe où sur la barre de progression pour sauter à cette position.
help-video-tool-volume = Volume
help-video-tool-volume-desc = Glissez le curseur de volume (0-150%) ou cliquez sur l'icône haut-parleur pour couper/remettre le son.
help-video-tool-loop = Boucle
help-video-tool-loop-desc = Activez pour redémarrer automatiquement la vidéo à la fin.
help-video-tool-stepping = Navigation image par image
help-video-tool-stepping-desc = En pause, avancez ou reculez d'une seule image pour une navigation précise.
help-video-tool-capture = Capture d'image
help-video-tool-capture-desc = Enregistrez l'image vidéo actuelle comme fichier image (s'ouvre dans l'éditeur).

help-video-key-playpause = Lire ou mettre en pause la vidéo
help-video-key-mute = Activer/désactiver le son
help-video-key-seek = Avancer/reculer dans la vidéo (pendant la lecture)
help-video-key-volume = Augmenter/diminuer le volume
help-video-key-step-back = Reculer d'une image (en pause)
help-video-key-step-forward = Avancer d'une image (en pause)
help-video-key-speed-down = Réduire la vitesse de lecture
help-video-key-speed-up = Augmenter la vitesse de lecture

# ─────────────────────────────────────────────────────────────────────────────
# Section Éditeur d'images
# ─────────────────────────────────────────────────────────────────────────────
help-section-editor = Éditeur d'images
help-editor-role = Retouchez vos images : rotation, rognage d'une zone spécifique, ou redimensionnement.
help-editor-workflow = Toutes les modifications sont non-destructives jusqu'à l'enregistrement. Utilisez « Enregistrer » pour écraser l'original, ou « Enregistrer sous » pour créer un nouveau fichier.

help-editor-rotate-title = Rotation
help-editor-rotate-desc = Pivotez ou retournez l'image pour corriger l'orientation ou créer des effets miroir.
help-editor-rotate-left = Pivoter de 90° dans le sens antihoraire
help-editor-rotate-right = Pivoter de 90° dans le sens horaire
help-editor-flip-h = Retourner horizontalement (miroir gauche/droite)
help-editor-flip-v = Retourner verticalement (miroir haut/bas)

help-editor-crop-title = Rognage
help-editor-crop-desc = Supprimez les zones indésirables en sélectionnant la région à conserver.
help-editor-crop-ratios = Choisissez un ratio prédéfini (1:1 carré, 16:9 paysage, 9:16 portrait, 4:3 ou 3:4 photo) ou rognez librement.
help-editor-crop-usage = Faites glisser les poignées pour ajuster la sélection, puis cliquez « Appliquer » pour confirmer.

help-editor-resize-title = Redimensionnement
help-editor-resize-desc = Modifiez les dimensions de l'image pour l'agrandir ou la réduire.
help-editor-resize-scale = Échelle en pourcentage (10% à 400%)
help-editor-resize-dimensions = Entrez la largeur et la hauteur exactes en pixels
help-editor-resize-lock = Verrouillez le ratio pour conserver les proportions
help-editor-resize-presets = Utilisez les préréglages pour un redimensionnement rapide (25%, 50%, 200%, etc.)
help-editor-resize-ai-upscale = Upscaling IA : Utilisez Real-ESRGAN pour des agrandissements plus nets (activer dans Paramètres)

help-editor-light-title = Lumière
help-editor-light-desc = Ajustez la luminosité et le contraste de votre image.
help-editor-light-brightness = Luminosité : éclaircir ou assombrir l'image
help-editor-light-contrast = Contraste : augmenter ou réduire la différence entre zones claires et sombres
help-editor-light-preview = Les modifications sont prévisualisées en temps réel avant application

help-editor-save-title = Enregistrement
help-editor-save-overwrite = Enregistrer : écrase le fichier original
help-editor-save-as = Enregistrer sous : crée un nouveau fichier (choisissez l'emplacement et le format)

help-editor-key-save = Enregistrer les modifications
help-editor-key-undo = Annuler la dernière modification
help-editor-key-redo = Rétablir la modification annulée
help-editor-key-cancel = Annuler toutes les modifications et quitter

help-editor-mouse-title = Contrôles souris
help-editor-mouse-wheel = Zoomer ou dézoomer l'image
help-editor-mouse-drag = Déplacer l'image lorsque zoomée

# ─────────────────────────────────────────────────────────────────────────────
# Section Capture d'image vidéo
# ─────────────────────────────────────────────────────────────────────────────
help-section-capture = Capture d'image vidéo
help-capture-role = Extrayez n'importe quelle image d'une vidéo et enregistrez-la comme fichier image. Idéal pour créer des miniatures ou capturer des moments précis.

help-capture-step1 = Lisez ou naviguez dans la vidéo jusqu'à l'image souhaitée
help-capture-step2 = Mettez en pause (utilisez la navigation image par image pour plus de précision)
help-capture-step3 = Cliquez sur le bouton caméra dans les contrôles vidéo
help-capture-step4 = L'image s'ouvre dans l'éditeur — enregistrez en PNG, JPEG ou WebP

help-capture-formats = Formats d'export disponibles : PNG (sans perte), JPEG (fichier plus léger), WebP (format moderne avec bonne compression).

# ─────────────────────────────────────────────────────────────────────────────
# Section Édition des métadonnées
# ─────────────────────────────────────────────────────────────────────────────
help-section-metadata = Édition des métadonnées
help-metadata-role = Consultez et modifiez les métadonnées EXIF intégrées dans vos fichiers image. Modifiez les informations de l'appareil photo, la date de prise de vue, les coordonnées GPS et les paramètres d'exposition.

help-metadata-tool-view = Mode affichage
help-metadata-tool-view-desc = Consultez les informations du fichier, les détails de l'appareil, les paramètres d'exposition et les coordonnées GPS dans le panneau d'informations.
help-metadata-tool-edit = Mode édition
help-metadata-tool-edit-desc = Cliquez sur le bouton Éditer pour modifier les champs de métadonnées. Les modifications sont validées en temps réel.
help-metadata-tool-save = Options d'enregistrement
help-metadata-tool-save-desc = Enregistrer pour mettre à jour le fichier original, ou Enregistrer sous pour créer une copie avec les nouvelles métadonnées.

help-metadata-fields-title = Champs modifiables
help-metadata-field-camera = Marque et modèle de l'appareil
help-metadata-field-date = Date de prise de vue (format EXIF)
help-metadata-field-exposure = Temps d'exposition, ouverture, ISO
help-metadata-field-focal = Focale et équivalent 35mm
help-metadata-field-gps = Latitude et longitude GPS

help-metadata-note = Note : L'édition des métadonnées n'est disponible que pour les images. L'édition des métadonnées vidéo est prévue pour une future version.

# Écran À propos
about-title = À propos
about-back-to-viewer-button = Retour

about-section-app = Application
about-app-name = { -app-name }
about-app-description = Visionneuse d'images et de vidéos légère avec édition basique d'images.

about-section-license = Licence
about-license-name = Mozilla Public License 2.0 (MPL-2.0)
about-license-summary = Copyleft au niveau des fichiers : les fichiers modifiés doivent être partagés sous la même licence. Compatible avec le code propriétaire.

about-section-icon-license = Licence des icônes
about-icon-license-name = Licence des icônes { -app-name }
about-icon-license-summary = Toutes les icônes (logo et icônes d'interface) ne peuvent être redistribuées que sans modification pour représenter { -app-name }.

about-section-credits = Crédits
about-credits-iced = Développé avec la bibliothèque Iced
about-credits-ffmpeg = Lecture vidéo propulsée par FFmpeg
about-credits-onnx = Défloutage IA propulsé par ONNX Runtime
about-credits-fluent = Internationalisation par Project Fluent
about-credits-full-list = Voir la liste complète des dépendances

about-section-third-party = Licences tierces
about-third-party-ffmpeg = FFmpeg est distribué sous licence LGPL 2.1+
about-third-party-onnx = ONNX Runtime et DirectML sont distribués sous licence MIT
about-third-party-details = Voir THIRD_PARTY_LICENSES.md pour les détails

about-section-links = Liens
about-link-repository = Code source
about-link-issues = Signaler un problème

# Notifications
notification-save-success = Image enregistrée avec succès
notification-save-error = Échec de l'enregistrement de l'image
notification-frame-capture-success = Image capturée avec succès
notification-frame-capture-error = Échec de la capture d'image
notification-delete-success = Fichier supprimé avec succès
notification-delete-error = Échec de la suppression du fichier
notification-config-save-error = Échec de l'enregistrement des paramètres
notification-config-load-error = Échec du chargement des paramètres, valeurs par défaut utilisées
notification-state-parse-error = Échec de lecture de l'état, valeurs par défaut utilisées
notification-state-read-error = Impossible d'ouvrir le fichier d'état
notification-state-path-error = Impossible de déterminer le chemin des données
notification-state-dir-error = Impossible de créer le dossier de données
notification-state-write-error = Échec de l'enregistrement de l'état
notification-state-create-error = Impossible de créer le fichier d'état
notification-scan-dir-error = Échec de l'analyse du dossier
notification-editor-frame-error = Impossible d'ouvrir l'éditeur avec l'image capturée
notification-editor-create-error = Impossible d'ouvrir l'éditeur d'images
notification-editor-load-error = Impossible de charger l'image pour l'édition
notification-video-editing-unsupported = L'édition vidéo n'est pas encore supportée

# Panneau de métadonnées
metadata-panel-title = Informations du fichier
metadata-panel-close = Fermer le panneau
metadata-panel-close-disabled = Enregistrez ou annulez d'abord les modifications
metadata-section-file = Fichier
metadata-section-camera = Appareil photo
metadata-section-exposure = Exposition
metadata-section-video = Vidéo
metadata-section-audio = Audio
metadata-section-gps = Localisation
metadata-label-dimensions = Dimensions
metadata-label-file-size = Taille du fichier
metadata-label-format = Format
metadata-label-date-taken = Date de prise de vue
metadata-label-camera = Appareil
metadata-label-exposure = Exposition
metadata-label-aperture = Ouverture
metadata-label-iso = ISO
metadata-label-focal-length = Focale
metadata-label-gps = Coordonnées
metadata-label-codec = Codec
metadata-label-bitrate = Débit
metadata-label-duration = Durée
metadata-label-fps = Images/seconde
metadata-value-unknown = Inconnu

# Édition des métadonnées
metadata-edit-button = Éditer
metadata-edit-disabled-video = L'édition des métadonnées n'est pas disponible pour les vidéos
metadata-cancel-button = Annuler
metadata-save-button = Enregistrer
metadata-save-as-button = Enregistrer sous...
metadata-save-warning = Enregistrer modifiera le fichier original
metadata-label-make = Marque
metadata-label-model = Modèle
metadata-label-focal-length-35mm = Focale (35mm)
metadata-label-flash = Flash
metadata-label-latitude = Latitude
metadata-label-longitude = Longitude
metadata-validation-date-format = Format : AAAA:MM:JJ HH:MM:SS
metadata-validation-date-invalid = Valeurs de date/heure invalides
metadata-date-placeholder = AAAA-MM-JJ HH:MM:SS
metadata-date-now = Maintenant
metadata-date-help = Accepte : AAAA-MM-JJ, JJ/MM/AAAA, etc.
metadata-validation-exposure-format = Format : 1/250 ou 0.004
metadata-validation-aperture-format = Format : f/2.8 ou 2.8
metadata-validation-iso-positive = Doit être un entier positif
metadata-validation-focal-format = Format : 50 mm ou 50
metadata-validation-lat-range = Doit être entre -90 et 90
metadata-validation-lon-range = Doit être entre -180 et 180
metadata-validation-invalid-number = Nombre invalide

# Notifications de métadonnées
notification-metadata-save-success = Métadonnées enregistrées avec succès
notification-metadata-save-error = Impossible d'enregistrer les métadonnées
notification-metadata-validation-error = Veuillez corriger les erreurs de validation avant d'enregistrer

# Divulgation progressive des métadonnées
metadata-add-field = Ajouter un champ de métadonnées...
metadata-no-fields-message = Aucun champ de métadonnées. Utilisez "Ajouter un champ de métadonnées" pour ajouter des champs.

# Métadonnées Dublin Core / XMP
metadata-section-dublin-core = Dublin Core
metadata-label-dc-title = Titre
metadata-label-dc-creator = Créateur
metadata-label-dc-description = Description
metadata-label-dc-subject = Mots-clés
metadata-label-dc-rights = Droits d'auteur

navbar-info-button = Info

# Empty state (no media loaded)
empty-state-title = Aucun média chargé
empty-state-subtitle = Déposez des fichiers ici ou cliquez pour ouvrir
empty-state-button = Ouvrir un fichier
empty-state-drop-hint = Glissez-déposez des images ou vidéos n'importe où

# Additional notifications
notification-empty-dir = Aucun fichier média compatible trouvé dans ce dossier
notification-load-error-io = Impossible d'ouvrir le fichier. Vérifiez qu'il existe et que vous avez les permissions.
notification-load-error-svg = Impossible de rendre le SVG. Le fichier est peut-être malformé.
notification-load-error-video = Impossible de lire la vidéo. Le format n'est peut-être pas supporté.
notification-load-error-timeout = Le chargement a expiré. Le fichier est peut-être trop volumineux ou le système est occupé.
notification-skipped-corrupted-files = Ignorés : { $files }
notification-skipped-and-others = +{ $count } autres

# Paramètres IA
settings-enable-deblur-label = Défloutage IA
settings-enable-deblur-hint = Activer le défloutage d'images par IA avec le modèle NAFNet (~92 Mo à télécharger).
settings-deblur-model-url-label = URL du modèle
settings-deblur-model-url-placeholder = https://huggingface.co/...
settings-deblur-model-url-hint = URL pour télécharger le modèle NAFNet ONNX.
settings-deblur-status-label = État du modèle
settings-deblur-status-downloading = Téléchargement du modèle ({ $progress }%)...
settings-deblur-status-validating = Validation du modèle...
settings-deblur-status-ready = Modèle prêt
settings-deblur-status-error = Erreur : { $message }
settings-deblur-status-not-downloaded = Modèle non téléchargé
settings-deblur-enabled = Activé
settings-deblur-disabled = Désactivé

# Paramètres upscale IA
settings-enable-upscale-label = Agrandissement IA
settings-enable-upscale-hint = Activer l'agrandissement d'images par IA avec le modèle Real-ESRGAN 4x (~64 Mo à télécharger).
settings-upscale-model-url-label = URL du modèle
settings-upscale-model-url-placeholder = https://huggingface.co/...
settings-upscale-model-url-hint = URL pour télécharger le modèle Real-ESRGAN ONNX.
settings-upscale-status-label = État du modèle
settings-upscale-status-downloading = Téléchargement du modèle ({ $progress }%)...
settings-upscale-status-validating = Validation du modèle...
settings-upscale-status-ready = Modèle prêt
settings-upscale-status-error = Erreur : { $message }
settings-upscale-status-not-downloaded = Modèle non téléchargé
settings-upscale-enabled = Activé
settings-upscale-disabled = Désactivé

# Outil défloutage de l'éditeur
image-editor-tool-deblur = Défloutage IA
image-editor-deblur-lossless-warning = Pour une meilleure qualité, exportez en WebP sans perte ou PNG.
image-editor-deblur-apply = Appliquer le défloutage
image-editor-deblur-processing = Traitement en cours
image-editor-deblur-cancel = Annuler
image-editor-upscale-processing = Agrandissement IA en cours...
image-editor-deblur-model-not-ready = Activez d'abord le défloutage IA dans les paramètres
image-editor-deblur-validating = Validation du modèle en cours...
image-editor-deblur-downloading = Téléchargement du modèle ({ $progress }%)...
image-editor-deblur-error = Erreur : { $error }
image-editor-deblur-already-applied = Défloutage déjà appliqué. Utilisez Annuler pour revenir en arrière si nécessaire.

# Section d'aide IA
help-editor-deblur-title = Défloutage IA
help-editor-deblur-desc = Utilisez l'IA pour améliorer la netteté des images floues avec le réseau neuronal NAFNet.
help-editor-deblur-enable = À activer dans Paramètres → IA / Apprentissage automatique (télécharge un modèle de ~92 Mo)
help-editor-deblur-lossless = Pour une meilleure qualité, exportez en WebP sans perte ou PNG

# Notifications IA
notification-deblur-success = Image défloutée avec succès
notification-deblur-error = Échec du défloutage : { $error }
notification-deblur-download-success = Modèle de défloutage téléchargé avec succès
notification-deblur-download-error = Échec du téléchargement du modèle : { $error }
notification-deblur-validation-error = Échec de la validation du modèle : { $error }
notification-deblur-ready = Le défloutage IA est prêt à l'emploi
notification-deblur-apply-success = Image défloutée avec succès
notification-deblur-apply-error = Échec du défloutage : { $error }

# Notifications upscale IA
notification-upscale-ready = L'agrandissement IA est prêt à l'emploi
notification-upscale-download-error = Échec du téléchargement du modèle upscale : { $error }
notification-upscale-validation-error = Échec de la validation du modèle : { $error }
notification-upscale-resize-success = Image redimensionnée avec l'agrandissement IA
notification-upscale-resize-error = Échec de l'agrandissement IA : { $error }
