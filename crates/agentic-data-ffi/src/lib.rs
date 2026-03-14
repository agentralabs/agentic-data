//! AgenticData FFI — C-compatible bindings for cross-language integration.
//!
//! Functions follow the pattern: adat_<module>_<operation>
//! All functions return 0 on success, negative on error.
//! String outputs are written to caller-provided buffers.

use std::ffi::{c_char, c_int, CStr, CString};
use std::ptr;

/// Returns the library version. Caller must not free the returned pointer.
#[no_mangle]
pub extern "C" fn adat_version() -> *const c_char {
    c"0.1.0".as_ptr()
}

/// Detect format of data. Writes format name to out_buf.
/// Returns format confidence as integer percentage (0-100), or -1 on error.
#[no_mangle]
pub unsafe extern "C" fn adat_detect_format(
    data: *const c_char,
    data_len: usize,
    out_buf: *mut c_char,
    out_buf_len: usize,
) -> c_int {
    if data.is_null() || out_buf.is_null() { return -1; }
    let data_slice = std::slice::from_raw_parts(data as *const u8, data_len);
    let data_str = match std::str::from_utf8(data_slice) {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let detection = agentic_data::parser::detect::detect_format(data_str, None);
    let name = detection.format.name();

    if let Ok(c_name) = CString::new(name) {
        let bytes = c_name.as_bytes_with_nul();
        if bytes.len() <= out_buf_len {
            ptr::copy_nonoverlapping(bytes.as_ptr(), out_buf as *mut u8, bytes.len());
        }
    }

    (detection.confidence * 100.0) as c_int
}

/// Count supported formats.
#[no_mangle]
pub extern "C" fn adat_format_count() -> c_int {
    agentic_data::parser::supported_formats().len() as c_int
}

/// Compute BLAKE3 hash of data. Writes hex hash to out_buf.
/// Returns 0 on success, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn adat_hash(
    data: *const u8,
    data_len: usize,
    out_buf: *mut c_char,
    out_buf_len: usize,
) -> c_int {
    if data.is_null() || out_buf.is_null() || out_buf_len < 65 { return -1; }
    let slice = std::slice::from_raw_parts(data, data_len);
    let hash = blake3::hash(slice);
    let hex = hash.to_hex();
    let hex_bytes = hex.as_bytes();
    ptr::copy_nonoverlapping(hex_bytes.as_ptr(), out_buf as *mut u8, 64);
    *out_buf.add(64) = 0; // null terminator
    0
}

/// Check if a string value contains PII. Returns 1 if PII detected, 0 if clean, -1 on error.
#[no_mangle]
pub unsafe extern "C" fn adat_has_pii(
    field_name: *const c_char,
    value: *const c_char,
) -> c_int {
    if field_name.is_null() || value.is_null() { return -1; }
    let field = match CStr::from_ptr(field_name).to_str() {
        Ok(s) => s, Err(_) => return -1,
    };
    let val = match CStr::from_ptr(value).to_str() {
        Ok(s) => s, Err(_) => return -1,
    };

    let mut fields = std::collections::HashMap::new();
    fields.insert(field.to_string(), serde_json::json!(val));
    let record = agentic_data::DataRecord::new("ffi", "ffi", fields);
    let detections = agentic_data::RedactionEngine::detect(&record);
    if detections.is_empty() { 0 } else { 1 }
}

/// Calculate haversine distance in meters between two points.
#[no_mangle]
pub extern "C" fn adat_geo_distance(
    lat1: f64, lng1: f64,
    lat2: f64, lng2: f64,
) -> f64 {
    let a = agentic_data::GeoPoint::new(lat1, lng1);
    let b = agentic_data::GeoPoint::new(lat2, lng2);
    a.distance_meters(&b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let v = adat_version();
        assert!(!v.is_null());
        let s = unsafe { CStr::from_ptr(v) }.to_str().unwrap();
        assert_eq!(s, "0.1.0");
    }

    #[test]
    fn test_format_count() {
        assert!(adat_format_count() >= 16);
    }

    #[test]
    fn test_geo_distance() {
        let d = adat_geo_distance(40.7128, -74.0060, 34.0522, -118.2437);
        assert!(d > 3_900_000.0 && d < 4_000_000.0);
    }
}
