# Application name term - single source of truth for branding
-app-name = IcedLens

window-title = { -app-name }
hello-message = Hallo, Welt!
open-settings-button = Einstellungen
settings-back-to-viewer-button = Zurück zum Viewer
settings-title = Einstellungen
settings-section-general = Allgemein
settings-section-display = Anzeige
settings-section-video = Video
settings-section-fullscreen = Vollbild
select-language-label = Sprache auswählen:
language-name-en-US = Englisch
language-name-fr = Französisch
language-name-es = Spanisch
language-name-de = Deutsch
language-name-it = Italienisch
error-load-image-heading = Das Bild konnte nicht geöffnet werden.
error-load-image-general = Beim Laden des Bildes ist ein Fehler aufgetreten.
error-load-image-io = Die Datei konnte nicht gelesen werden. Überprüfen Sie, ob sie noch existiert und Sie die Berechtigung zum Öffnen haben.
error-load-image-svg = Diese SVG-Datei konnte nicht gerendert werden. Sie ist möglicherweise fehlerhaft oder wird nicht unterstützt.
error-details-show = Details anzeigen
error-details-hide = Details verbergen
error-details-technical-heading = Technische Details
viewer-zoom-label = Zoom
viewer-zoom-indicator-label = Zoom
viewer-zoom-input-placeholder = 100
viewer-zoom-reset-button = Zurücksetzen
viewer-fit-to-window-toggle = An Fenster anpassen
viewer-fit-percentage-label = Angepasster Zoom
viewer-zoom-input-error-invalid = Bitte geben Sie eine gültige Zahl ein.
viewer-zoom-step-error-invalid = Die Zoomstufe muss eine Zahl sein.
viewer-zoom-step-error-range = Die Zoomstufe muss zwischen 1% und 200% liegen.
viewer-position-label = Position
viewer-delete-tooltip = Aktuelles Bild löschen
viewer-zoom-in-tooltip = Vergrößern
viewer-zoom-out-tooltip = Verkleinern
viewer-fullscreen-tooltip = Vollbild umschalten
viewer-double-click = Doppelklick
viewer-scroll-wheel = Mausrad
viewer-click-drag = Klick + Ziehen
settings-zoom-step-label = Zoomstufe
settings-zoom-step-placeholder = 10
settings-zoom-step-hint = Wählen Sie, wie stark sich der Zoom beim Verwenden der Steuerelemente ändert (1% bis 200%).
settings-background-label = Viewer-Hintergrund
settings-background-light = Hell
settings-background-dark = Dunkel
settings-background-checkerboard = Schachbrett
settings-theme-mode-label = Anwendungsthema
settings-theme-system = Systemeinstellung folgen
settings-theme-light = Hell
settings-theme-dark = Dunkel
help-usage-heading = VERWENDUNG:
help-options-heading = OPTIONEN:
help-args-heading = ARGUMENTE:
help-examples-heading = BEISPIELE:
help-line-option-help = -h, --help        Diesen Hilfetext anzeigen
help-line-option-lang =     --lang <id>    Sprache festlegen (z.B. en-US, fr)
help-arg-image-path = <BILDPFAD>      Pfad zu einer Bilddatei zum Öffnen
help-example-1 = iced_lens ./foto.png
help-example-2 = iced_lens --lang fr ./bild.jpg
help-example-3 = iced_lens --help
help-description = { -app-name } – Bildbetrachter
help-line-option-i18n-dir =     --i18n-dir <pfad>  Übersetzungen aus Verzeichnis laden
help-line-option-data-dir =     --data-dir <pfad>  Datenverzeichnis überschreiben (Zustandsdateien)
help-line-option-config-dir =     --config-dir <pfad>  Konfigurationsverzeichnis überschreiben (settings.toml)
settings-sort-order-label = Sortierreihenfolge für Bildnavigation
settings-sort-alphabetical = Alphabetisch
settings-sort-modified = Änderungsdatum
settings-sort-created = Erstellungsdatum
settings-overlay-timeout-label = Verzögerung für automatisches Ausblenden im Vollbildmodus
settings-overlay-timeout-hint = Zeit bis zum Verschwinden der Steuerelemente im Vollbildmodus.
seconds = Sekunden
image-editor-title = Bildeditor
image-editor-back-to-viewer = Zurück zum Viewer
image-editor-cancel = Abbrechen
image-editor-save = Speichern
image-editor-save-as = Speichern unter...
image-editor-tool-rotate = Drehen
image-editor-tool-crop = Zuschneiden
image-editor-tool-resize = Größe ändern
image-editor-tool-light = Licht
image-editor-rotate-section-title = Drehung
image-editor-rotate-left = Nach links drehen
image-editor-rotate-right-tooltip = Bild im Uhrzeigersinn drehen
image-editor-rotate-left-tooltip = Bild gegen den Uhrzeigersinn drehen
image-editor-flip-section-title = Spiegeln
image-editor-flip-horizontal-tooltip = Bild horizontal spiegeln (links-rechts)
image-editor-flip-vertical-tooltip = Bild vertikal spiegeln (oben-unten)
image-editor-resize-section-title = Größe ändern
image-editor-resize-scale-label = Skalierung
image-editor-resize-dimensions-label = Zielgröße
image-editor-resize-width-label = Breite (px)
image-editor-resize-height-label = Höhe (px)
image-editor-resize-lock-aspect = Seitenverhältnis sperren
image-editor-resize-presets-label = Voreinstellungen
image-editor-resize-apply = Größenänderung anwenden
image-editor-light-section-title = Lichtanpassungen
image-editor-light-brightness-label = Helligkeit
image-editor-light-contrast-label = Kontrast
image-editor-light-reset = Zurücksetzen
image-editor-light-apply = Anwenden
image-editor-crop-section-title = Zuschneiden
image-editor-crop-ratio-label = Seitenverhältnis
image-editor-crop-ratio-free = Frei
image-editor-crop-ratio-square = Quadrat (1:1)
image-editor-crop-ratio-landscape = Querformat (16:9)
image-editor-crop-ratio-portrait = Hochformat (9:16)
image-editor-crop-ratio-photo = Foto (4:3)
image-editor-crop-ratio-photo-portrait = Foto Hochformat (3:4)
image-editor-crop-apply = Zuschnitt anwenden
image-editor-undo-redo-section-title = Letzte Änderung
image-editor-undo = Rückgängig
image-editor-redo = Wiederholen
image-editor-export-format-label = Exportformat
error-delete-image-io = Diese Datei konnte nicht gelöscht werden. Stellen Sie sicher, dass sie nicht anderweitig geöffnet ist und Sie sie entfernen können.
media-loading = Lädt...
error-loading-timeout = Zeitüberschreitung beim Laden. Die Datei ist möglicherweise zu groß oder nicht zugänglich.
settings-video-autoplay-label = Video-Autoplay
settings-video-autoplay-enabled = Aktiviert
settings-video-autoplay-disabled = Deaktiviert
settings-video-autoplay-hint = Wenn aktiviert, starten Videos beim Öffnen automatisch die Wiedergabe.
video-play-tooltip = Wiedergabe (Leertaste)
video-pause-tooltip = Pause (Leertaste)
video-mute-tooltip = Stummschalten (M)
video-unmute-tooltip = Ton einschalten (M)
video-loop-tooltip = Wiederholen
video-capture-tooltip = Aktuelles Bild aufnehmen
video-step-forward-tooltip = Ein Bild vorwärts (.)
video-step-backward-tooltip = Ein Bild rückwärts (,)
video-more-tooltip = Weitere Optionen
settings-audio-normalization-label = Audio-Lautstärkenormalisierung
settings-audio-normalization-enabled = Aktiviert
settings-audio-normalization-disabled = Deaktiviert
settings-audio-normalization-hint = Gleicht automatisch die Audiolautstärke zwischen verschiedenen Mediendateien an, um plötzliche Lautstärkeänderungen zu vermeiden.
settings-frame-cache-label = Video-Frame-Cache-Größe
settings-frame-cache-hint = Höhere Werte verbessern die Suchleistung, verwenden aber mehr Speicher. Änderungen gelten beim Öffnen eines neuen Videos.
settings-frame-history-label = Frame-Stepping-Verlaufsgröße
settings-frame-history-hint = Speicher für bildweises Rückwärtsgehen. Wird nur während des Stepping-Modus verwendet, nicht während der normalen Wiedergabe.
settings-keyboard-seek-step-label = Tastatur-Suchschritt
settings-keyboard-seek-step-hint = Zeitsprung beim Verwenden der Pfeiltasten während der Videowiedergabe.
megabytes = MB
error-load-video-heading = Dieses Video konnte nicht abgespielt werden.
error-load-video-general = Beim Laden des Videos ist ein Fehler aufgetreten.
error-load-video-unsupported-format = Dieses Dateiformat wird nicht unterstützt.
error-load-video-unsupported-codec = Der Video-Codec '{ $codec }' wird auf diesem System nicht unterstützt.
error-load-video-corrupted = Die Videodatei scheint beschädigt oder ungültig zu sein.
error-load-video-no-video-stream = In dieser Datei wurde keine Videospur gefunden.
error-load-video-decoding-failed = Video-Dekodierung fehlgeschlagen: { $message }
error-load-video-io = Diese Datei konnte nicht gelesen werden. Überprüfen Sie, ob sie noch existiert und Sie die Berechtigung zum Öffnen haben.
error-video-retry = Erneut versuchen
video-editor-unavailable = Videobearbeitung ist in dieser Version nicht verfügbar.
video-editor-future = Videobearbeitung ist für eine zukünftige Version geplant.

