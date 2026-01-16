// SPDX-License-Identifier: MPL-2.0
//! Handler methods for App message processing.
//!
//! This module contains the `handle_*` methods for App that process
//! async operation results (AI model downloads, validation, image loading, etc.).

use super::{notifications, update, App, Message};
use crate::diagnostics::{AIModel, AppOperation, AppStateEvent, Dimensions, ErrorType, WarningType};
use crate::media::{self, MediaData};
use crate::ui::image_editor;
use iced::Task;

impl App {
    /// Handles the result of applying AI deblur to an image.
    pub(super) fn handle_deblur_apply_completed(
        &mut self,
        result: Result<Box<image_rs::DynamicImage>, String>,
    ) -> Task<Message> {
        // Ignore results if shutting down
        if self.shutting_down {
            return Task::none();
        }

        // Calculate duration from start time
        // Truncation is safe: millis won't exceed u64::MAX for practical durations
        #[allow(clippy::cast_possible_truncation)]
        let duration_ms = self
            .deblur_started_at
            .take()
            .map_or(0, |start| start.elapsed().as_millis() as u64);

        let diagnostics_handle = self.diagnostics.handle();

        if let Some(editor) = self.image_editor.as_mut() {
            // Get image dimensions for diagnostics
            let (w, h) = (
                editor.working_image().width(),
                editor.working_image().height(),
            );
            // Estimate memory size (RGBA = 4 bytes per pixel)
            let file_size_bytes = u64::from(w) * u64::from(h) * 4;
            let dimensions = Some(Dimensions::new(w, h));

            match result {
                Ok(deblurred_image) => {
                    // Log operation with success
                    diagnostics_handle.log_operation(AppOperation::AIDeblurProcess {
                        duration_ms,
                        file_size_bytes,
                        dimensions,
                        success: true,
                    });
                    diagnostics_handle.log_state(AppStateEvent::EditorDeblurCompleted);

                    editor.apply_deblur_result(*deblurred_image);
                    self.notifications
                        .push(notifications::Notification::success(
                            "notification-deblur-apply-success",
                        ));
                }
                Err(e) => {
                    // Log operation with failure
                    diagnostics_handle.log_operation(AppOperation::AIDeblurProcess {
                        duration_ms,
                        file_size_bytes,
                        dimensions,
                        success: false,
                    });

                    editor.deblur_failed();
                    self.notifications.push(
                        notifications::Notification::error("notification-deblur-apply-error")
                            .with_error_type(ErrorType::AIModelError)
                            .with_arg("error", e),
                    );
                }
            }
        }
        Task::none()
    }

    /// Handles the result of applying AI upscale resize to an image.
    pub(super) fn handle_upscale_resize_completed(
        &mut self,
        result: Result<Box<image_rs::DynamicImage>, String>,
    ) -> Task<Message> {
        // Ignore results if shutting down
        if self.shutting_down {
            return Task::none();
        }

        // Calculate duration from start time
        // Truncation is safe: millis won't exceed u64::MAX for practical durations
        #[allow(clippy::cast_possible_truncation)]
        let duration_ms = self
            .upscale_started_at
            .take()
            .map_or(0, |start| start.elapsed().as_millis() as u64);

        let scale_factor = self.upscale_scale_factor.take().unwrap_or(1.0);
        let diagnostics_handle = self.diagnostics.handle();

        if let Some(editor) = self.image_editor.as_mut() {
            // Get original image dimensions for diagnostics
            let (w, h) = (
                editor.working_image().width(),
                editor.working_image().height(),
            );
            // Estimate memory size (RGBA = 4 bytes per pixel)
            let file_size_bytes = u64::from(w) * u64::from(h) * 4;
            let dimensions = Some(Dimensions::new(w, h));

            match result {
                Ok(upscaled_image) => {
                    // Log operation with success
                    diagnostics_handle.log_operation(AppOperation::AIUpscaleProcess {
                        duration_ms,
                        scale_factor,
                        file_size_bytes,
                        dimensions,
                        success: true,
                    });

                    // apply_upscale_resize_result clears the processing state
                    editor.apply_upscale_resize_result(*upscaled_image);
                    self.notifications
                        .push(notifications::Notification::success(
                            "notification-upscale-resize-success",
                        ));
                }
                Err(e) => {
                    // Log operation with failure
                    diagnostics_handle.log_operation(AppOperation::AIUpscaleProcess {
                        duration_ms,
                        scale_factor,
                        file_size_bytes,
                        dimensions,
                        success: false,
                    });

                    // Clear processing state on error
                    editor.clear_upscale_processing();
                    self.notifications.push(
                        notifications::Notification::error("notification-upscale-resize-error")
                            .with_error_type(ErrorType::AIModelError)
                            .with_arg("error", e),
                    );
                }
            }
        }
        Task::none()
    }

