use super::Catalog;
use crate::Error;
use serde_json::{Map, Value};

pub(super) fn ast_document(text: &str) -> Result<jsonc_parser::ast::Value<'_>, Error> {
    let options = jsonc_parser::ParseOptions {
        allow_comments: true,
        allow_trailing_commas: true,
        allow_loose_object_property_names: false,
        allow_missing_commas: false,
        allow_single_quoted_strings: false,
        allow_hexadecimal_numbers: false,
        allow_unary_plus_numbers: false,
        allow_bare_decimal_point_numbers: false,
        allow_non_finite_numbers: false,
        allow_extended_string_escapes: false,
    };
    let parsed = jsonc_parser::parse_to_ast(text, &Default::default(), &options)
        .map_err(|_| Error::Invalid("invalid Terminal settings JSONC"))?;
    let value = parsed
        .value
        .ok_or(Error::Invalid("empty Terminal settings"))?;
    unique_keys(&value)?;
    Ok(value)
}
pub(super) fn document(text: &str) -> Result<Map<String, Value>, Error> {
    let text = text.trim_start_matches('\u{feff}');
    ast_document(text)?;
    let value: Value = jsonc_parser::parse_to_serde_value(text, &Default::default())
        .map_err(|_| Error::Invalid("invalid Terminal settings JSONC"))?;
    value
        .as_object()
        .cloned()
        .ok_or(Error::Invalid("settings must be an object"))
}
fn unique_keys(value: &jsonc_parser::ast::Value<'_>) -> Result<(), Error> {
    use jsonc_parser::ast;
    match value {
        ast::Value::Object(object) => {
            let mut names = std::collections::HashSet::new();
            for property in &object.properties {
                if !names.insert(property.name.as_str()) {
                    return Err(Error::Invalid("duplicate settings key"));
                }
                unique_keys(&property.value)?;
            }
        }
        ast::Value::Array(array) => {
            for value in &array.elements {
                unique_keys(value)?;
            }
        }
        _ => (),
    }
    Ok(())
}
pub(super) fn profiles(
    root: &Map<String, Value>,
) -> Result<(Map<String, Value>, Vec<Value>), Error> {
    let Some(container) = root.get("profiles") else {
        return Ok((Map::new(), Vec::new()));
    };
    let mut defaults = match container.get("defaults") {
        Some(value) => value
            .as_object()
            .cloned()
            .ok_or(Error::Invalid("invalid profile defaults"))?,
        None => Map::new(),
    };
    for key in ["guid", "name", "source", "commandline"] {
        defaults.remove(key);
    }
    let list = container
        .as_array()
        .or_else(|| container.get("list").and_then(Value::as_array));
    if !container.is_object() && !container.is_array() {
        return Err(Error::Invalid("invalid profiles container"));
    }
    Ok((defaults, list.cloned().unwrap_or_default()))
}
impl Catalog {
    /// Parse without writes or generated providers. Use `from_sources` for installed settings.
    pub fn parse(text: &str) -> Result<Self, Error> {
        Self::from_sources(text, None, &[], &[])
    }
    pub fn default(&self) -> Result<&super::Profile, Error> {
        let guid = self.default_profile.as_ref().ok_or(Error::MissingProfile)?;
        self.profiles
            .iter()
            .find(|profile| &profile.guid == guid)
            .ok_or(Error::MissingProfile)
    }
}