# Navigation bar
menu-button-tooltip = Menü
menu-settings = Einstellungen
menu-help = Hilfe
menu-about = Über
navbar-edit-button = Bearbeiten

# Help screen
help-title = Hilfe
help-back-to-viewer-button = Zurück zum Viewer

# Common labels
help-toc-title = Inhalt
help-tools-title = Verfügbare Werkzeuge
help-shortcuts-title = Tastaturkürzel
help-usage-title = Verwendung

# ─────────────────────────────────────────────────────────────────────────────
# Viewer Section
# ─────────────────────────────────────────────────────────────────────────────
help-section-viewer = Bild- und Videobetrachter
help-viewer-role = Durchsuchen und betrachten Sie Ihre Bilder und Videos. Navigieren Sie durch Dateien im selben Ordner und passen Sie die Anzeige an Ihre Vorlieben an.

help-viewer-tool-navigation = Navigation
help-viewer-tool-navigation-desc = Verwenden Sie die Pfeiltasten oder die Tastatur, um zwischen Dateien im Ordner zu wechseln.
help-viewer-tool-zoom = Zoom
help-viewer-tool-zoom-desc = Scrollen Sie mit dem Mausrad, verwenden Sie die +/- Tasten oder geben Sie direkt einen Prozentsatz ein.
help-viewer-tool-pan = Schwenken
help-viewer-tool-pan-desc = Wenn vergrößert, klicken und ziehen Sie das Bild, um sich zu bewegen.
help-viewer-tool-fit = An Fenster anpassen
help-viewer-tool-fit-desc = Skaliert das Bild automatisch, um vollständig in das Fenster zu passen.
help-viewer-tool-fullscreen = Vollbild
help-viewer-tool-fullscreen-desc = Immersive Ansicht mit automatisch ausblendbaren Steuerelementen (Verzögerung in Einstellungen konfigurierbar).
help-viewer-tool-delete = Löschen
help-viewer-tool-delete-desc = Aktuelle Datei dauerhaft entfernen (wird in den Systempapierkorbverschoben, falls verfügbar).

