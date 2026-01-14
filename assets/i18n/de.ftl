# Application name term - single source of truth for branding
-app-name = IcedLens

window-title = { -app-name }
new-image-title = Neues Bild
settings-back-to-viewer-button = Zurück zum Viewer
settings-title = Einstellungen
settings-section-general = Allgemein
settings-section-display = Anzeige
settings-section-video = Video
settings-section-fullscreen = Vollbild
settings-section-ai = KI / Maschinelles Lernen
select-language-label = Sprache auswählen:
language-name-en-US = Englisch
language-name-fr = Französisch
language-name-es = Spanisch
language-name-de = Deutsch
language-name-it = Italienisch
error-load-image-heading = Das Bild konnte nicht geöffnet werden.
error-details-show = Details anzeigen
error-details-hide = Details verbergen
error-details-technical-heading = Technische Details
viewer-zoom-label = Zoom
viewer-zoom-input-placeholder = 100
viewer-zoom-reset-button = Zurücksetzen
viewer-fit-to-window-toggle = An Fenster anpassen
viewer-zoom-input-error-invalid = Bitte geben Sie eine gültige Zahl ein.
viewer-zoom-step-error-invalid = Die Zoomstufe muss eine Zahl sein.
viewer-zoom-step-error-range = Die Zoomstufe muss zwischen 1% und 200% liegen.
viewer-delete-tooltip = Aktuelles Bild löschen
viewer-zoom-in-tooltip = Vergrößern
viewer-zoom-out-tooltip = Verkleinern
viewer-fullscreen-tooltip = Vollbild umschalten
viewer-rotate-cw-tooltip = Im Uhrzeigersinn drehen
viewer-rotate-ccw-tooltip = Gegen Uhrzeigersinn drehen
viewer-fullscreen-disabled-unsaved = Änderungen zuerst speichern oder abbrechen
viewer-double-click = Doppelklick
viewer-scroll-wheel = Mausrad
viewer-click-drag = Klick + Ziehen

