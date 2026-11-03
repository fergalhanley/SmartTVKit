//! Small, testable subset of the A2UI v0.9.1 message interpreter.
//! This is a renderer feasibility spike, not a full A2UI implementation.

use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

pub mod source;

pub const BUNDLED_A2UI: &str = include_str!("../app/home.a2ui.jsonl");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError(pub String);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for ConfigError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rail {
    pub title: String,
    pub cards: Vec<Card>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scene {
    pub headline: String,
    pub subtitle: String,
    pub rails: Vec<Rail>,
}

impl Scene {
    /// Decode a stream of unmodified A2UI v0.9.1 JSON message envelopes.
    /// Only Column, Row, Card and Text from the Basic Catalog are rendered
    /// by this prototype. Unknown components fail rather than disappear.
    pub fn from_a2ui_jsonl(source: &str) -> Result<Self, ConfigError> {
        let mut surface: Option<String> = None;
        let mut components: HashMap<String, Value> = HashMap::new();
        let mut data = Value::Null;

        for (line_index, line) in source.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let msg: Value = serde_json::from_str(line).map_err(|error| {
                ConfigError(format!("A2UI line {}: {error}", line_index + 1))
            })?;
            if msg.get("version").and_then(Value::as_str) != Some("v0.9.1") {
                return Err(ConfigError(format!(
                    "A2UI line {}: only v0.9.1 is implemented",
                    line_index + 1
                )));
            }

            if let Some(create) = msg.get("createSurface") {
                if surface.is_some() {
                    return Err(ConfigError("duplicate createSurface".into()));
                }
                let id = get_string(create, "surfaceId")?;
                let _catalog = get_string(create, "catalogId")?;
                surface = Some(id.to_owned());
            } else if let Some(update) = msg.get("updateComponents") {
                check_surface(update, &surface)?;
                let entries = update
                    .get("components")
                    .and_then(Value::as_array)
                    .ok_or_else(|| ConfigError("updateComponents.components missing".into()))?;
                for entry in entries {
                    let id = get_string(entry, "id")?;
                    let kind = get_string(entry, "component")?;
                    if !matches!(kind, "Text" | "Row" | "Column" | "Card") {
                        return Err(ConfigError(format!(
                            "component {id}: unsupported Basic Catalog component {kind}"
                        )));
                    }
                    components.insert(id.to_owned(), entry.clone());
                }
            } else if let Some(update) = msg.get("updateDataModel") {
                check_surface(update, &surface)?;
                let path = update.get("path").and_then(Value::as_str).unwrap_or("/");
                let value = update.get("value").cloned().unwrap_or(Value::Null);
                if path == "/" || path.is_empty() {
                    data = value;
                } else {
                    let destination = data.pointer_mut(path).ok_or_else(|| {
                        ConfigError(format!("data model path {path} does not exist"))
                    })?;
                    *destination = value;
                }
            } else if let Some(delete) = msg.get("deleteSurface") {
                check_surface(delete, &surface)?;
                components.clear();
                data = Value::Null;
                surface = None;
            } else {
                return Err(ConfigError(format!(
                    "A2UI line {}: unknown message",
                    line_index + 1
                )));
            }
        }

        if surface.is_none() {
            return Err(ConfigError("no active A2UI surface".into()));
        }

        let root = components
            .get("root")
            .ok_or_else(|| ConfigError("missing root component".into()))?;
        if root.get("component").and_then(Value::as_str) != Some("Column") {
            return Err(ConfigError("prototype root must be a Column".into()));
        }
        let child_ids = get_children(root)?;
        let mut headings = Vec::new();
        let mut pending_label: Option<String> = None;
        let mut rails = Vec::new();

        for child_id in child_ids {
            let child = find_component(&components, child_id)?;
            match get_string(child, "component")? {
                "Text" => {
                    let label = resolve_text(child, &data)?;
                    if headings.len() < 2 {
                        headings.push(label);
                    } else {
                        pending_label = Some(label);
                    }
                }
                "Row" => {
                    let mut cards = Vec::new();
                    for id in get_children(child)? {
                        let card = find_component(&components, id)?;
                        if get_string(card, "component")? != "Card" {
                            return Err(ConfigError(format!(
                                "{id} must be Card inside Row"
                            )));
                        }
                        let text_id = get_string(card, "child")?;
                        let title_node = find_component(&components, text_id)?;
                        if get_string(title_node, "component")? != "Text" {
                            return Err(ConfigError(format!(
                                "{text_id} must be a Text inside Card"
                            )));
                        }
                        cards.push(Card {
                            id: id.to_owned(),
                            title: resolve_text(title_node, &data)?,
                        });
                    }
                    if cards.is_empty() {
                        return Err(ConfigError("an A2UI Row contains no cards".into()));
                    }
                    rails.push(Rail {
                        title: pending_label.take().unwrap_or_else(|| "Collection".into()),
                        cards,
                    });
                }
                other => {
                    return Err(ConfigError(format!(
                        "unexpected root child component {other}"
                    )));
                }
            }
        }

        if headings.len() < 2 || rails.is_empty() {
            return Err(ConfigError(
                "expected headline, subtitle and at least one Row".into(),
            ));
        }
        Ok(Self {
            headline: headings.remove(0),
            subtitle: headings.remove(0),
            rails,
        })
    }
}