help-viewer-key-navigate = Zur vorherigen/nächsten Datei wechseln
help-viewer-key-edit = Bild im Editor öffnen
help-viewer-key-fullscreen = Vollbild betreten/verlassen
help-viewer-key-exit-fullscreen = Vollbildmodus verlassen
help-viewer-key-info = Dateiinformationsbereich umschalten

help-mouse-title = Mausinteraktionen
help-viewer-mouse-doubleclick = Doppelklick auf Bild/Video zum Umschalten des Vollbildmodus
help-viewer-mouse-wheel = Vergrößern/Verkleinern
help-viewer-mouse-drag = Bild schwenken, wenn vergrößert

# ─────────────────────────────────────────────────────────────────────────────
# Video Playback Section
# ─────────────────────────────────────────────────────────────────────────────
help-section-video = Videowiedergabe
help-video-role = Spielen Sie Videodateien mit vollständigen Wiedergabesteuerungen ab. Passen Sie die Lautstärke an, springen Sie in der Zeitleiste und navigieren Sie bildweise für präzise Positionierung.

help-video-tool-playback = Wiedergabe/Pause
help-video-tool-playback-desc = Starten oder stoppen Sie die Videowiedergabe mit der Wiedergabetaste oder der Leertaste.
help-video-tool-timeline = Zeitleiste
help-video-tool-timeline-desc = Klicken Sie irgendwo auf die Fortschrittsleiste, um zu dieser Position zu springen.
help-video-tool-volume = Lautstärke
help-video-tool-volume-desc = Ziehen Sie den Lautstärkeregler oder klicken Sie auf das Lautsprechersymbol zum Stummschalten/Einschalten.
help-video-tool-loop = Wiederholen
help-video-tool-loop-desc = Aktivieren, um das Video automatisch neu zu starten, wenn es endet.
help-video-tool-stepping = Bildweises Vor- und Zurückgehen
help-video-tool-stepping-desc = Wenn pausiert, bewegen Sie sich bildweise vor oder zurück für präzise Navigation.
help-video-tool-capture = Bildaufnahme
help-video-tool-capture-desc = Speichern Sie das aktuelle Videobild als Bilddatei (wird im Editor geöffnet).

