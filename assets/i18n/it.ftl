# Application name term - single source of truth for branding
-app-name = IcedLens

window-title = { -app-name }
hello-message = Ciao, mondo!
open-settings-button = Impostazioni
settings-back-to-viewer-button = Torna al visualizzatore
settings-title = Impostazioni
settings-section-general = Generale
settings-section-display = Visualizzazione
settings-section-video = Video
settings-section-fullscreen = Schermo intero
select-language-label = Seleziona lingua:
language-name-en-US = Inglese
language-name-fr = Francese
language-name-es = Spagnolo
language-name-de = Tedesco
language-name-it = Italiano
error-load-image-heading = Impossibile aprire l'immagine.
error-load-image-general = Si è verificato un errore durante il caricamento dell'immagine.
error-load-image-io = Impossibile leggere questo file. Verifica che esista ancora e che tu abbia il permesso di aprirlo.
error-load-image-svg = Impossibile visualizzare questo file SVG. Potrebbe essere malformato o non supportato.
error-details-show = Mostra dettagli
error-details-hide = Nascondi dettagli
error-details-technical-heading = Dettagli tecnici
viewer-zoom-label = Zoom
viewer-zoom-indicator-label = Zoom
viewer-zoom-input-placeholder = 100
viewer-zoom-reset-button = Ripristina
viewer-fit-to-window-toggle = Adatta alla finestra
viewer-fit-percentage-label = Zoom adattato
viewer-zoom-input-error-invalid = Inserisci un numero valido.
viewer-zoom-step-error-invalid = Il passo dello zoom deve essere un numero.
viewer-zoom-step-error-range = Il passo dello zoom deve essere compreso tra 1% e 200%.
viewer-position-label = Posizione
viewer-delete-tooltip = Elimina l'immagine corrente
viewer-zoom-in-tooltip = Ingrandisci
viewer-zoom-out-tooltip = Riduci
viewer-fullscreen-tooltip = Attiva/disattiva schermo intero
viewer-double-click = Doppio clic
viewer-scroll-wheel = Rotella del mouse
viewer-click-drag = Clic + trascina
settings-zoom-step-label = Passo dello zoom
settings-zoom-step-placeholder = 10
settings-zoom-step-hint = Scegli quanto cambia lo zoom quando usi i controlli (dall'1% al 200%).
settings-background-label = Sfondo del visualizzatore
settings-background-light = Chiaro
settings-background-dark = Scuro
settings-background-checkerboard = Scacchiera
settings-theme-mode-label = Tema dell'applicazione
settings-theme-system = Segui il sistema
settings-theme-light = Chiaro
settings-theme-dark = Scuro
help-usage-heading = USO:
help-options-heading = OPZIONI:
help-args-heading = ARGOMENTI:
help-examples-heading = ESEMPI:
help-line-option-help = -h, --help        Mostra questo testo di aiuto
help-line-option-lang =     --lang <id>    Imposta la lingua (es. en-US, fr)
help-arg-image-path = <PERCORSO_IMMAGINE>      Percorso di un file immagine da aprire
help-example-1 = iced_lens ./foto.png
help-example-2 = iced_lens --lang fr ./immagine.jpg
help-example-3 = iced_lens --help
help-description = { -app-name } – Visualizzatore di immagini
help-line-option-i18n-dir =     --i18n-dir <percorso>  Carica le traduzioni dalla directory
help-line-option-data-dir =     --data-dir <percorso>  Sovrascrivi directory dei dati (file di stato)
help-line-option-config-dir =     --config-dir <percorso>  Sovrascrivi directory di configurazione (settings.toml)
settings-sort-order-label = Ordine di navigazione delle immagini
settings-sort-alphabetical = Alfabetico
settings-sort-modified = Data di modifica
settings-sort-created = Data di creazione
settings-overlay-timeout-label = Ritardo di scomparsa automatica a schermo intero
settings-overlay-timeout-hint = Tempo prima che i controlli scompaiano in modalità a schermo intero.
seconds = secondi
image-editor-title = Editor di immagini
image-editor-back-to-viewer = Torna al visualizzatore
image-editor-cancel = Annulla
image-editor-save = Salva
image-editor-save-as = Salva come...
image-editor-tool-rotate = Ruota
image-editor-tool-crop = Ritaglia
image-editor-tool-resize = Ridimensiona
image-editor-tool-light = Luce
image-editor-rotate-section-title = Rotazione
image-editor-rotate-left = Ruota a sinistra
image-editor-rotate-right-tooltip = Ruota l'immagine in senso orario
image-editor-rotate-left-tooltip = Ruota l'immagine in senso antiorario
image-editor-flip-section-title = Capovolgi
image-editor-flip-horizontal-tooltip = Capovolgi l'immagine orizzontalmente (specchio sinistra-destra)
image-editor-flip-vertical-tooltip = Capovolgi l'immagine verticalmente (specchio alto-basso)
image-editor-resize-section-title = Ridimensiona
image-editor-resize-scale-label = Scala
image-editor-resize-dimensions-label = Dimensione target
image-editor-resize-width-label = Larghezza (px)
image-editor-resize-height-label = Altezza (px)
image-editor-resize-lock-aspect = Blocca proporzioni
image-editor-resize-presets-label = Preimpostazioni
image-editor-resize-apply = Applica ridimensionamento
image-editor-light-section-title = Regolazioni di luce
image-editor-light-brightness-label = Luminosità
image-editor-light-contrast-label = Contrasto
image-editor-light-reset = Ripristina
image-editor-light-apply = Applica
image-editor-crop-section-title = Ritaglia
image-editor-crop-ratio-label = Proporzioni
image-editor-crop-ratio-free = Libero
image-editor-crop-ratio-square = Quadrato (1:1)
image-editor-crop-ratio-landscape = Orizzontale (16:9)
image-editor-crop-ratio-portrait = Verticale (9:16)
image-editor-crop-ratio-photo = Foto (4:3)
image-editor-crop-ratio-photo-portrait = Foto verticale (3:4)
image-editor-crop-apply = Applica ritaglio
image-editor-undo-redo-section-title = Ultima modifica
image-editor-undo = Annulla
image-editor-redo = Ripeti
image-editor-export-format-label = Formato di esportazione
error-delete-image-io = Impossibile eliminare questo file. Assicurati che non sia aperto altrove e che tu possa rimuoverlo.
media-loading = Caricamento...
error-loading-timeout = Timeout del caricamento. Il file potrebbe essere troppo grande o inaccessibile.
settings-video-autoplay-label = Riproduzione automatica video
settings-video-autoplay-enabled = Attivata
settings-video-autoplay-disabled = Disattivata
settings-video-autoplay-hint = Quando è attivata, i video iniziano a essere riprodotti automaticamente all'apertura.
video-play-tooltip = Riproduci (Spazio)
video-pause-tooltip = Pausa (Spazio)
video-mute-tooltip = Silenzia (M)
video-unmute-tooltip = Attiva audio (M)
video-loop-tooltip = Ripeti
video-capture-tooltip = Cattura fotogramma corrente
video-step-forward-tooltip = Avanza di un fotogramma (.)
video-step-backward-tooltip = Indietreggia di un fotogramma (,)
video-more-tooltip = Altre opzioni
settings-audio-normalization-label = Normalizzazione del volume audio
settings-audio-normalization-enabled = Attivata
settings-audio-normalization-disabled = Disattivata
settings-audio-normalization-hint = Livella automaticamente il volume audio tra diversi file multimediali per evitare cambiamenti improvvisi di volume.
settings-frame-cache-label = Dimensione cache fotogrammi video
settings-frame-cache-hint = Valori più alti migliorano le prestazioni di ricerca ma usano più memoria. Le modifiche si applicano all'apertura di un nuovo video.
settings-frame-history-label = Dimensione cronologia navigazione fotogrammi
settings-frame-history-hint = Memoria utilizzata per l'avanzamento fotogramma per fotogramma all'indietro. Utilizzata solo durante la modalità di navigazione, non durante la riproduzione normale.
settings-keyboard-seek-step-label = Passo di ricerca da tastiera
settings-keyboard-seek-step-hint = Tempo da saltare quando si usano i tasti freccia durante la riproduzione video.
megabytes = MB
error-load-video-heading = Impossibile riprodurre questo video.
error-load-video-general = Si è verificato un errore durante il caricamento del video.
error-load-video-unsupported-format = Questo formato di file non è supportato.
error-load-video-unsupported-codec = Il codec video '{ $codec }' non è supportato su questo sistema.
error-load-video-corrupted = Il file video sembra essere danneggiato o non valido.
error-load-video-no-video-stream = Nessuna traccia video trovata in questo file.
error-load-video-decoding-failed = Decodifica video fallita: { $message }
error-load-video-io = Impossibile leggere questo file. Verifica che esista ancora e che tu abbia il permesso di aprirlo.
error-video-retry = Riprova
video-editor-unavailable = La modifica video non è disponibile in questa versione.
video-editor-future = La modifica video è pianificata per una versione futura.

# Navigation bar
menu-button-tooltip = Menu
menu-settings = Impostazioni
menu-help = Aiuto
menu-about = Informazioni
navbar-edit-button = Modifica

# Help screen
help-title = Aiuto
help-back-to-viewer-button = Torna al visualizzatore

# Common labels
help-toc-title = Contenuti
help-tools-title = Strumenti disponibili
help-shortcuts-title = Scorciatoie da tastiera
help-usage-title = Come usare

# ─────────────────────────────────────────────────────────────────────────────
# Viewer Section
# ─────────────────────────────────────────────────────────────────────────────
help-section-viewer = Visualizzatore di immagini e video
help-viewer-role = Sfoglia e visualizza le tue immagini e i tuoi video. Naviga tra i file nella stessa cartella e regola la visualizzazione secondo le tue preferenze.

help-viewer-tool-navigation = Navigazione
help-viewer-tool-navigation-desc = Usa i pulsanti freccia o la tastiera per spostarti tra i file nella cartella.
help-viewer-tool-zoom = Zoom
help-viewer-tool-zoom-desc = Scorri con la rotella del mouse, usa i pulsanti +/- o inserisci direttamente una percentuale.
help-viewer-tool-pan = Sposta
help-viewer-tool-pan-desc = Quando ingrandito, fai clic e trascina l'immagine per muoverti.
help-viewer-tool-fit = Adatta alla finestra
help-viewer-tool-fit-desc = Ridimensiona automaticamente l'immagine per adattarla completamente alla finestra.
help-viewer-tool-fullscreen = Schermo intero
help-viewer-tool-fullscreen-desc = Vista immersiva con controlli che si nascondono automaticamente (ritardo configurabile nelle Impostazioni).
help-viewer-tool-delete = Elimina
help-viewer-tool-delete-desc = Rimuovi permanentemente il file corrente (spostato nel cestino di sistema, se disponibile).

help-viewer-key-navigate = Vai al file precedente/successivo
help-viewer-key-edit = Apri l'immagine nell'editor
help-viewer-key-fullscreen = Entra/esci da schermo intero
help-viewer-key-exit-fullscreen = Esci dalla modalità a schermo intero
help-viewer-key-info = Attiva/disattiva pannello informazioni file

help-mouse-title = Interazioni con il mouse
help-viewer-mouse-doubleclick = Doppio clic su immagine/video per attivare/disattivare schermo intero
help-viewer-mouse-wheel = Ingrandisci/riduci
help-viewer-mouse-drag = Sposta l'immagine quando ingrandita

# ─────────────────────────────────────────────────────────────────────────────
# Video Playback Section
# ─────────────────────────────────────────────────────────────────────────────
help-section-video = Riproduzione video
help-video-role = Riproduci file video con controlli di riproduzione completi. Regola il volume, cerca nella timeline e naviga fotogramma per fotogramma per un posizionamento preciso.

help-video-tool-playback = Riproduci/Pausa
help-video-tool-playback-desc = Avvia o interrompi la riproduzione video con il pulsante di riproduzione o il tasto Spazio.
help-video-tool-timeline = Timeline
help-video-tool-timeline-desc = Fai clic ovunque sulla barra di progresso per saltare a quella posizione.
help-video-tool-volume = Volume
help-video-tool-volume-desc = Trascina il cursore del volume o fai clic sull'icona dell'altoparlante per silenziare/attivare l'audio.
help-video-tool-loop = Ripeti
help-video-tool-loop-desc = Attiva per riavviare automaticamente il video quando termina.
help-video-tool-stepping = Navigazione fotogramma per fotogramma
help-video-tool-stepping-desc = Quando in pausa, spostati avanti o indietro di un fotogramma alla volta per una navigazione precisa.
help-video-tool-capture = Cattura fotogramma
help-video-tool-capture-desc = Salva il fotogramma video corrente come file immagine (si apre nell'editor).

help-video-key-playpause = Riproduci o metti in pausa il video
help-video-key-mute = Attiva/disattiva silenziamento audio
help-video-key-seek = Cerca indietro/avanti (durante la riproduzione)
help-video-key-volume = Aumenta/diminuisci volume
help-video-key-step-back = Indietreggia di un fotogramma (quando in pausa)
help-video-key-step-forward = Avanza di un fotogramma (quando in pausa)

# ─────────────────────────────────────────────────────────────────────────────
# Image Editor Section
# ─────────────────────────────────────────────────────────────────────────────
help-section-editor = Editor di immagini
help-editor-role = Apporta modifiche alle tue immagini: ruota, ritaglia un'area specifica o ridimensiona a dimensioni diverse.
help-editor-workflow = Tutte le modifiche sono non distruttive finché non salvi. Usa "Salva" per sovrascrivere l'originale, o "Salva come" per creare un nuovo file e preservare l'originale.

help-editor-rotate-title = Rotazione
help-editor-rotate-desc = Ruota o capovolgi l'immagine per correggere l'orientamento o creare effetti speculari.
help-editor-rotate-left = Ruota 90° in senso antiorario
help-editor-rotate-right = Ruota 90° in senso orario
help-editor-flip-h = Capovolgi orizzontalmente (specchio sinistra/destra)
help-editor-flip-v = Capovolgi verticalmente (specchio alto/basso)

help-editor-crop-title = Ritaglia
help-editor-crop-desc = Rimuovi le aree indesiderate selezionando la regione che vuoi mantenere.
help-editor-crop-ratios = Scegli un rapporto preimpostato (1:1 quadrato, 16:9 orizzontale, 9:16 verticale, 4:3 o 3:4 foto) o ritaglia liberamente.
help-editor-crop-usage = Trascina le maniglie per regolare la selezione, quindi fai clic su "Applica" per confermare.

help-editor-resize-title = Ridimensiona
help-editor-resize-desc = Cambia le dimensioni dell'immagine per renderla più grande o più piccola.
help-editor-resize-scale = Scala per percentuale (es. 50% per dimezzare le dimensioni)
help-editor-resize-dimensions = Inserisci larghezza e altezza esatte in pixel
help-editor-resize-lock = Blocca le proporzioni per mantenere le proporzioni
help-editor-resize-presets = Usa le preimpostazioni per dimensioni comuni (HD, Full HD, 4K...)

help-editor-light-title = Luce
help-editor-light-desc = Regola finemente la luminosità e il contrasto della tua immagine.
help-editor-light-brightness = Luminosità: schiarisci o scurisci l'immagine complessiva
help-editor-light-contrast = Contrasto: aumenta o diminuisci la differenza tra aree chiare e scure
help-editor-light-preview = Le modifiche vengono visualizzate in anteprima in tempo reale prima dell'applicazione

help-editor-save-title = Salvataggio
help-editor-save-overwrite = Salva: sovrascrive il file originale
help-editor-save-as = Salva come: crea un nuovo file (scegli posizione e formato)

help-editor-key-save = Salva le modifiche correnti
help-editor-key-undo = Annulla l'ultima modifica
help-editor-key-redo = Ripeti la modifica annullata
help-editor-key-cancel = Annulla tutte le modifiche ed esci

# ─────────────────────────────────────────────────────────────────────────────
# Frame Capture Section
# ─────────────────────────────────────────────────────────────────────────────
help-section-capture = Cattura fotogramma video
help-capture-role = Estrai qualsiasi fotogramma da un video e salvalo come file immagine. Perfetto per creare miniature o catturare momenti specifici.

help-capture-step1 = Riproduci o naviga il video fino al fotogramma desiderato
help-capture-step2 = Metti in pausa il video (usa la navigazione fotogramma per fotogramma per precisione)
help-capture-step3 = Fai clic sul pulsante della fotocamera nei controlli video
help-capture-step4 = Il fotogramma si apre nell'editor — salva come PNG, JPEG o WebP

help-capture-formats = Formati di esportazione supportati: PNG (senza perdita), JPEG (dimensione file più piccola), WebP (formato moderno con buona compressione).

# About screen
about-title = Informazioni
about-back-to-viewer-button = Torna al visualizzatore

about-section-app = Applicazione
about-app-name = { -app-name }
about-app-description = Visualizzatore leggero di immagini e video con editing di base delle immagini.

about-section-license = Licenza
about-license-name = Mozilla Public License 2.0 (MPL-2.0)
about-license-summary = Copyleft a livello di file: i file modificati devono essere condivisi sotto la stessa licenza. Compatibile con codice proprietario.

about-section-icon-license = Licenza icone
about-icon-license-name = Licenza icone { -app-name }
about-icon-license-summary = Tutte le icone (logo dell'applicazione e icone dell'interfaccia) possono essere ridistribuite solo non modificate per rappresentare { -app-name }.

about-section-credits = Riconoscimenti
about-credits-iced = Costruito con il toolkit GUI Iced
about-credits-ffmpeg = Riproduzione video alimentata da FFmpeg
about-credits-fluent = Internazionalizzazione da Project Fluent

about-section-links = Collegamenti
about-link-repository = Codice sorgente
about-link-issues = Segnala problemi

# Notifications
notification-save-success = Immagine salvata con successo
notification-save-error = Errore nel salvataggio dell'immagine
notification-frame-capture-success = Fotogramma catturato con successo
notification-frame-capture-error = Errore nella cattura del fotogramma
notification-delete-success = File eliminato con successo
notification-delete-error = Errore nell'eliminazione del file
notification-copy-success = Copiato negli appunti
notification-copy-error = Errore nella copia negli appunti
notification-config-save-error = Errore nel salvataggio delle impostazioni
notification-config-load-error = Errore nel caricamento delle impostazioni, uso dei valori predefiniti
notification-state-parse-error = Errore nella lettura dello stato dell'applicazione, uso dei valori predefiniti
notification-state-read-error = Errore nell'apertura del file di stato dell'applicazione
notification-state-path-error = Impossibile determinare il percorso dei dati dell'applicazione
notification-state-dir-error = Errore nella creazione della directory dei dati dell'applicazione
notification-state-write-error = Errore nel salvataggio dello stato dell'applicazione
notification-state-create-error = Errore nella creazione del file di stato dell'applicazione
notification-scan-dir-error = Errore nella scansione della directory
notification-editor-frame-error = Errore nell'apertura dell'editor con il fotogramma catturato
notification-editor-create-error = Errore nell'apertura dell'editor di immagini
notification-editor-load-error = Errore nel caricamento dell'immagine per la modifica
notification-video-editing-unsupported = La modifica video non è ancora supportata

# Metadata panel
metadata-panel-title = Informazioni file
metadata-section-file = File
metadata-section-camera = Fotocamera
metadata-section-exposure = Esposizione
metadata-section-video = Video
metadata-section-audio = Audio
metadata-section-gps = Posizione
metadata-label-dimensions = Dimensioni
metadata-label-file-size = Dimensione file
metadata-label-format = Formato
metadata-label-date-taken = Data di acquisizione
metadata-label-camera = Fotocamera
metadata-label-exposure = Esposizione
metadata-label-aperture = Apertura
metadata-label-iso = ISO
metadata-label-focal-length = Lunghezza focale
metadata-label-gps = Coordinate
metadata-label-codec = Codec
metadata-label-bitrate = Bitrate
metadata-label-duration = Durata
metadata-label-fps = Fotogrammi al secondo
metadata-value-unknown = Sconosciuto
navbar-info-button = Info
navbar-info-tooltip = Mostra informazioni file (I)