    /// Handles the metadata Save As dialog result.
    pub(super) fn handle_metadata_save_as(&mut self, path: &std::path::Path) -> Task<Message> {
        use crate::media::metadata_writer;

        // First, copy the original file to the new location
        // Use media_navigator as single source of truth for current path
        if let Some(source_path) = self.media_navigator.current_media_path() {
            if let Err(_e) = std::fs::copy(source_path, path) {
                self.notifications.push(
                    notifications::Notification::error("notification-metadata-save-error")
                        .with_error_type(ErrorType::IoError),
                );
                return Task::none();
            }
        } else {
            self.notifications.push(
                notifications::Notification::error("notification-metadata-save-error")
                    .with_error_type(ErrorType::IoError),
            );
            return Task::none();
        }

        // Then write metadata to the new file
        if let Some(editor_state) = self.metadata_editor_state.as_ref() {
            match metadata_writer::write_exif(path, editor_state.editable_metadata()) {
                Ok(()) => {
                    // Remember the save directory
                    self.persisted.set_last_save_directory_from_file(path);
                    if let Some(key) = self.persisted.save() {
                        self.notifications.push(
                            notifications::Notification::warning(&key)
                                .with_warning_type(WarningType::ConfigurationIssue),
                        );
                    }

                    // Refresh metadata display
                    self.current_metadata = media::metadata::extract_metadata(path);

                    // Exit edit mode
                    self.metadata_editor_state = None;

                    // Show success notification
                    self.notifications
                        .push(notifications::Notification::success(
                            "notification-metadata-save-success",
                        ));
                }
                Err(_e) => {
                    // Clean up: remove the copied file if write failed
                    let _ = std::fs::remove_file(path);
                    self.notifications.push(
                        notifications::Notification::error("notification-metadata-save-error")
                            .with_error_type(ErrorType::IoError),
                    );
                }
            }
        }
        Task::none()
    }

