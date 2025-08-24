use crate::error::{LofiTurtleError, Result};
use image::{self, GenericImageView};
use lofty::{prelude::*, probe::Probe};
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// ASCII characters for different brightness levels (darkest to brightest)
const ASCII_CHARS: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

/// Cache key for ASCII art generation
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct AsciiArtCacheKey {
    file_path: String,
    width: u32,
    height: u32,
    use_color: bool,
}

/// Global cache for ASCII art to avoid regeneration
type AsciiArtCache = Arc<Mutex<HashMap<AsciiArtCacheKey, String>>>;

/// Configuration for album art display
#[derive(Debug, Clone)]
pub struct AlbumArtConfig {
    pub width: u32,
    pub height: u32,
    pub show_art: bool,
    pub use_color: bool,
}

impl Default for AlbumArtConfig {
    fn default() -> Self {
        Self {
            width: 40,
            height: 20,
            show_art: true,
            use_color: false,
        }
    }
}

impl AlbumArtConfig {

    /// Builder pattern for configuration
    pub fn builder() -> AlbumArtConfigBuilder {
        AlbumArtConfigBuilder::default()
    }
}

/// Builder for AlbumArtConfig
#[derive(Default)]
pub struct AlbumArtConfigBuilder {
    width: Option<u32>,
    height: Option<u32>,
    show_art: Option<bool>,
    use_color: Option<bool>,
}

impl AlbumArtConfigBuilder {
    pub fn show_art(mut self, show_art: bool) -> Self {
        self.show_art = Some(show_art);
        self
    }

    pub fn build(self) -> AlbumArtConfig {
        AlbumArtConfig {
            width: self.width.unwrap_or(40),
            height: self.height.unwrap_or(20),
            show_art: self.show_art.unwrap_or(false),
            use_color: self.use_color.unwrap_or(false),
        }
    }
}

/// Album art extractor and renderer with caching
pub struct AlbumArtRenderer {
    config: AlbumArtConfig,
    /// Performance optimization: Cache ASCII art to avoid regeneration
    ascii_cache: AsciiArtCache,
}