# Filter dropdown
filter-dropdown-tooltip = Medien filtern
filter-dropdown-tooltip-active = Aktiv: { $filters }
filter-panel-title = Filter
filter-reset-button = Zurücksetzen
filter-media-type-label = Medientyp
filter-media-type-all = Alle
filter-media-type-images = Nur Bilder
filter-media-type-videos = Nur Videos
filter-media-type-placeholder = Typ auswählen...
filter-date-label = Datumsfilter
filter-date-field-label = Filtern nach
filter-date-field-modified = Änderungsdatum
filter-date-field-created = Erstellungsdatum
filter-date-start-label = Von
filter-date-end-label = Bis
filter-date-day-placeholder = TT
filter-date-month-placeholder = MM
filter-date-year-placeholder = JJJJ
filter-date-clear = Datum löschen
filter-tooltip-date-from = ab { $date }
filter-tooltip-date-to = bis { $date }
filter-tooltip-date-range = { $start } – { $end }

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
help-arg-image-path = <PFAD>      Pfad zu einer Mediendatei oder einem Verzeichnis zum Öffnen
help-example-1 = iced_lens ./foto.png
help-example-2 = iced_lens ./meine_fotos/
help-example-3 = iced_lens --lang fr ./bild.jpg
help-description = { -app-name } – Bildbetrachter
help-line-option-i18n-dir =     --i18n-dir <pfad>  Übersetzungen aus Verzeichnis laden
help-line-option-data-dir =     --data-dir <pfad>  Datenverzeichnis überschreiben (Zustandsdateien)
help-line-option-config-dir =     --config-dir <pfad>  Konfigurationsverzeichnis überschreiben (settings.toml)
settings-sort-order-label = Sortierreihenfolge für Bildnavigation
settings-sort-alphabetical = Alphabetisch
settings-sort-modified = Änderungsdatum
settings-sort-created = Erstellungsdatum
settings-max-skip-attempts-label = Beschädigte Dateien überspringen
settings-max-skip-attempts-hint = Maximale Anzahl beschädigter Dateien, die bei der Navigation übersprungen werden.
settings-persist-filters-label = Filter merken
settings-persist-filters-hint = Filtereinstellungen zwischen Sitzungen beibehalten.
settings-persist-filters-disabled = Aus
settings-persist-filters-enabled = An
settings-overlay-timeout-label = Verzögerung für automatisches Ausblenden im Vollbildmodus
settings-overlay-timeout-hint = Zeit bis zum Verschwinden der Steuerelemente im Vollbildmodus.
seconds = Sekunden
image-editor-title = Bildeditor
image-editor-back-to-viewer = Zurück zum Viewer
image-editor-cancel = Abbrechen
image-editor-save = Speichern
image-editor-save-as = Speichern unter...
image-editor-tool-crop = Zuschneiden
image-editor-tool-resize = Größe ändern
image-editor-tool-light = Licht
image-editor-rotate-section-title = Drehung
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
image-editor-resize-preview-label = Vorschau
image-editor-resize-ai-upscale = KI-Hochskalierung (Real-ESRGAN)
image-editor-resize-ai-model-not-downloaded = KI-Modell noch nicht heruntergeladen
image-editor-resize-ai-model-downloading = KI-Modell wird heruntergeladen
image-editor-resize-ai-model-validating = KI-Modell wird validiert
image-editor-resize-ai-model-error = KI-Modell-Fehler
image-editor-resize-ai-enlargement-only = KI-Upscaling gilt nur für Vergrößerungen
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
media-loading = Lädt...
settings-video-autoplay-label = Video-Autoplay
settings-video-autoplay-enabled = Aktiviert
settings-video-autoplay-disabled = Deaktiviert
settings-video-autoplay-hint = Wenn aktiviert, starten Videos beim Öffnen automatisch die Wiedergabe.
video-play-tooltip = Wiedergabe (Leertaste)
video-pause-tooltip = Pause (Leertaste)
video-mute-tooltip = Stummschalten (M)
video-unmute-tooltip = Ton einschalten (M)
video-no-audio-tooltip = Keine Audiospur
video-loop-tooltip = Wiederholen
video-capture-tooltip = Aktuelles Bild aufnehmen
video-step-forward-tooltip = Ein Bild vorwärts (.)
video-step-backward-tooltip = Ein Bild rückwärts (,)
video-more-tooltip = Weitere Optionen
video-speed-down-tooltip = Geschwindigkeit verringern (J)
video-speed-up-tooltip = Geschwindigkeit erhöhen (L)
hud-video-no-audio = Kein Audio
settings-audio-normalization-label = Audio-Lautstärkenormalisierung
settings-audio-normalization-enabled = Aktiviert
settings-audio-normalization-disabled = Deaktiviert
settings-audio-normalization-hint = Gleicht automatisch die Audiolautstärke zwischen verschiedenen Mediendateien an, um plötzliche Lautstärkeänderungen zu vermeiden.
settings-frame-cache-label = Keyframe-Cache-Größe (für Suche)
settings-frame-cache-hint = Speichert Video-Keyframes zwischen, um das Scrubben in der Timeline und Sprünge zu bestimmten Zeiten zu beschleunigen. Höhere Werte speichern mehr Keyframes für schnellere Suche. Änderungen gelten beim Öffnen eines neuen Videos.
settings-frame-history-label = Frame-Verlaufsgröße (für Rückwärtsgehen)
settings-frame-history-hint = Speichert kürzlich angezeigte Bilder, um bildweises Rückwärtsgehen zu ermöglichen. Wird nur beim manuellen Durchblättern verwendet, nicht während der normalen Wiedergabe.
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
error-video-decoder-died = Der Video-Decoder wurde unerwartet beendet. Versuchen Sie, das Video neu zu laden.
error-video-seek-timeout = Die Suche ist abgelaufen. Die Zielposition liegt möglicherweise hinter dem Ende des Videos.

# Navigation bar
menu-settings = Einstellungen
menu-help = Hilfe
menu-about = Über
menu-diagnostics = Diagnose
navbar-edit-button = Bearbeiten

# Help screen
help-title = Hilfe
help-back-to-viewer-button = Zurück zum Viewer

# Common labels
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
help-viewer-tool-filter = Filter
help-viewer-tool-filter-desc = Nur passende Dateien anzeigen. Filtern nach Medientyp, Ausrichtung oder Datumsbereich.
help-viewer-tool-delete = Löschen
help-viewer-tool-delete-desc = Aktuelle Datei dauerhaft entfernen (wird in den Systempapierkorbverschoben, falls verfügbar).

