use crate::dictionary::Dictionary;
use crate::models::WordDefinition;
use eframe::egui;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

pub struct DictNaviApp {
    dictionary: Arc<Dictionary>,
    search_term: String,
    search_result: Option<WordDefinition>,
    error_message: Option<String>,
    search_history: Vec<String>,
    // Fields related to autocomplete
    all_words: Option<Vec<String>>,
    suggestions: Vec<String>,
    selected_index: Option<usize>,
    show_suggestions: bool,
    keyboard_navigated: bool, // Whether keyboard navigation is being used
    // Fields related to settings menu
    show_settings_menu: bool,
    sync_status: Option<String>, // Status message for index building
    is_building_index: Arc<Mutex<bool>>, // Whether index is being built
    build_result: Arc<Mutex<Option<String>>>, // Result of index building
}

impl DictNaviApp {
    pub fn new(dictionary: Dictionary) -> Self {
        Self {
            dictionary: Arc::new(dictionary),
            search_term: String::new(),
            search_result: None,
            error_message: None,
            search_history: Vec::new(),
            all_words: None,
            suggestions: Vec::new(),
            selected_index: None,
            show_suggestions: false,
            keyboard_navigated: false,
            show_settings_menu: false,
            sync_status: None,
            is_building_index: Arc::new(Mutex::new(false)),
            build_result: Arc::new(Mutex::new(None)),
        }
    }

    // Load all words list (lazy loading)
    fn load_all_words(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.all_words.is_none() {
            self.all_words = Some(self.dictionary.list_words()?);
        }
        Ok(())
    }

    // Filter words based on input
    fn update_suggestions(&mut self) {
        if self.search_term.is_empty() {
            self.suggestions.clear();
            self.show_suggestions = false;
            self.selected_index = None;
            return;
        }

        if let Err(_) = self.load_all_words() {
            self.suggestions.clear();
            self.show_suggestions = false;
            return;
        }

        let search_lower = self.search_term.to_lowercase();
        if let Some(ref words) = self.all_words {
            self.suggestions = words
                .iter()
                .filter(|word| word.to_lowercase().starts_with(&search_lower))
                .take(10) // Display up to 10 suggestions
                .cloned()
                .collect();
            
            self.show_suggestions = !self.suggestions.is_empty();
            // Reset selected index
            if self.selected_index.is_some() && self.selected_index.unwrap() >= self.suggestions.len() {
                self.selected_index = None;
            }
        }
    }

    fn search_word(&mut self) {
        if self.search_term.is_empty() {
            return;
        }

        match self.dictionary.lookup_word(&self.search_term) {
            Ok(Some(definition)) => {
                self.search_result = Some(definition);
                self.error_message = None;
            }
            Ok(None) => {
                self.search_result = None;
                self.error_message = Some(format!("Word '{}' not found", self.search_term));
            }
            Err(e) => {
                self.search_result = None;
                self.error_message = Some(format!("Error looking up word: {}", e));
            }
        }
    }

    fn clear_search(&mut self) {
        self.search_term.clear();
        self.search_result = None;
        self.error_message = None;
        self.show_suggestions = false;
        self.selected_index = None;
        self.suggestions.clear();
        self.keyboard_navigated = false;
    }
}