impl AlbumArtRenderer {
    /// Create a new album art renderer with the given configuration
    pub fn new(config: AlbumArtConfig) -> Self {
        Self { 
            config,
            ascii_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Extract album art from an audio file
    pub fn extract_album_art<P: AsRef<Path>>(&self, file_path: P) -> Result<Option<Vec<u8>>> {
        let tagged_file = Probe::open(file_path.as_ref())
            .map_err(|e| LofiTurtleError::UnsupportedFormat(format!("Failed to probe file: {}", e)))?
            .read()
            .map_err(|e| LofiTurtleError::UnsupportedFormat(format!("Failed to read file: {}", e)))?;

        // Try to get album art from the primary tag
        if let Some(tag) = tagged_file.primary_tag() {
            if let Some(picture) = tag.pictures().first() {
                return Ok(Some(picture.data().to_vec()));
            }
        }

        // Try other tags if primary tag doesn't have album art
        for tag in tagged_file.tags() {
            if let Some(picture) = tag.pictures().first() {
                return Ok(Some(picture.data().to_vec()));
            }
        }

        Ok(None)
    }

    /// Convert image data to ASCII art
    pub fn image_to_ascii(&self, image_data: &[u8]) -> Result<String> {
        if !self.config.show_art {
            return Ok(String::new());
        }

        let image = image::load_from_memory(image_data)
            .map_err(|e| LofiTurtleError::Configuration(format!("Failed to load image: {}", e)))?;

        let resized = image.resize_exact(
            self.config.width,
            self.config.height,
            image::imageops::FilterType::Lanczos3,
        );

        let rgba_image = resized.to_rgba8();
        let mut ascii_art = String::new();

        for y in 0..self.config.height {
            for x in 0..self.config.width {
                let pixel = rgba_image.get_pixel(x, y);
                let brightness = self.calculate_brightness(pixel.0);
                let char_index = (brightness * (ASCII_CHARS.len() - 1) as f32) as usize;
                ascii_art.push(ASCII_CHARS[char_index]);
            }
            ascii_art.push('\n');
        }

        Ok(ascii_art)
    }

    /// Calculate brightness of a pixel (0.0 = black, 1.0 = white)
    fn calculate_brightness(&self, rgba: [u8; 4]) -> f32 {
        // Use luminance formula: 0.299*R + 0.587*G + 0.114*B
        let r = rgba[0] as f32 / 255.0;
        let g = rgba[1] as f32 / 255.0;
        let b = rgba[2] as f32 / 255.0;
        let alpha = rgba[3] as f32 / 255.0;

        let luminance = 0.299 * r + 0.587 * g + 0.114 * b;
        luminance * alpha // Factor in alpha channel
    }



    /// Render album art as ASCII art
    pub fn render_album_art(&self, image_data: &[u8]) -> Result<String> {
        if !self.config.show_art {
            return Ok(String::new());
        }

        // Always use ASCII art
        self.image_to_ascii(image_data)
    }


    /// Generate a placeholder ASCII art when no album art is available
    pub fn generate_placeholder(&self) -> String {
        if !self.config.show_art {
            return String::new();
        }

        let mut placeholder = String::new();
        let center_x = self.config.width / 2;
        let center_y = self.config.height / 2;

        for y in 0..self.config.height {
            for x in 0..self.config.width {
                // Create a simple musical note pattern
                if self.is_music_note_pattern(x, y, center_x, center_y) {
                    placeholder.push('♪');
                } else if (x == 0 || x == self.config.width - 1) || (y == 0 || y == self.config.height - 1) {
                    placeholder.push('│');
                } else {
                    placeholder.push(' ');
                }
            }
            placeholder.push('\n');
        }

        placeholder
    }

    /// Check if the position should contain part of a music note pattern
    fn is_music_note_pattern(&self, x: u32, y: u32, center_x: u32, center_y: u32) -> bool {
        let dx = (x as i32 - center_x as i32).abs() as u32;
        let dy = (y as i32 - center_y as i32).abs() as u32;

        // Simple pattern: musical note in the center
        (dx <= 2 && dy <= 1) || (x == center_x && y < center_y && dy <= 3)
    }

    /// Render album art for a song file (main entry point) with caching
    pub fn render_album_art_from_file<P: AsRef<Path>>(&self, file_path: P) -> Result<String> {
        if !self.config.show_art {
            return Ok(String::new());
        }

        // Performance optimization: Check cache first
        let cache_key = AsciiArtCacheKey {
            file_path: file_path.as_ref().to_string_lossy().to_string(),
            width: self.config.width,
            height: self.config.height,
            use_color: self.config.use_color,
        };

        // Try to get from cache
        if let Ok(cache) = self.ascii_cache.lock() {
            if let Some(cached_art) = cache.get(&cache_key) {
                return Ok(cached_art.clone());
            }
        }

        // Generate new ASCII art if not in cache
        let ascii_art = match self.extract_album_art(&file_path)? {
            Some(image_data) => {
                // Use the new render_album_art method that handles display modes
                self.render_album_art(&image_data)?
            }
            None => self.generate_placeholder(),
        };

        // Store in cache for future use
        if let Ok(mut cache) = self.ascii_cache.lock() {
            cache.insert(cache_key, ascii_art.clone());
        }

        Ok(ascii_art)
    }



    /// Update dimensions for dynamic scaling based on available area
    pub fn update_dimensions(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
    }

    /// Calculate optimal dimensions using exact CSS formulas with actual image data
    /// Implements: width: 80% and height: auto using intrinsic image dimensions
    pub fn calculate_optimal_dimensions(image_data: &[u8], panel_width: u16, panel_height: u16) -> std::result::Result<(u32, u32), Box<dyn Error>> {
        // Parent Container Width = panel width minus borders
        let parent_container_width = panel_width.saturating_sub(2) as f32;
        let usable_height = panel_height.saturating_sub(2) as f32;

        // CSS Formula: Image Width = 0.8 × Parent Container Width
        let image_width = (0.8 * parent_container_width) as u32;

        // Ensure minimum width for readability
        let min_width = 16;
        let final_image_width = image_width.max(min_width);

        // Get intrinsic dimensions from image data
        let img = image::load_from_memory(image_data).map_err(|e| Box::new(e) as Box<dyn Error>)?;
        let (intrinsic_width, intrinsic_height) = img.dimensions();
        let aspect_ratio = intrinsic_height as f32 / intrinsic_width as f32;

        // CSS Formula: Image Height = Image Width × (Intrinsic Height / Intrinsic Width)
        let calculated_height = final_image_width as f32 * aspect_ratio;

        // Terminal character compensation (chars are ~2x taller than wide)
        let terminal_compensated_height = calculated_height * 1.0;

        // Apply reasonable bounds
        let min_height = 8;
        let max_height = usable_height as u32;

        let final_height = (terminal_compensated_height as u32)
            .max(min_height)
            .min(max_height);

        Ok((final_image_width, final_height))
    }

    /// Render album art with dynamic dimensions for a specific panel size
    pub fn render_album_art_for_panel(&mut self, image_data: &[u8], panel_width: u16, panel_height: u16) -> std::result::Result<String, Box<dyn Error>> {
        if !self.config.show_art {
            return Ok(String::new());
        }

        // Calculate optimal dimensions based on actual image aspect ratio
        let (optimal_width, optimal_height) = Self::calculate_optimal_dimensions(image_data, panel_width, panel_height)?;
        self.update_dimensions(optimal_width, optimal_height);

        // Render with updated dimensions
        self.render_album_art(image_data).map_err(|e| Box::new(e) as Box<dyn Error>)
    }


    /// Calculate optimal dimensions for placeholders using CSS formulas (assumes square aspect ratio)
    /// Implements: width: 80% and height: auto for placeholders
    pub fn calculate_placeholder_dimensions(panel_width: u16, panel_height: u16) -> (u32, u32) {
        // Parent Container Width = panel width minus borders
        let parent_container_width = panel_width.saturating_sub(2) as f32;
        let usable_height = panel_height.saturating_sub(2) as f32;

        // CSS Formula: Image Width = 0.8 × Parent Container Width
        let image_width = (0.8 * parent_container_width) as u32;

        // Ensure minimum width for readability
        let min_width = 16;
        let final_image_width = image_width.max(min_width);

        // CSS Formula: Image Height = Image Width × (Intrinsic Height / Intrinsic Width)
        // For placeholder: assume square image (1:1 aspect ratio)
        // Image Height = Image Width × (1 / 1) = Image Width
        let calculated_height = final_image_width as f32;

        // Terminal character compensation (chars are ~2x taller than wide)
        let terminal_compensated_height = calculated_height * 2.0;

        // Apply reasonable bounds
        let min_height = 8;
        let max_height = usable_height as u32;

        let final_height = (terminal_compensated_height as u32)
            .max(min_height)
            .min(max_height);

        (final_image_width, final_height)
    }

    /// Generate a placeholder with dynamic dimensions for a specific panel size
    pub fn generate_placeholder_for_panel(&mut self, panel_width: u16, panel_height: u16) -> String {
        if !self.config.show_art {
            return String::new();
        }

        // Calculate and update optimal dimensions for this panel size
        let (optimal_width, optimal_height) = Self::calculate_placeholder_dimensions(panel_width, panel_height);
        self.update_dimensions(optimal_width, optimal_height);
        
        // Generate placeholder with updated dimensions
        self.generate_placeholder()
    }
}

impl Default for AlbumArtRenderer {
    fn default() -> Self {
        Self::new(AlbumArtConfig::default())
    }
}

/// Utility functions for album art processing
pub mod utils {
}