help-viewer-key-navigate = Zur vorherigen/nächsten Datei wechseln
help-viewer-key-edit = Bild im Editor öffnen
help-viewer-key-fullscreen = Vollbild betreten/verlassen
help-viewer-key-exit-fullscreen = Vollbildmodus verlassen
help-viewer-key-info = Dateiinformationsbereich umschalten
help-viewer-key-rotate-cw = Im Uhrzeigersinn drehen
help-viewer-key-rotate-ccw = Gegen den Uhrzeigersinn drehen

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
help-video-tool-volume-desc = Ziehen Sie den Lautstärkeregler (0-150%) oder klicken Sie auf das Lautsprechersymbol zum Stummschalten/Einschalten.
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
help-video-key-speed-down = Wiedergabegeschwindigkeit verringern
help-video-key-speed-up = Wiedergabegeschwindigkeit erhöhen

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
help-editor-resize-scale = Nach Prozentsatz skalieren (10% bis 400%)
help-editor-resize-dimensions = Geben Sie exakte Breite und Höhe in Pixeln ein
help-editor-resize-lock = Seitenverhältnis sperren, um Proportionen beizubehalten
help-editor-resize-presets = Verwenden Sie Voreinstellungen für schnelle Skalierung (25%, 50%, 200%, etc.)
help-editor-resize-ai-upscale = KI-Upscaling: Verwenden Sie Real-ESRGAN für schärfere Vergrößerungen (in Einstellungen aktivieren)
help-editor-resize-ai-validation = KI-Modell wird beim ersten Öffnen des Editors validiert (nicht beim Start)

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

help-editor-mouse-title = Maussteuerung
help-editor-mouse-wheel = Bild vergrößern oder verkleinern
help-editor-mouse-drag = Bild verschieben wenn vergrößert

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

# ─────────────────────────────────────────────────────────────────────────────
# Metadaten-Bearbeitungs-Abschnitt
# ─────────────────────────────────────────────────────────────────────────────
help-section-metadata = Metadaten-Bearbeitung
help-metadata-role = Sehen und bearbeiten Sie Metadaten in Ihren Bilddateien. Zwei Typen werden unterstützt: Dublin Core (XMP) für beschreibende Informationen wie Titel und Autor, und EXIF für technische Kameradaten.

help-metadata-tool-view = Anzeigemodus
help-metadata-tool-view-desc = Drücken Sie I oder klicken Sie auf Info, um alle Metadaten im Seitenbereich anzuzeigen.
help-metadata-tool-edit = Bearbeitungsmodus
help-metadata-tool-edit-desc = Klicken Sie auf Bearbeiten, um Felder zu ändern. Verwenden Sie „Feld hinzufügen" für neue Metadaten. Die Validierung erfolgt während der Eingabe.
help-metadata-tool-save = Speicheroptionen
help-metadata-tool-save-desc = Speichern aktualisiert die Originaldatei, Speichern unter erstellt eine Kopie mit den neuen Metadaten.

help-metadata-fields-title = Bearbeitbare Felder
help-metadata-field-dc = Dublin Core (XMP): Titel, Autor, Beschreibung, Stichwörter, Copyright
help-metadata-field-camera = Kamerahersteller und Modell
help-metadata-field-date = Aufnahmedatum (mit Datumsauswahl)
help-metadata-field-exposure = Belichtungszeit, Blende, ISO-Empfindlichkeit
help-metadata-field-focal = Brennweite und 35mm-Äquivalent
help-metadata-field-gps = GPS-Koordinaten (Breiten- und Längengrad)

help-metadata-formats = Unterstützte Formate: JPEG, PNG, WebP und TIFF unterstützen Dublin Core und EXIF. Andere Formate haben möglicherweise eingeschränkte oder schreibgeschützte Unterstützung.
help-metadata-note = Hinweis: Videodateien zeigen Metadaten nur im Lesemodus an. Nur die von jedem Format unterstützten Felder sind zur Bearbeitung verfügbar.

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
about-credits-onnx = KI-Funktionen mit ONNX Runtime (NAFNet, Real-ESRGAN)
about-credits-fluent = Internationalisierung durch Project Fluent
about-credits-full-list = Vollständige Abhängigkeitsliste anzeigen

about-section-third-party = Drittanbieter-Lizenzen
about-third-party-ffmpeg = FFmpeg ist unter LGPL 2.1+ lizenziert
about-third-party-onnx = ONNX Runtime und DirectML sind unter MIT lizenziert
about-third-party-details = Siehe THIRD_PARTY_LICENSES.md für Details

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
metadata-panel-close = Panel schließen
metadata-panel-close-disabled = Änderungen zuerst speichern oder abbrechen
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
metadata-section-processing = Verarbeitung
metadata-label-software = Software
metadata-label-date-modified = Geändert
metadata-label-codec = Codec
metadata-label-bitrate = Bitrate
metadata-label-duration = Dauer
metadata-label-fps = Bildrate
metadata-value-unknown = Unbekannt