impl eframe::App for DictNaviApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check the result of asynchronous index building
        if let Ok(mut result) = self.build_result.lock() {
            if let Some(status) = result.take() {
                self.sync_status = Some(status);
                // Clear the cached word list, force reload
                self.all_words = None;
            }
        }
        
        // If index is being built, request repaint periodically to update UI
        if *self.is_building_index.lock().unwrap() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        
        // Top bar: settings button
        let mut settings_button_rect = None;
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Settings button
                    let settings_button = ui.button("⚙");
                    if settings_button.clicked() {
                        self.show_settings_menu = !self.show_settings_menu;
                    }
                    settings_button_rect = Some(settings_button.rect);
                });
            });
        });
        
        // Settings dropdown menu (displayed outside the top bar)
        if self.show_settings_menu {
            if let Some(button_rect) = settings_button_rect {
                let menu_pos = egui::pos2(
                    button_rect.right() - 150.0, // Menu right aligned to button
                    button_rect.bottom(),
                );
                
                let response = egui::Area::new(egui::Id::new("settings_menu"))
                    .order(egui::Order::Foreground)
                    .fixed_pos(menu_pos)
                    .show(ctx, |ui| {
                        egui::Frame::popup(ui.style())
                            .fill(ui.style().visuals.extreme_bg_color)
                            .show(ui, |ui| {
                                ui.set_min_width(150.0);
                                
                                // Build index option (asynchronous)
                                let is_building = *self.is_building_index.lock().unwrap();
                                let button_text = if is_building {
                                    "Building index..."
                                } else {
                                    "Build index"
                                };
                                
                                let button = ui.add_enabled(!is_building, egui::Button::new(button_text));
                                if button.clicked() && !is_building {
                                    self.show_settings_menu = false;
                                    *self.is_building_index.lock().unwrap() = true;
                                    self.sync_status = Some("Building index, please wait...".to_string());
                                    
                                    let dictionary = Arc::clone(&self.dictionary);
                                    let status_arc = Arc::clone(&self.is_building_index);
                                    let result_arc = Arc::clone(&self.build_result);
                                    
                                    tokio::spawn(async move {
                                        let result = dictionary.build_index_async().await;
                                        
                                        *status_arc.lock().unwrap() = false;
                                        
                                        match result {
                                            Ok((doc_count, json_count)) => {
                                                *result_arc.lock().unwrap() = Some(format!(
                                                    "Index built successfully! Indexed {} documents (total {} files)",
                                                    doc_count, json_count
                                                ));
                                            }
                                            Err(e) => {
                                                *result_arc.lock().unwrap() = Some(format!("Index building failed: {}", e));
                                            }
                                        }
                                    });
                                }
                            });
                    });
                
                // Click outside the area to close the menu
                if ctx.input(|i| i.pointer.primary_clicked()) {
                    let click_pos = ctx.input(|i| i.pointer.interact_pos());
                    if let Some(pos) = click_pos {
                        if !response.response.rect.contains(pos) && !button_rect.contains(pos) {
                            self.show_settings_menu = false;
                        }
                    }
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Display sync status message
            if let Some(status) = &self.sync_status {
                let is_success = status.contains("successfully");
                let status_clone = status.clone();
                ui.horizontal(|ui| {
                    if is_success {
                        ui.colored_label(egui::Color32::from_rgb(0, 150, 0), &status_clone);
                    } else {
                        ui.colored_label(egui::Color32::RED, &status_clone);
                    }
                    if ui.button("✕").clicked() {
                        self.sync_status = None;
                    }
                });
                ui.separator();
            }

            // Search section with autocomplete
            ui.horizontal(|ui| {
                // Detect input change
                let previous_search_term = self.search_term.clone();
                let text_response = ui.text_edit_singleline(&mut self.search_term);
                
                // If input changed, update suggestions list
                if previous_search_term != self.search_term {
                    self.update_suggestions();
                    self.keyboard_navigated = false; // Reset keyboard navigation flag when input changed
                }

                // Handle keyboard events (when input box has focus)
                if text_response.has_focus() {
                    let input = ui.input(|i| i.clone());
                    
                    // Handle down arrow
                    if input.key_pressed(egui::Key::ArrowDown) {
                        self.keyboard_navigated = true; // Mark keyboard navigation as used
                        if let Some(current_idx) = self.selected_index {
                            if current_idx + 1 < self.suggestions.len() {
                                self.selected_index = Some(current_idx + 1);
                            } else {
                                self.selected_index = Some(0);
                            }
                        } else if !self.suggestions.is_empty() {
                            self.selected_index = Some(0);
                        }
                        // Ensure suggestions list is displayed
                        if !self.suggestions.is_empty() {
                            self.show_suggestions = true;
                        }
                        text_response.request_focus(); // Keep focus
                    } 
                    // Handle up arrow
                    else if input.key_pressed(egui::Key::ArrowUp) {
                        self.keyboard_navigated = true; // Mark keyboard navigation as used
                        if let Some(current_idx) = self.selected_index {
                            if current_idx > 0 {
                                self.selected_index = Some(current_idx - 1);
                            } else {
                                self.selected_index = Some(self.suggestions.len().saturating_sub(1));
                            }
                        } else if !self.suggestions.is_empty() {
                            self.selected_index = Some(self.suggestions.len().saturating_sub(1));
                        }
                        // Ensure suggestions list is displayed
                        if !self.suggestions.is_empty() {
                            self.show_suggestions = true;
                        }
                        text_response.request_focus(); // Keep focus
                    } 
                    // Handle Enter key
                    else if input.key_pressed(egui::Key::Enter) {
                        // If there is a selected suggestion, use it for query
                        if let Some(idx) = self.selected_index {
                            if idx < self.suggestions.len() {
                                self.search_term = self.suggestions[idx].clone();
                                self.show_suggestions = false;
                                self.selected_index = None;
                                self.keyboard_navigated = false; // Reset flag
                                self.search_word();
                            }
                        } else if !self.search_term.is_empty() {
                            // If no suggestion is selected, query the current input
                            self.show_suggestions = false;
                            self.search_word();
                        }
                    } 
                    // Handle Escape key
                    else if input.key_pressed(egui::Key::Escape) {
                        self.show_suggestions = false;
                        self.selected_index = None;
                        self.keyboard_navigated = false; // Reset flag
                    }
                }

                let mut suggestions_rect = None;
                let clicked_suggestion = RefCell::new(None);
                let hovered_index = RefCell::new(None);
                
                if self.show_suggestions && !self.suggestions.is_empty() {
                    let text_rect = text_response.rect;
                    let suggestions_clone = self.suggestions.clone();
                    let selected_idx = self.selected_index;
                    
                    let below_text = egui::Rect::from_min_size(
                        text_rect.left_bottom(),
                        egui::vec2(text_rect.width(), 200.0),
                    );

                    let area_response = egui::Area::new(ui.id().with("suggestions"))
                        .order(egui::Order::Foreground)
                        .fixed_pos(below_text.min)
                        .show(ctx, |ui| {
                            let frame_response = egui::Frame::popup(ui.style())
                                .fill(ui.style().visuals.extreme_bg_color)
                                .show(ui, |ui| {
                                    egui::ScrollArea::vertical()
                                        .max_height(200.0)
                                        .show(ui, |ui| {
                                            for (idx, suggestion) in suggestions_clone.iter().enumerate() {
                                                let is_selected = selected_idx == Some(idx);
                                                
                                                let mut label_text = egui::RichText::new(suggestion);
                                                if is_selected {
                                                    label_text = label_text.background_color(
                                                        ui.style().visuals.selection.bg_fill
                                                    );
                                                }

                                                let response = ui.selectable_label(is_selected, label_text);
                                                
                                                // Collect clicked index
                                                if response.clicked() {
                                                    *clicked_suggestion.borrow_mut() = Some(idx);
                                                }
                                                
                                                // Collect hovered index
                                                if response.hovered() {
                                                    *hovered_index.borrow_mut() = Some(idx);
                                                }
                                            }
                                        });
                                });
                            frame_response.response.rect
                        });
                    suggestions_rect = Some(area_response.response.rect);
                }
                
                // Handle clicked suggestion
                if let Some(idx) = clicked_suggestion.into_inner() {
                    if idx < self.suggestions.len() {
                        self.search_term = self.suggestions[idx].clone();
                        self.show_suggestions = false;
                        self.selected_index = None;
                        self.keyboard_navigated = false; // Reset flag
                        self.search_word();
                    }
                }
                
                // Handle hover highlight (only applied when keyboard navigation is not used)
                if !self.keyboard_navigated {
                    if let Some(idx) = hovered_index.into_inner() {
                        self.selected_index = Some(idx);
                    }
                }

                // Click outside the area to close the dropdown list
                if ui.input(|i| i.pointer.primary_clicked()) {
                    let click_pos = ui.input(|i| i.pointer.interact_pos());
                    if let Some(pos) = click_pos {
                        let in_text_box = text_response.rect.contains(pos);
                        let in_suggestions = suggestions_rect.map_or(false, |rect| rect.contains(pos));
                        
                        if !in_text_box && !in_suggestions {
                            self.show_suggestions = false;
                            self.selected_index = None;
                        }
                    }
                }

                if ui.button("Search").clicked() {
                    self.show_suggestions = false;
                    self.selected_index = None;
                    self.search_word();
                }

                if ui.button("Clear").clicked() {
                    self.clear_search();
                }
            });

            // Show search history as buttons
            if !self.search_history.is_empty() {
                ui.separator();
                ui.label("Recent:");
                ui.horizontal_wrapped(|ui| {
                    // Create a temporary vector to avoid borrowing issues
                    let recent_words: Vec<String> =
                        self.search_history.iter().rev().take(10).cloned().collect();
                    for word in recent_words {
                        if ui.button(&word).clicked() {
                            self.search_term = word.clone();
                            self.search_word();
                        }
                    }
                });
            }

            ui.separator();

            // Display results or errors in a scrollable area
            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(error) = &self.error_message {
                    ui.colored_label(egui::Color32::RED, error);
                } else if let Some(definition) = &self.search_result {
                    // Add search result to search history
                    if !self.search_history.contains(&definition.word) {
                        self.search_history.push(definition.word.clone());
                    }

                    // Display word information
                    ui.heading(&definition.word);
                    if let Some(phonetic) = &definition.phonetic {
                        if !phonetic.is_empty() {
                            ui.label(format!("/{}/", phonetic));
                        }
                    }

                    // Display concise definition if available with better styling
                    if let Some(concise_def) = &definition.concise_definition {
                        ui.horizontal(|ui| {
                            ui.colored_label(egui::Color32::from_rgb(0, 100, 0), concise_def);
                        });
                    }

                    ui.separator();

                    // Display meanings
                    if let Some(meanings) = &definition.meanings {
                        for (i, meaning) in meanings.iter().enumerate() {
                            // Part of speech with color
                            ui.colored_label(
                                egui::Color32::DARK_BLUE,
                                format!("{}. {}", i + 1, meaning.part_of_speech),
                            );

                            // English explanation
                            ui.label(format!("{}", meaning.explanation_en));
                            // Transaltion explanation with color
                            if let Some(explanation_cn) = &meaning.explanation_cn {
                                ui.colored_label(
                                    egui::Color32::from_rgb(0, 100, 0),
                                    explanation_cn,
                                );
                            }

                            // English example with italic style
                            if let Some(example) = &meaning.example_en {
                                ui.horizontal(|ui| {
                                    ui.add_space(10.0);
                                    ui.label(egui::RichText::new(example).italics());
                                });
                            }

                            // Chinese example with color and italic style
                            if let Some(example_cn) = &meaning.example_cn {
                                ui.horizontal(|ui| {
                                    ui.add_space(10.0);
                                    ui.label(
                                        egui::RichText::new(example_cn)
                                            .color(egui::Color32::from_rgb(0, 100, 0)).
                                            italics()
                                    );
                                });
                            }

                            ui.add_space(10.0);
                        }
                    }

                    ui.separator();

                    if let Some(comparisons) = &definition.comparisons {
                        for (_, comparison) in comparisons.iter().enumerate() {
                            // English comparison word
                            ui.label(format!("  {}", comparison.word));
                            if let Some(analysis) = &comparison.analysis {
                                ui.label(
                                    egui::RichText::new(format!("{}", analysis))
                                        .color(egui::Color32::from_rgb(0, 100, 0)),
                                );
                            }
                            ui.add_space(10.0);
                        }
                    }
                } else if !self.search_term.is_empty() {
                    ui.label("Enter a word and click Search to look it up.");
                }
            });
        });
    }
}
