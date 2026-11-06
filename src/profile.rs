//! User Custom Lighting Profiles and JSON Persistence.

use std::fs;
use std::io;
use std::path::PathBuf;

use crate::color::Rgb;
use crate::matrix::TOTAL_ZONES;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomProfile {
    pub name: String,
    pub description: String,
    pub zones: [Rgb; TOTAL_ZONES],
}

impl CustomProfile {
    pub fn new(name: impl Into<String>, description: impl Into<String>, zones: [Rgb; TOTAL_ZONES]) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            zones,
        }
    }
}

/// Returns default configuration directory path: `~/.config/msi-mysticlight`
pub fn get_config_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config").join("msi-mysticlight"))
}

/// Returns default custom profiles file path: `~/.config/msi-mysticlight/profiles.json`
pub fn get_profiles_path() -> Option<PathBuf> {
    get_config_dir().map(|d| d.join("profiles.json"))
}

/// Serialize a list of custom profiles into clean JSON text.
pub fn serialize_profiles(profiles: &[CustomProfile]) -> String {
    let mut out = String::from("[\n");
    for (i, p) in profiles.iter().enumerate() {
        out.push_str("  {\n");
        out.push_str(&format!("    \"name\": \"{}\",\n", escape_json(&p.name)));
        out.push_str(&format!("    \"description\": \"{}\",\n", escape_json(&p.description)));
        out.push_str("    \"zones\": [\n");

        for (z_idx, z) in p.zones.iter().enumerate() {
            let comma = if z_idx + 1 < TOTAL_ZONES { "," } else { "" };
            let prefix = if z_idx % 4 == 0 { "      " } else { " " };
            let newline = if z_idx % 4 == 3 || z_idx + 1 == TOTAL_ZONES { "\n" } else { "" };
            out.push_str(&format!("{}\"{}\"{}{}", prefix, z, comma, newline));
        }

        out.push_str("    ]\n");
        let item_comma = if i + 1 < profiles.len() { "," } else { "" };
        out.push_str(&format!("  }}{}\n", item_comma));
    }
    out.push_str("]\n");
    out
}

/// Deserializes a list of custom profiles from JSON text (zero-dependency parser).
pub fn deserialize_profiles(json: &str) -> Vec<CustomProfile> {
    let mut profiles = Vec::new();
    let trimmed = json.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return profiles;
    }

    // Split into top-level JSON objects by looking for `{ ... }` blocks
    let mut inside_obj = false;
    let mut current_block = String::new();

    for ch in trimmed.chars() {
        if ch == '{' {
            inside_obj = true;
            current_block.clear();
        } else if ch == '}' && inside_obj {
            inside_obj = false;
            if let Some(profile) = parse_single_profile(&current_block) {
                profiles.push(profile);
            }
            current_block.clear();
        } else if inside_obj {
            current_block.push(ch);
        }
    }

    profiles
}

fn parse_single_profile(block: &str) -> Option<CustomProfile> {
    let name = extract_string_field(block, "\"name\"")?;
    let mut description = String::new();
    let mut zones = [Rgb::new(255, 255, 255); TOTAL_ZONES];

    // Extract description
    if let Some(d) = extract_string_field(block, "\"description\"") {
        description = d;
    }

    // Extract zones array
    if let Some(zones_part) = block.split("\"zones\"").nth(1) {
        if let Some(start_bracket) = zones_part.find('[') {
            if let Some(end_bracket) = zones_part.find(']') {
                let inner = &zones_part[start_bracket + 1..end_bracket];
                let mut idx = 0;
                for token in inner.split(',') {
                    if idx >= TOTAL_ZONES {
                        break;
                    }
                    let cleaned = token.replace(['\"', '\n', '\r'], "").trim().to_string();
                    if !cleaned.is_empty() {
                        if let Ok(rgb) = Rgb::from_hex(&cleaned) {
                            zones[idx] = rgb;
                            idx += 1;
                        }
                    }
                }
            }
        }
    }

    Some(CustomProfile::new(name, description, zones))
}

fn extract_string_field(block: &str, field_key: &str) -> Option<String> {
    let part = block.split(field_key).nth(1)?;
    let colon_part = part.split(':').nth(1)?;
    let first_quote = colon_part.find('\"')?;
    let rest = &colon_part[first_quote + 1..];
    let end_quote = rest.find('\"')?;
    Some(rest[..end_quote].to_string())
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\"', "\\\"")
}

/// Load custom profiles from disk, returning empty list if file doesn't exist.
pub fn load_custom_profiles() -> Vec<CustomProfile> {
    let Some(path) = get_profiles_path() else {
        return Vec::new();
    };
    if !path.exists() {
        return Vec::new();
    }
    match fs::read_to_string(&path) {
        Ok(content) => deserialize_profiles(&content),
        Err(_) => Vec::new(),
    }
}

/// Save custom profiles list to disk.
pub fn save_custom_profiles(profiles: &[CustomProfile]) -> io::Result<()> {
    let Some(path) = get_profiles_path() else {
        return Err(io::Error::new(io::ErrorKind::NotFound, "No home directory found"));
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serialize_profiles(profiles);
    fs::write(path, content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_and_deserialize_profiles() {
        let mut zones = [Rgb::new(0, 0, 0); TOTAL_ZONES];
        zones[0] = Rgb::new(255, 0, 128);
        zones[23] = Rgb::new(0, 255, 200);

        let original = vec![
            CustomProfile::new("Night Ride", "Cool midnight drive", zones),
            CustomProfile::new("Cyber Matrix", "Hacker green terminal", [Rgb::new(0, 255, 65); TOTAL_ZONES]),
        ];

        let json = serialize_profiles(&original);
        let parsed = deserialize_profiles(&json);

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].name, "Night Ride");
        assert_eq!(parsed[0].description, "Cool midnight drive");
        assert_eq!(parsed[0].zones[0], Rgb::new(255, 0, 128));
        assert_eq!(parsed[0].zones[23], Rgb::new(0, 255, 200));

        assert_eq!(parsed[1].name, "Cyber Matrix");
        assert_eq!(parsed[1].zones[10], Rgb::new(0, 255, 65));
    }
}