# Metadaten-Bearbeitung
metadata-edit-button = Bearbeiten
metadata-edit-disabled-video = Metadaten-Bearbeitung ist für Videos nicht verfügbar
metadata-cancel-button = Abbrechen
metadata-save-button = Speichern
metadata-save-as-button = Speichern unter...
metadata-save-warning = Speichern ändert die Originaldatei
metadata-label-make = Hersteller
metadata-label-model = Modell
metadata-label-focal-length-35mm = Brennweite (35mm)
metadata-label-flash = Blitz
metadata-label-latitude = Breitengrad
metadata-label-longitude = Längengrad
metadata-validation-date-format = Format: JJJJ:MM:TT HH:MM:SS
metadata-validation-date-invalid = Ungültige Datum/Zeit-Werte
metadata-date-placeholder = JJJJ-MM-TT HH:MM:SS
metadata-date-now = Jetzt
metadata-date-help = Akzeptiert: JJJJ-MM-TT, TT/MM/JJJJ, usw.
metadata-validation-exposure-format = Format: 1/250 oder 0.004
metadata-validation-aperture-format = Format: f/2.8 oder 2.8
metadata-validation-iso-positive = Muss eine positive Ganzzahl sein
metadata-validation-focal-format = Format: 50 mm oder 50
metadata-validation-lat-range = Muss zwischen -90 und 90 liegen
metadata-validation-lon-range = Muss zwischen -180 und 180 liegen
metadata-validation-invalid-number = Ungültige Zahl

# Metadaten-Benachrichtigungen
notification-metadata-save-success = Metadaten erfolgreich gespeichert
notification-metadata-save-error = Fehler beim Speichern der Metadaten
notification-metadata-validation-error = Bitte beheben Sie die Validierungsfehler vor dem Speichern

# Metadaten progressive Offenlegung
metadata-add-field = Metadatenfeld hinzufügen...
metadata-no-fields-message = Keine Metadatenfelder. Verwenden Sie "Metadatenfeld hinzufügen", um Felder hinzuzufügen.

# Dublin Core / XMP Metadaten
metadata-section-dublin-core = Dublin Core
metadata-label-dc-title = Titel
metadata-label-dc-creator = Ersteller
metadata-label-dc-description = Beschreibung
metadata-label-dc-subject = Schlagwörter
metadata-label-dc-rights = Urheberrecht

navbar-info-button = Info

# Empty state (no media loaded)
empty-state-title = Keine Medien geladen
empty-state-subtitle = Dateien hier ablegen oder klicken zum Öffnen
empty-state-button = Datei öffnen
empty-state-drop-hint = Bilder oder Videos hier hinziehen

# Additional notifications
notification-empty-dir = Keine unterstützten Mediendateien in diesem Ordner gefunden
notification-load-error-io = Datei konnte nicht geöffnet werden. Prüfen Sie, ob sie existiert und Sie Zugriffsrechte haben.
notification-load-error-svg = SVG konnte nicht gerendert werden. Die Datei ist möglicherweise fehlerhaft.
notification-load-error-video = Video konnte nicht abgespielt werden. Das Format wird möglicherweise nicht unterstützt.
notification-load-error-timeout = Laden hat zu lange gedauert. Die Datei ist möglicherweise zu groß oder das System ist ausgelastet.
notification-skipped-corrupted-files = Übersprungen: { $files }
notification-skipped-and-others = +{ $count } weitere

# KI-Einstellungen
settings-enable-deblur-label = KI-Entunschärfung
settings-enable-deblur-hint = KI-gestützte Bildentunschärfung mit dem NAFNet-Modell aktivieren (~92 MB Download).
settings-deblur-model-url-label = Modell-URL
settings-deblur-model-url-placeholder = https://huggingface.co/...
settings-deblur-model-url-hint = URL zum Herunterladen des NAFNet ONNX-Modells.
settings-deblur-status-label = Modellstatus
settings-deblur-status-downloading = Modell wird heruntergeladen ({ $progress }%)...
settings-deblur-status-validating = Modell wird validiert...
settings-deblur-status-ready = Modell bereit
settings-deblur-status-error = Fehler: { $message }
settings-deblur-status-not-downloaded = Modell nicht heruntergeladen
settings-deblur-status-needs-validation = Modell heruntergeladen (Validierung bei erster Verwendung)
settings-deblur-enabled = Aktiviert
settings-deblur-disabled = Deaktiviert