fn get_string<'a>(obj: &'a Value, field: &str) -> Result<&'a str, ConfigError> {
    obj.get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| ConfigError(format!("missing or invalid field {field}")))
}

fn get_children(value: &Value) -> Result<Vec<&str>, ConfigError> {
    value
        .get("children")
        .and_then(Value::as_array)
        .ok_or_else(|| ConfigError("component children must be an array".into()))?
        .iter()
        .map(|child| {
            child.as_str().ok_or_else(|| {
                ConfigError("component child reference must be a string".into())
            })
        })
        .collect()
}

fn find_component<'a>(
    nodes: &'a HashMap<String, Value>,
    id: &str,
) -> Result<&'a Value, ConfigError> {
    nodes
        .get(id)
        .ok_or_else(|| ConfigError(format!("component reference {id} not found")))
}

fn check_surface(msg: &Value, surface: &Option<String>) -> Result<(), ConfigError> {
    let current = surface
        .as_deref()
        .ok_or_else(|| ConfigError("createSurface must come first".into()))?;
    if get_string(msg, "surfaceId")? != current {
        return Err(ConfigError("A2UI surfaceId mismatch".into()));
    }
    Ok(())
}

fn resolve_text(text: &Value, data: &Value) -> Result<String, ConfigError> {
    let value = text
        .get("text")
        .ok_or_else(|| ConfigError("Text component has no text property".into()))?;
    if let Some(literal) = value.as_str() {
        return Ok(literal.to_owned());
    }
    let path = get_string(value, "path")?;
    let resolved = data
        .pointer(path)
        .and_then(Value::as_str)
        .ok_or_else(|| ConfigError(format!("Text binding {path} is missing or not a string")))?;
    Ok(resolved.to_owned())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Navigation {
    pub row: usize,
    pub column: usize,
    pub details_open: bool,
}

impl Navigation {
    pub fn selected<'a>(&self, scene: &'a Scene) -> Option<&'a Card> {
        scene
            .rails
            .get(self.row)
            .and_then(|rail| rail.cards.get(self.column))
    }

    pub fn move_direction(&mut self, direction: Direction, scene: &Scene) {
        if self.details_open || scene.rails.is_empty() {
            return;
        }
        match direction {
            Direction::Left => self.column = self.column.saturating_sub(1),
            Direction::Right => {
                let max_index = scene.rails[self.row].cards.len().saturating_sub(1);
                self.column = (self.column + 1).min(max_index);
            }
            Direction::Up => self.row = self.row.saturating_sub(1),
            Direction::Down => self.row = (self.row + 1).min(scene.rails.len() - 1),
        }
        let max_index = scene.rails[self.row].cards.len().saturating_sub(1);
        self.column = self.column.min(max_index);
    }

    pub fn enter(&mut self, scene: &Scene) {
        self.details_open = self.selected(scene).is_some();
    }

    pub fn back(&mut self) {
        self.details_open = false;
    }

    pub fn reset_if_out_of_bounds(&mut self, scene: &Scene) {
        self.row = self.row.min(scene.rails.len().saturating_sub(1));
        let size = scene
            .rails
            .get(self.row)
            .map(|rail| rail.cards.len())
            .unwrap_or(0);
        self.column = self.column.min(size.saturating_sub(1));
        self.details_open = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_stream_parses() {
        let scene = Scene::from_a2ui_jsonl(BUNDLED_A2UI).unwrap();
        assert_eq!(scene.headline, "Television, reimagined.");
        assert_eq!(scene.rails.len(), 2);
        assert_eq!(scene.rails[0].title, "Featured experiences");
        assert_eq!(scene.rails[0].cards[0].title, "After the Horizon");
        assert_eq!(scene.rails[1].cards.len(), 5);
    }

    #[test]
    fn changing_only_a2ui_data_updates_ui_text() {
        let edited = BUNDLED_A2UI.replace("After the Horizon", "After the Sunrise");
        let scene = Scene::from_a2ui_jsonl(&edited).unwrap();
        assert_eq!(scene.rails[0].cards[0].title, "After the Sunrise");
    }

    #[test]
    fn no_surface_is_rejected() {
        let bad = r#"{"version":"v0.9.1","updateComponents":{"surfaceId":"home","components":[]}}"#;
        assert!(Scene::from_a2ui_jsonl(bad).is_err());
    }

    #[test]
    fn unknown_component_is_rejected() {
        let edited = BUNDLED_A2UI.replace("\"component\":\"Card\"", "\"component\":\"MagicCard\"");
        assert!(Scene::from_a2ui_jsonl(&edited).is_err());
    }

    #[test]
    fn directional_focus_and_details() {
        let scene = Scene::from_a2ui_jsonl(BUNDLED_A2UI).unwrap();
        let mut nav = Navigation::default();
        nav.move_direction(Direction::Right, &scene);
        assert_eq!(nav.selected(&scene).unwrap().title, "The Quiet Signal");
        nav.move_direction(Direction::Down, &scene);
        assert_eq!(nav.selected(&scene).unwrap().title, "Nature Live");
        nav.enter(&scene);
        assert!(nav.details_open);
        nav.move_direction(Direction::Left, &scene);
        assert_eq!(nav.column, 1);
        nav.back();
        nav.move_direction(Direction::Left, &scene);
        assert_eq!(nav.column, 0);
        nav.move_direction(Direction::Left, &scene);
        assert_eq!(nav.column, 0);
    }
}