help-video-key-playpause = Video wiedergeben oder pausieren
help-video-key-mute = Ton stummschalten umschalten
help-video-key-seek = Rückwärts/Vorwärts suchen (während der Wiedergabe)
help-video-key-volume = Lautstärke erhöhen/verringern
help-video-key-step-back = Ein Bild zurück (wenn pausiert)
help-video-key-step-forward = Ein Bild vor (wenn pausiert)

# ─────────────────────────────────────────────────────────────────────────────
# Image Editor Section
# ─────────────────────────────────────────────────────────────────────────────
help-section-editor = Bildeditor
help-editor-role = Nehmen Sie Anpassungen an Ihren Bildern vor: drehen, auf einen bestimmten Bereich zuschneiden oder auf andere Dimensionen skalieren.
help-editor-workflow = Alle Änderungen sind nicht-destruktiv, bis Sie speichern. Verwenden Sie „Speichern", um das Original zu überschreiben, oder „Speichern unter", um eine neue Datei zu erstellen und das Original zu erhalten.

help-editor-rotate-title = Drehung
help-editor-rotate-desc = Drehen oder spiegeln Sie das Bild, um die Ausrichtung zu korrigieren oder Spiegeleffekte zu erstellen.
help-editor-rotate-left = 90° gegen den Uhrzeigersinn drehen
help-editor-rotate-right = 90° im Uhrzeigersinn drehen
help-editor-flip-h = Horizontal spiegeln (links/rechts)
help-editor-flip-v = Vertikal spiegeln (oben/unten)

help-editor-crop-title = Zuschneiden
help-editor-crop-desc = Entfernen Sie unerwünschte Bereiche, indem Sie die Region auswählen, die Sie behalten möchten.
help-editor-crop-ratios = Wählen Sie ein voreingestelltes Verhältnis (1:1 Quadrat, 16:9 Querformat, 9:16 Hochformat, 4:3 oder 3:4 Foto) oder schneiden Sie frei zu.
help-editor-crop-usage = Ziehen Sie die Griffe, um die Auswahl anzupassen, und klicken Sie dann auf „Anwenden", um zu bestätigen.

help-editor-resize-title = Größe ändern
help-editor-resize-desc = Ändern Sie die Bildabmessungen, um es größer oder kleiner zu machen.
help-editor-resize-scale = Nach Prozentsatz skalieren (z.B. 50% um die Größe zu halbieren)
help-editor-resize-dimensions = Geben Sie exakte Breite und Höhe in Pixeln ein
help-editor-resize-lock = Seitenverhältnis sperren, um Proportionen beizubehalten
help-editor-resize-presets = Verwenden Sie Voreinstellungen für gängige Größen (HD, Full HD, 4K...)

help-editor-light-title = Licht
help-editor-light-desc = Passen Sie die Helligkeit und den Kontrast Ihres Bildes fein an.
help-editor-light-brightness = Helligkeit: Gesamtbild aufhellen oder abdunkeln
help-editor-light-contrast = Kontrast: Differenz zwischen hellen und dunklen Bereichen erhöhen oder verringern
help-editor-light-preview = Änderungen werden in Echtzeit vor dem Anwenden angezeigt

help-editor-save-title = Speichern
help-editor-save-overwrite = Speichern: Überschreibt die Originaldatei
help-editor-save-as = Speichern unter: Erstellt eine neue Datei (Speicherort und Format wählen)

help-editor-key-save = Aktuelle Änderungen speichern
help-editor-key-undo = Letzte Änderung rückgängig machen
help-editor-key-redo = Rückgängig gemachte Änderung wiederholen
help-editor-key-cancel = Alle Änderungen abbrechen und beenden

# ─────────────────────────────────────────────────────────────────────────────
# Frame Capture Section
# ─────────────────────────────────────────────────────────────────────────────
help-section-capture = Videobild-Aufnahme
help-capture-role = Extrahieren Sie beliebige Bilder aus einem Video und speichern Sie sie als Bilddatei. Perfekt zum Erstellen von Vorschaubildern oder Erfassen spezifischer Momente.