# KI-Hochskalierung Einstellungen
settings-enable-upscale-label = KI-Hochskalierung
settings-enable-upscale-hint = KI-gestützte Bildvergrößerung mit dem Real-ESRGAN 4x-Modell aktivieren (~64 MB Download).
settings-upscale-model-url-label = Modell-URL
settings-upscale-model-url-placeholder = https://huggingface.co/...
settings-upscale-model-url-hint = URL zum Herunterladen des Real-ESRGAN ONNX-Modells.
settings-upscale-status-label = Modellstatus
settings-upscale-status-downloading = Modell wird heruntergeladen ({ $progress }%)...
settings-upscale-status-validating = Modell wird validiert...
settings-upscale-status-ready = Modell bereit
settings-upscale-status-error = Fehler: { $message }
settings-upscale-status-not-downloaded = Modell nicht heruntergeladen
settings-upscale-status-needs-validation = Modell heruntergeladen (Validierung bei erster Verwendung)
settings-upscale-enabled = Aktiviert
settings-upscale-disabled = Deaktiviert

# KI-Editor-Werkzeug
image-editor-tool-deblur = KI-Entunschärfung
image-editor-deblur-lossless-warning = Für beste Qualität als verlustfreies WebP oder PNG exportieren.
image-editor-deblur-apply = Entunschärfung anwenden
image-editor-deblur-processing = Verarbeitung
image-editor-deblur-cancel = Abbrechen
image-editor-upscale-processing = KI-Hochskalierung...
image-editor-deblur-model-not-ready = Aktivieren Sie zuerst KI-Entunschärfung in den Einstellungen
image-editor-deblur-validating = Modell wird validiert, bitte warten...
image-editor-deblur-downloading = Modell wird heruntergeladen ({ $progress }%)...
image-editor-deblur-error = Fehler: { $error }
image-editor-deblur-already-applied = Entunschärfung bereits angewendet. Verwenden Sie Rückgängig, um bei Bedarf zurückzusetzen.
image-editor-metadata-options-title = Metadaten
image-editor-metadata-add-software = Software-Tag und Änderungsdatum hinzufügen
image-editor-metadata-strip-gps = GPS-Standortdaten entfernen

# KI-Hilfeabschnitt
help-editor-deblur-title = KI-Entunschärfung
help-editor-deblur-desc = Verwenden Sie KI, um unscharfe Bilder mit dem neuronalen Netzwerk NAFNet zu schärfen.
help-editor-deblur-enable = Aktivieren unter Einstellungen → KI / Maschinelles Lernen (lädt ~92 MB Modell herunter)
help-editor-deblur-validation = Modell wird beim ersten Öffnen des Editors validiert (nicht beim Start)
help-editor-deblur-lossless = Für beste Qualität als verlustfreies WebP oder PNG exportieren

# KI-Benachrichtigungen
notification-deblur-success = Bild erfolgreich entschärft
notification-deblur-error = Entunschärfung fehlgeschlagen: { $error }
notification-deblur-download-success = Entunschärfungs-Modell erfolgreich heruntergeladen
notification-deblur-download-error = Herunterladen des Modells fehlgeschlagen: { $error }
notification-deblur-validation-error = Modellvalidierung fehlgeschlagen: { $error }
notification-deblur-ready = KI-Entunschärfung ist einsatzbereit
notification-deblur-apply-success = Bild erfolgreich entunschärft
notification-deblur-apply-error = Entunschärfung fehlgeschlagen: { $error }

# KI-Hochskalierung Benachrichtigungen
notification-upscale-ready = KI-Hochskalierung ist einsatzbereit
notification-upscale-download-error = Herunterladen des Hochskalierungs-Modells fehlgeschlagen: { $error }
notification-upscale-validation-error = Modellvalidierung fehlgeschlagen: { $error }
notification-upscale-resize-success = Bild mit KI-Hochskalierung vergrößert
notification-upscale-resize-error = KI-Hochskalierung fehlgeschlagen: { $error }

# Diagnose-Bildschirm
diagnostics-title = Diagnose
diagnostics-back-button = Zurück zum Viewer
diagnostics-status-enabled = Sammlung: Aktiviert
diagnostics-status-disabled = Sammlung: Deaktiviert
diagnostics-status-error = Sammlung: Fehler
diagnostics-events-running-for = Ereignisse: { $duration }
diagnostics-resources-running-for = Ressourcen: { $duration }
diagnostics-buffer-count = Puffer: { $count } Ereignisse
diagnostics-toggle-label = Ressourcensammlung aktivieren
