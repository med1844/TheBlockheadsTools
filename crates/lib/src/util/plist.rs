use crate::BhResult;

/// Recursively compare two plist::Value trees, collecting key-set
/// differences. Reports contaminated keys (in serialized but not original)
/// and lost keys (in original but not serialized).
pub fn diff_plist_keys(
    path: &str,
    original: &plist::Value,
    serialized: &plist::Value,
    diffs: &mut Vec<String>,
) {
    use plist::Value;
    match (original, serialized) {
        (Value::Dictionary(orig), Value::Dictionary(ser)) => {
            for key in ser.keys() {
                if !orig.contains_key(key) {
                    diffs.push(format!("  CONTAMINATION: {path}/{key} (not in original)"));
                }
            }
            for key in orig.keys() {
                if !ser.contains_key(key) {
                    diffs.push(format!("  DATA LOSS: {path}/{key} (not in serialized)"));
                }
            }
            // Recurse into shared keys
            for key in orig.keys() {
                if let (Some(ov), Some(sv)) = (orig.get(key), ser.get(key)) {
                    diff_plist_keys(&format!("{path}/{key}"), ov, sv, diffs);
                }
            }
        }
        (Value::Array(orig), Value::Array(ser)) => {
            if orig.len() != ser.len() {
                diffs.push(format!(
                    "  ARRAY LEN: {path} (original: {}, serialized: {})",
                    orig.len(),
                    ser.len()
                ));
            }
            for (i, (ov, sv)) in orig.iter().zip(ser.iter()).enumerate() {
                diff_plist_keys(&format!("{path}[{i}]"), ov, sv, diffs);
            }
        }
        _ => {} // Leaf values — struct equality already covers correctness
    }
}

pub fn to_xml_plist<T: serde::ser::Serialize>(value: &T) -> BhResult<Vec<u8>> {
    let mut serialized = Vec::new();
    plist::to_writer_xml(&mut serialized, value)?;
    Ok(serialized)
}