help-capture-step1 = Spielen Sie das Video ab oder navigieren Sie zum gewünschten Bild
help-capture-step2 = Pausieren Sie das Video (verwenden Sie bildweises Vor- und Zurückgehen für Präzision)
help-capture-step3 = Klicken Sie auf die Kamerataste in den Videosteuerungen
help-capture-step4 = Das Bild wird im Editor geöffnet — speichern Sie als PNG, JPEG oder WebP

help-capture-formats = Unterstützte Exportformate: PNG (verlustfrei), JPEG (kleinere Dateigröße), WebP (modernes Format mit guter Kompression).

# About screen
about-title = Über
about-back-to-viewer-button = Zurück zum Viewer

about-section-app = Anwendung
about-app-name = { -app-name }
about-app-description = Leichtgewichtiger Bild- und Videobetrachter mit grundlegender Bildbearbeitung.

about-section-license = Lizenz
about-license-name = Mozilla Public License 2.0 (MPL-2.0)
about-license-summary = Copyleft auf Dateiebene: Geänderte Dateien müssen unter derselben Lizenz geteilt werden. Kompatibel mit proprietärem Code.

about-section-icon-license = Icon-Lizenz
about-icon-license-name = { -app-name } Icon-Lizenz
about-icon-license-summary = Alle Icons (Anwendungslogo und UI-Icons) dürfen nur unverändert weitergegeben werden, um { -app-name } zu repräsentieren.

about-section-credits = Danksagungen
about-credits-iced = Erstellt mit dem Iced GUI-Toolkit
about-credits-ffmpeg = Videowiedergabe mit FFmpeg
about-credits-fluent = Internationalisierung durch Project Fluent

about-section-links = Links
about-link-repository = Quellcode
about-link-issues = Probleme melden

# Notifications
notification-save-success = Bild erfolgreich gespeichert
notification-save-error = Fehler beim Speichern des Bildes
notification-frame-capture-success = Bild erfolgreich aufgenommen
notification-frame-capture-error = Fehler beim Aufnehmen des Bildes
notification-delete-success = Datei erfolgreich gelöscht
notification-delete-error = Fehler beim Löschen der Datei
notification-copy-success = In die Zwischenablage kopiert
notification-copy-error = Fehler beim Kopieren in die Zwischenablage
notification-config-save-error = Fehler beim Speichern der Einstellungen
notification-config-load-error = Fehler beim Laden der Einstellungen, verwende Standardwerte
notification-state-parse-error = Fehler beim Lesen des Anwendungszustands, verwende Standardwerte
notification-state-read-error = Fehler beim Öffnen der Zustandsdatei
notification-state-path-error = Anwendungsdatenpfad kann nicht bestimmt werden
notification-state-dir-error = Fehler beim Erstellen des Anwendungsdatenverzeichnisses
notification-state-write-error = Fehler beim Speichern des Anwendungszustands
notification-state-create-error = Fehler beim Erstellen der Zustandsdatei
notification-scan-dir-error = Fehler beim Scannen des Verzeichnisses
notification-editor-frame-error = Fehler beim Öffnen des Editors mit dem aufgenommenen Bild
notification-editor-create-error = Fehler beim Öffnen des Bildeditors
notification-editor-load-error = Fehler beim Laden des Bildes zur Bearbeitung
notification-video-editing-unsupported = Videobearbeitung wird noch nicht unterstützt

# Metadata panel
metadata-panel-title = Dateiinformationen
metadata-section-file = Datei
metadata-section-camera = Kamera
metadata-section-exposure = Belichtung
metadata-section-video = Video
metadata-section-audio = Audio
metadata-section-gps = Standort
metadata-label-dimensions = Abmessungen
metadata-label-file-size = Dateigröße
metadata-label-format = Format
metadata-label-date-taken = Aufnahmedatum
metadata-label-camera = Kamera
metadata-label-exposure = Belichtung
metadata-label-aperture = Blende
metadata-label-iso = ISO
metadata-label-focal-length = Brennweite
metadata-label-gps = Koordinaten
metadata-label-codec = Codec
metadata-label-bitrate = Bitrate
metadata-label-duration = Dauer
metadata-label-fps = Bildrate
metadata-value-unknown = Unbekannt
navbar-info-button = Info
navbar-info-tooltip = Dateiinformationen anzeigen (I)
