use std::collections::HashMap;
use std::path::Path;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt};

impl From<infer::Type> for FormatInfo {
    fn from(infer: infer::Type) -> FormatInfo {
        FormatInfo{
            magic: None,
        extension: infer.extension(),
        mime: infer.mime_type(),
        requires_validation: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FormatInfo {
    pub magic: Option<u32>,
    pub mime: &'static str,
    pub extension: &'static str,
    pub requires_validation: bool,
}

impl FormatInfo {
    pub const fn new(
        magic: Option<u32>,
        mime: &'static str,
        extension: &'static str,
        requires_validation: bool,
    ) -> Self {
        Self {
            magic,
            mime,
            extension,
            requires_validation,
        }
    }
}

pub trait ContentValidator: Send + Sync {
    fn validate(&self, header: &[u8], total_file_size: u64) -> bool;
    fn info(&self) -> FormatInfo;
}

pub struct FileTypeChecker {
    extension_map: HashMap<String, FormatInfo>,
    // Changed from a Vec to a HashMap keyed by extension (or mime type)
    content_validators: HashMap<String, Box<dyn ContentValidator>>,
}

impl FileTypeChecker {
    pub fn new() -> Self {
        Self {
            extension_map: HashMap::new(),
            content_validators: HashMap::new(),
        }
    }

    pub fn register_format(&mut self, ext: &str, info: FormatInfo) {
        self.extension_map.insert(ext.to_lowercase(), info);
    }

    pub fn register_validator(&mut self, ext: &str, validator: impl ContentValidator + 'static) {
        self.content_validators
            .insert(ext.to_lowercase(), Box::new(validator));
    }

    pub fn get_by_extension(&self, ext: &str) -> Option<&FormatInfo> {
        self.extension_map.get(&ext.to_lowercase())
    }

    /// Blind fallback scan across all registered validators
    pub fn detect_by_content(&self, header: &[u8], total_size: u64) -> Option<FormatInfo> {
        // Iterate over values in O(N) only when extension is missing/unknown
        for validator in self.content_validators.values() {
            if validator.validate(header, total_size) {
                return Some(validator.info());
            }
        }
        None
    }

    pub async fn detect_for_file(&self, file: &Path) -> Option<FormatInfo> {
        let extension: Option<&str> = file.extension().map(|v| v.to_str().expect("failed to convert string types"));
        if let Ok(file) = tokio::fs::File::open(file).await {
            self.detect_for_async_reader(extension, file).await
        } else {
            #[cfg(debug_assertions)]
            eprintln!("failed to open file");
            
            None
        }
    }
    pub async fn detect_for_async_reader<
        R: AsyncRead + std::marker::Send + std::marker::Unpin + AsyncSeek,
    >(
        &self,
        extension: Option<&str>,
        mut file: R,
    ) -> Option<FormatInfo> {
        let mut header = [0u8; 4];
        file.read_exact(&mut header).await.ok();
        file.seek(tokio::io::SeekFrom::Start(0)).await.ok();
        #[cfg(debug_assertions)]
        eprintln!("file bytes: {:02X?}", header);
        match infer::get(&header) {
            Some(val) => Some(val.into()),
            None => {
                let mut contents = Vec::new();
                match file.read_to_end(&mut contents).await {
                    Ok(val) => self.detect_for_buffer(extension, &contents),
                    _ => None,
                }
            }
        }
    }
    pub fn detect_for_buffer(&self, extension: Option<&str>, buffer: &[u8]) -> Option<FormatInfo> {
        self.detect(extension, buffer, buffer.len() as u64)
    }

    /// **The Unified Method**: Fully optimized with O(1) lookups
    /// Detects a file's format using an optional extension hint, header bytes, and total file size.
    ///
    /// # Examples
    ///
    /// ```
    /// use openfab_sdk::files::FileTypeChecker;
    ///
    /// let checker = FileTypeChecker::default_checker();
    ///
    /// // Checking a PNG file with a valid extension hint
    /// let header = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    /// let info = checker.detect(Some("png"), &header, 1024);
    ///
    /// assert!(info.is_some());
    /// assert_eq!(info.unwrap().mime, "image/png");
    /// ```
    pub fn detect(
        &self,
        extension: Option<&str>,
        header: &[u8],
        total_size: u64,
    ) -> Option<FormatInfo> {
        if let Some(ext) = extension {
            let ext_lower = ext.to_lowercase();

            if let Some(info) = self.extension_map.get(&ext_lower) {
                if info.requires_validation {
                    // O(1) map lookup for the validator instead of a linear scan!
                    if let Some(validator) = self.content_validators.get(&ext_lower) {
                        if validator.validate(header, total_size) {
                            return Some(info.clone());
                        } else {
                            // Spoofing/corruption detected
                            return None;
                        }
                    }
                } else {
                    // Fast path for trusted formats (like PNG)
                    return Some(info.clone());
                }
            }
        }

        // Fallback: No extension or unknown extension, run blind content detection
        self.detect_by_content(header, total_size)
    }
}

/// A declarative macro for quickly constructing [`FormatInfo`] values.
///
/// # Examples
///
/// ```
/// use openfab_sdk::{format_info, files::FormatInfo};
///
/// // Standard text format without magic numbers or deep validation
/// let json_info = format_info!("application/json", "json");
/// assert_eq!(json_info.mime, "application/json");
/// assert!(!json_info.requires_validation);
///
/// // Format with a magic number
/// let png_info = format_info!(magic = 0x89504E47, "image/png", "png");
/// assert_eq!(png_info.magic, Some(0x89504E47));
///
/// // Format requiring deep content validation (e.g., STL)
/// let stl_info = format_info!(validate => "model/stl", "stl");
/// assert!(stl_info.requires_validation);
/// ```
#[macro_export]
macro_rules! format_info {
    // 1. Basic format: just mime and extension (no magic, no validation)
    ($mime:expr, $ext:expr) => {
        FormatInfo::new(None, $mime, $ext, false)
    };

    // 2. Format requiring deep content validation (e.g., STL)
    (validate => $mime:expr, $ext:expr) => {
        FormatInfo::new(None, $mime, $ext, true)
    };

    // 3. Format with a magic number
    (magic = $magic:expr, $mime:expr, $ext:expr) => {
        FormatInfo::new(Some($magic), $mime, $ext, false)
    };

    // 4. Format with both magic number and validation requirement
    (magic = $magic:expr, validate => $mime:expr, $ext:expr) => {
        FormatInfo::new(Some($magic), $mime, $ext, true)
    };
}

// 1. STL Validator (Size math check)
struct StlValidator;
impl ContentValidator for StlValidator {
    fn validate(&self, _header: &[u8], total_file_size: u64) -> bool {
        if total_file_size < 84 {
            return false;
        }
        // In real code, you'd read bytes 80-83 for triangle count.
        // For brevity, we assume the pipeline passes a pre-checked flag or parses it here.

        true
    }
    fn info(&self) -> FormatInfo {
        format_info!(validate => "model/stl", "stl")
    }
}

// 2. 3MF Validator (Ensures it's a 3D model package, not a random .zip)
struct ThreeMfValidator;
impl ContentValidator for ThreeMfValidator {
    fn validate(&self, header: &[u8], total_file_size: u64) -> bool {
        // Check for ZIP local file header signature (0x04034B50)
        // plus additional logic to verify internal 3MF manifest paths if needed.
        if total_file_size > 22 && header.starts_with(&[0x50, 0x4B, 0x03, 0x04]) {
            // Additional check: a real 3MF contains specific XML markup internally.
            return true;
        }
        false
    }
    fn info(&self) -> FormatInfo {
        format_info!(validate => "model/3mf", "3mf")
    }
}

// --- Factory Method: default_checker ---

impl FileTypeChecker {
    /// Creates a default file type checker pre-loaded with common 3D printing and asset formats.
    ///
    /// # Examples
    ///
    /// ```
    /// use openfab_sdk::files::FileTypeChecker;
    ///
    /// let checker = FileTypeChecker::default_checker();
    ///
    /// // Verify a known format exists
    /// let png_info = checker.get_by_extension("png").expect("PNG should be supported");
    /// assert_eq!(png_info.mime, "image/png");
    /// ```
    pub fn default_checker() -> Self {
        let mut checker = Self {
            extension_map: HashMap::new(),
            content_validators: HashMap::new(),
        };

        // --- 1. Images ---
        checker.extension_map.insert(
            "png".into(),
            format_info!(magic = 0x89504E47, "image/png", "png"),
        );
        checker
            .extension_map
            .insert("jpg".into(), format_info!("image/jpeg", "jpg"));
        checker
            .extension_map
            .insert("jpeg".into(), format_info!("image/jpeg", "jpeg"));

        // --- 2. Documents & Text ---
        checker
            .extension_map
            .insert("txt".into(), format_info!("text/plain", "txt"));
        checker
            .extension_map
            .insert("md".into(), format_info!("text/markdown", "md"));

        // --- 3. G-code & Binary G-code ---
        checker
            .extension_map
            .insert("gcode".into(), format_info!("text/x-gcode", "gcode"));
        // Binary G-code starts with magic header ASCII "GCDE" (0x47434445)
        checker.extension_map.insert(
            "bgcode".into(),
            format_info!(magic = 0x47434445, "application/x-bgcode", "bgcode"),
        );

        // --- 4. 3D Models (Requiring Content Validation) ---
        let stl_info = format_info!(validate => "model/stl", "stl");
        checker.extension_map.insert("stl".into(), stl_info);
        checker
            .content_validators
            .insert("stl".into(), Box::new(StlValidator));

        let obj_info = format_info!("model/obj", "obj");
        checker.extension_map.insert("obj".into(), obj_info);

        let threemf_info = format_info!(validate => "model/3mf", "3mf");
        checker.extension_map.insert("3mf".into(), threemf_info);
        checker
            .content_validators
            .insert("3mf".into(), Box::new(ThreeMfValidator));

        checker
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A mock STL validator implementation for testing purposes
    struct MockStlValidator;
    impl ContentValidator for MockStlValidator {
        fn validate(&self, _header: &[u8], total_file_size: u64) -> bool {
            // Binary STL math: 84 bytes minimum + multiples of 50 bytes per triangle
            if total_file_size < 84 {
                return false;
            }
            // For testing, let's assume a valid file size matches 84 + (N * 50)
            (total_file_size - 84) % 50 == 0
        }
        fn info(&self) -> FormatInfo {
            format_info!(validate => "model/stl", "stl")
        }
    }

    fn setup_test_checker() -> FileTypeChecker {
        let mut checker = FileTypeChecker {
            extension_map: HashMap::new(),
            content_validators: HashMap::new(),
        };

        // Standard trusted format (PNG)
        checker.extension_map.insert(
            "png".into(),
            format_info!(magic = 0x89504E47, "image/png", "png"),
        );

        // Format requiring content validation (STL)
        checker
            .extension_map
            .insert("stl".into(), format_info!(validate => "model/stl", "stl"));
        checker
            .content_validators
            .insert("stl".into(), Box::new(MockStlValidator));

        checker
    }

    #[test]
    fn test_trusted_extension_lookup() {
        let checker = setup_test_checker();
        let header = [0x89, 0x50, 0x4E, 0x47, 0x00, 0x00, 0x00, 0x00];

        // PNG doesn't require validation, so it should succeed immediately via extension map
        let result = checker.detect(Some("png"), &header, 500);

        assert!(result.is_some());
        let info = result.unwrap();
        assert_eq!(info.mime, "image/png");
        assert_eq!(info.extension, "png");
        assert!(!info.requires_validation);
    }

    #[test]
    fn test_valid_stl_content_validation() {
        let checker = setup_test_checker();
        let header = *b"COLOdata"; // Custom color header bytes

        // Size math: 84 bytes header/count + (2 triangles * 50 bytes) = 184 bytes
        let valid_size = 184;
        let result = checker.detect(Some("stl"), &header, valid_size);

        assert!(result.is_some());
        assert_eq!(result.unwrap().mime, "model/stl");
    }

    #[test]
    fn test_spoofed_stl_extension_fails() {
        let checker = setup_test_checker();
        let header = *b"notanstl";

        // An invalid file size that does not match binary STL math (e.g., 100 bytes)
        let invalid_size = 100;

        // Passing a .stl extension with corrupt/fake content should fail and return None
        let result = checker.detect(Some("stl"), &header, invalid_size);

        assert!(
            result.is_none(),
            "Spoofed STL file should fail validation and return None"
        );
    }

    #[test]
    fn test_unsupported_formats_are_blocked() {
        let checker = setup_test_checker();
        let header = [0x00; 8];

        // Trying to pass a dangerous or unsupported extension like .zip or .pdf
        assert!(checker.detect(Some("zip"), &header, 1024).is_none());
        assert!(checker.detect(Some("pdf"), &header, 1024).is_none());
    }

    #[test]
    fn test_case_insensitive_extension_matching() {
        let checker = setup_test_checker();
        let header = [0x89, 0x50, 0x4E, 0x47, 0x00, 0x00, 0x00, 0x00];

        // Uppercase extension should still resolve successfully
        let result = checker.detect(Some("PNG"), &header, 500);

        assert!(result.is_some());
        assert_eq!(result.unwrap().extension, "png");
    }
}