    /// Handles the result of deblur model download.
    pub(super) fn handle_deblur_download_completed(
        &mut self,
        result: Result<(), String>,
    ) -> Task<Message> {
        // Don't start validation if shutting down
        if self.shutting_down {
            return Task::none();
        }

        match result {
            Ok(()) => {
                // Log state event for diagnostics
                self.diagnostics
                    .handle()
                    .log_state(AppStateEvent::ModelDownloadCompleted {
                        model: AIModel::Deblur,
                    });

                // Download succeeded - start validation
                self.settings
                    .set_deblur_model_status(media::deblur::ModelStatus::Validating);

                // Start validation task using spawn_blocking for CPU-intensive ONNX inference
                let cancel_token = self.cancellation_token.clone();
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            // Create manager and try to load + validate the model
                            let mut manager = media::deblur::DeblurManager::new();
                            manager.load_session(Some(&cancel_token))?;
                            media::deblur::validate_model(&mut manager, Some(&cancel_token))?;
                            Ok::<(), media::deblur::DeblurError>(())
                        })
                        .await
                        .map_err(|e| media::deblur::DeblurError::InferenceFailed(e.to_string()))?
                    },
                    |result: media::deblur::DeblurResult<()>| match result {
                        Ok(()) => Message::DeblurValidationCompleted {
                            result: Ok(()),
                            is_startup: false,
                        },
                        Err(e) => Message::DeblurValidationCompleted {
                            result: Err(e.to_string()),
                            is_startup: false,
                        },
                    },
                )
            }
            Err(e) => {
                // Log state event for diagnostics
                self.diagnostics
                    .handle()
                    .log_state(AppStateEvent::ModelDownloadFailed {
                        model: AIModel::Deblur,
                        reason: e.clone(),
                    });

                // Download failed
                self.settings
                    .set_deblur_model_status(media::deblur::ModelStatus::Error(e.clone()));
                self.notifications.push(
                    notifications::Notification::error("notification-deblur-download-error")
                        .with_error_type(ErrorType::AIModelError)
                        .with_arg("error", e),
                );
                Task::none()
            }
        }
    }

    /// Handles the result of deblur model validation.
    ///
    /// When `is_startup` is true, success notifications are suppressed (the user expects
    /// the feature to work from previous sessions). Failure notifications are always shown.
    /// If the app is shutting down, the result is ignored.
    pub(super) fn handle_deblur_validation_completed(
        &mut self,
        result: Result<(), String>,
        is_startup: bool,
    ) -> Task<Message> {
        // Ignore validation results if the app is shutting down
        if self.shutting_down {
            return Task::none();
        }

        match result {
            Ok(()) => {
                // Validation succeeded - enable deblur and persist state
                self.settings
                    .set_deblur_model_status(media::deblur::ModelStatus::Ready);
                self.settings.set_enable_deblur(true);
                self.persisted.enable_deblur = true;
                if let Some(key) = self.persisted.save() {
                    self.notifications.push(
                        notifications::Notification::warning(&key)
                            .with_warning_type(WarningType::ConfigurationIssue),
                    );
                }
                // Only show success notification for user-initiated activation, not startup
                if !is_startup {
                    self.notifications
                        .push(notifications::Notification::success(
                            "notification-deblur-ready",
                        ));
                }
            }
            Err(e) => {
                // Validation failed - reset enable_deblur, delete the model and show error
                self.settings
                    .set_deblur_model_status(media::deblur::ModelStatus::Error(e.clone()));
                self.settings.set_enable_deblur(false);
                self.persisted.enable_deblur = false;
                if let Some(key) = self.persisted.save() {
                    self.notifications.push(
                        notifications::Notification::warning(&key)
                            .with_warning_type(WarningType::ConfigurationIssue),
                    );
                }
                // Delete the invalid model file
                let _ = std::fs::remove_file(media::deblur::get_model_path());
                self.notifications.push(
                    notifications::Notification::error("notification-deblur-validation-error")
                        .with_error_type(ErrorType::AIModelError)
                        .with_arg("error", e),
                );
            }
        }
        Task::none()
    }

    /// Handles the result of upscale model download.
    pub(super) fn handle_upscale_download_completed(
        &mut self,
        result: Result<(), String>,
    ) -> Task<Message> {
        // Don't start validation if shutting down
        if self.shutting_down {
            return Task::none();
        }

        match result {
            Ok(()) => {
                // Log state event for diagnostics
                self.diagnostics
                    .handle()
                    .log_state(AppStateEvent::ModelDownloadCompleted {
                        model: AIModel::Upscale,
                    });

                // Download succeeded - start validation
                self.settings
                    .set_upscale_model_status(media::upscale::UpscaleModelStatus::Validating);

                // Start validation task using spawn_blocking for CPU-intensive ONNX inference
                let cancel_token = self.cancellation_token.clone();
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            let mut manager = media::upscale::UpscaleManager::new();
                            manager.load_session(Some(&cancel_token))?;
                            media::upscale::validate_model(&mut manager, Some(&cancel_token))?;
                            Ok::<(), media::upscale::UpscaleError>(())
                        })
                        .await
                        .map_err(|e| media::upscale::UpscaleError::InferenceFailed(e.to_string()))?
                    },
                    |result: media::upscale::UpscaleResult<()>| match result {
                        Ok(()) => Message::UpscaleValidationCompleted {
                            result: Ok(()),
                            is_startup: false,
                        },
                        Err(e) => Message::UpscaleValidationCompleted {
                            result: Err(e.to_string()),
                            is_startup: false,
                        },
                    },
                )
            }
            Err(e) => {
                // Log state event for diagnostics
                self.diagnostics
                    .handle()
                    .log_state(AppStateEvent::ModelDownloadFailed {
                        model: AIModel::Upscale,
                        reason: e.clone(),
                    });

                // Download failed
                self.settings
                    .set_upscale_model_status(media::upscale::UpscaleModelStatus::Error(e.clone()));
                self.notifications.push(
                    notifications::Notification::error("notification-upscale-download-error")
                        .with_error_type(ErrorType::AIModelError)
                        .with_arg("error", e),
                );
                Task::none()
            }
        }
    }

    /// Handles the result of upscale model validation.
    pub(super) fn handle_upscale_validation_completed(
        &mut self,
        result: Result<(), String>,
        is_startup: bool,
    ) -> Task<Message> {
        // Ignore validation results if the app is shutting down
        if self.shutting_down {
            return Task::none();
        }

        match result {
            Ok(()) => {
                // Validation succeeded - enable upscale and persist state
                self.settings
                    .set_upscale_model_status(media::upscale::UpscaleModelStatus::Ready);
                self.settings.set_enable_upscale(true);
                self.persisted.enable_upscale = true;
                if let Some(key) = self.persisted.save() {
                    self.notifications.push(
                        notifications::Notification::warning(&key)
                            .with_warning_type(WarningType::ConfigurationIssue),
                    );
                }
                // Only show success notification for user-initiated activation, not startup
                if !is_startup {
                    self.notifications
                        .push(notifications::Notification::success(
                            "notification-upscale-ready",
                        ));
                }
            }
            Err(e) => {
                // Validation failed - reset enable_upscale, delete the model and show error
                self.settings
                    .set_upscale_model_status(media::upscale::UpscaleModelStatus::Error(e.clone()));
                self.settings.set_enable_upscale(false);
                self.persisted.enable_upscale = false;
                if let Some(key) = self.persisted.save() {
                    self.notifications.push(
                        notifications::Notification::warning(&key)
                            .with_warning_type(WarningType::ConfigurationIssue),
                    );
                }
                // Delete the invalid model file
                let _ = std::fs::remove_file(media::upscale::get_model_path());
                self.notifications.push(
                    notifications::Notification::error("notification-upscale-validation-error")
                        .with_error_type(ErrorType::AIModelError)
                        .with_arg("error", e),
                );
            }
        }
        Task::none()
    }

    /// Handles async image loading result for the editor.
    // Allow too_many_lines: sequential async result handling with navigation logic.
    // Marginal benefit from extraction (111 lines vs 100 limit).
    #[allow(clippy::too_many_lines)]
    pub(super) fn handle_image_editor_loaded(
        &mut self,
        result: Result<MediaData, crate::error::Error>,
    ) -> Task<Message> {
        use crate::ui::viewer::{LoadOrigin, NavigationDirection};

        if let Ok(media_data) = result {
            // Editor only supports images - videos are skipped during navigation
            let MediaData::Image(image_data) = media_data else {
                // Should not happen: peek_*_image() only returns images
                return Task::none();
            };

            // Get the tentative path from viewer and confirm navigation
            let Some(path) = self.viewer.current_media_path.clone() else {
                return Task::none();
            };

            // Confirm navigation in MediaNavigator (pessimistic update)
            self.media_navigator.confirm_navigation(&path);

            // Check if we skipped any files during navigation
            let load_origin = std::mem::take(&mut self.viewer.load_origin);
            if let LoadOrigin::Navigation { skipped_files, .. } = load_origin {
                if !skipped_files.is_empty() {
                    let files_text =
                        update::format_skipped_files_message(&self.i18n, &skipped_files);
                    self.notifications.push(
                        notifications::Notification::warning(
                            "notification-skipped-corrupted-files",
                        )
                        .with_warning_type(WarningType::UnsupportedFormat)
                        .with_arg("files", files_text)
                        .auto_dismiss(std::time::Duration::from_secs(8)),
                    );
                }
            }

            // Create a new ImageEditorState with the loaded image
            match image_editor::State::new(path, &image_data) {
                Ok(new_editor_state) => {
                    self.image_editor = Some(new_editor_state);
                }
                Err(_) => {
                    self.notifications.push(
                        notifications::Notification::error("notification-editor-create-error")
                            .with_error_type(ErrorType::InternalError),
                    );
                }
            }
            Task::none()
        } else {
            // Get the failed filename from viewer's tentative path
            let failed_filename = self
                .viewer
                .current_media_path
                .as_ref()
                .and_then(|p| p.file_name())
                .map_or_else(
                    || "unknown".to_string(),
                    |n| n.to_string_lossy().to_string(),
                );

            // Handle based on load origin
            let load_origin = std::mem::take(&mut self.viewer.load_origin);
            match load_origin {
                LoadOrigin::Navigation {
                    direction,
                    skip_attempts,
                    mut skipped_files,
                } => {
                    // Add failed file to the list
                    skipped_files.push(failed_filename);
                    let new_attempts = skip_attempts + 1;
                    let max_attempts = self.viewer.max_skip_attempts;

                    if new_attempts <= max_attempts.value() {
                        // Use peek_nth_*_image with skip_count to find the next file
                        // without modifying navigator state. State is only updated
                        // via confirm_navigation after successful load.
                        let next_path = match direction {
                            NavigationDirection::Next => self
                                .media_navigator
                                .peek_nth_next_image(new_attempts as usize),
                            NavigationDirection::Previous => self
                                .media_navigator
                                .peek_nth_previous_image(new_attempts as usize),
                        };

                        if let Some(path) = next_path {
                            // Set tentative path for next retry
                            self.viewer.current_media_path = Some(path.clone());

                            // Auto-skip: retry navigation in the same direction
                            self.viewer.set_load_origin(LoadOrigin::Navigation {
                                direction,
                                skip_attempts: new_attempts,
                                skipped_files,
                            });

                            Task::perform(
                                async move { media::load_media(&path) },
                                Message::ImageEditorLoaded,
                            )
                        } else {
                            // No more images to navigate to
                            let files_text =
                                update::format_skipped_files_message(&self.i18n, &skipped_files);
                            self.notifications.push(
                                notifications::Notification::warning(
                                    "notification-skipped-corrupted-files",
                                )
                                .with_warning_type(WarningType::UnsupportedFormat)
                                .with_arg("files", files_text)
                                .auto_dismiss(std::time::Duration::from_secs(8)),
                            );
                            Task::none()
                        }
                    } else {
                        // Max attempts reached: show grouped notification
                        let files_text =
                            update::format_skipped_files_message(&self.i18n, &skipped_files);
                        self.notifications.push(
                            notifications::Notification::warning(
                                "notification-skipped-corrupted-files",
                            )
                            .with_warning_type(WarningType::UnsupportedFormat)
                            .with_arg("files", files_text)
                            .auto_dismiss(std::time::Duration::from_secs(8)),
                        );
                        Task::none()
                    }
                }
                LoadOrigin::DirectOpen => {
                    // This case should not happen in the editor since all loads
                    // come from navigation. Kept as defensive fallback.
                    #[cfg(debug_assertions)]
                    eprintln!("[WARN] Unexpected DirectOpen in image editor error handler");
                    self.notifications.push(
                        notifications::Notification::error("notification-load-error")
                            .with_error_type(ErrorType::DecodeError),
                    );
                    Task::none()
                }
            }
        }
    }
}
