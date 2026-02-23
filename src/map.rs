use macroquad_tiled_clone::{IrObjectShape, Map as TiledMap};
use serde::Deserialize;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static ACTIVE_RUNTIME_MAP: OnceLock<RuntimeMapAdapter> = OnceLock::new();

pub fn install_runtime_map(adapter: RuntimeMapAdapter) -> Result<(), RuntimeMapAdapter> {
    ACTIVE_RUNTIME_MAP.set(adapter)
}

pub fn install_runtime_map_compatible(
    adapter: RuntimeMapAdapter,
) -> Result<(), RuntimeMapInstallError> {
    match ACTIVE_RUNTIME_MAP.set(adapter) {
        Ok(()) => Ok(()),
        Err(adapter) => {
            let existing = ACTIVE_RUNTIME_MAP
                .get()
                .expect("OnceLock set failed but existing runtime map missing");

            if existing.is_compatible_with(&adapter) {
                Ok(())
            } else {
                Err(RuntimeMapInstallError::IncompatibleExisting {
                    existing: existing.summary(),
                    attempted: adapter.summary(),
                })
            }
        }
    }
}

pub fn runtime_map() -> Option<&'static RuntimeMapAdapter> {
    ACTIVE_RUNTIME_MAP.get()
}

pub fn try_map_width() -> Option<usize> {
    runtime_map().map(|m| m.width)
}

pub fn try_map_height() -> Option<usize> {
    runtime_map().map(|m| m.height)
}

pub fn try_map_tile_size() -> Option<f32> {
    runtime_map().map(|m| m.tile_size)
}

pub fn try_world_to_grid(x: f32, y: f32) -> Option<(usize, usize)> {
    runtime_map().map(|m| m.world_to_grid(x, y))
}

pub fn try_grid_to_world(x: usize, y: usize) -> Option<(f32, f32)> {
    runtime_map().map(|m| m.grid_to_world(x, y))
}

pub fn try_is_wall(x: usize, y: usize) -> Option<bool> {
    runtime_map().map(|m| m.is_wall(x, y))
}

pub fn try_blocked_for_agent(x: usize, y: usize) -> Option<bool> {
    runtime_map().map(|m| m.blocked_for_agent(x, y))
}

pub fn map_width() -> usize {
    try_map_width().expect("runtime map width unavailable")
}

pub fn map_height() -> usize {
    try_map_height().expect("runtime map height unavailable")
}

pub fn map_tile_size() -> f32 {
    try_map_tile_size().expect("runtime map tile size unavailable")
}

pub fn world_to_grid(x: f32, y: f32) -> (usize, usize) {
    try_world_to_grid(x, y).expect("runtime world_to_grid unavailable")
}

pub fn grid_to_world(x: usize, y: usize) -> (f32, f32) {
    try_grid_to_world(x, y).expect("runtime grid_to_world unavailable")
}

pub fn is_wall(x: usize, y: usize) -> bool {
    try_is_wall(x, y).expect("runtime is_wall unavailable")
}

pub fn blocked_for_agent(x: usize, y: usize) -> bool {
    try_blocked_for_agent(x, y).expect("runtime blocked_for_agent unavailable")
}

pub fn actor_spawn_from_tiled_map(tiled_map: &TiledMap, actor_type: &str) -> Option<(f32, f32)> {
    let layer = tiled_map
        .object_layers()
        .iter()
        .find(|layer| layer.name == "Actors_Layer" && layer.visible)?;

    let obj = layer.objects.iter().find(|obj| {
        actor_spawn_candidate_matches(
            actor_type,
            &obj.class_name,
            layer.visible,
            obj.visible,
            matches!(obj.shape, IrObjectShape::Point),
        )
    })?;

    Some((obj.x + layer.offset.x, obj.y + layer.offset.y))
}

pub fn end_zone_from_tiled_map(tiled_map: &TiledMap) -> Option<(f32, f32, f32, f32)> {
    let layer = tiled_map
        .object_layers()
        .iter()
        .find(|layer| layer.name == "End area" && layer.visible)?;

    let obj = layer.objects.iter().find(|obj| {
        obj.visible
            && obj.class_name == "EndArea"
            && matches!(obj.shape, IrObjectShape::Rectangle)
            && obj.width > 0.0
            && obj.height > 0.0
    })?;

    Some((
        obj.x + layer.offset.x,
        obj.y + layer.offset.y,
        obj.width,
        obj.height,
    ))
}

pub fn actor_spawn_from_tiled_json(
    path: impl AsRef<Path>,
    actor_type: &str,
) -> Result<Option<(f32, f32)>, RuntimeMapAdapterError> {
    let path = path.as_ref();
    let map = read_tiled_json_map(path)?;

    let layer = match map.layers.iter().find(|layer| {
        layer.kind == "objectgroup" && layer.name == "Actors_Layer" && layer.visible.unwrap_or(true)
    }) {
        Some(layer) => layer,
        None => return Ok(None),
    };

    let obj = match layer.objects.iter().find(|obj| {
        actor_spawn_candidate_matches(
            actor_type,
            obj.kind.as_deref().unwrap_or_default(),
            layer.visible.unwrap_or(true),
            obj.visible.unwrap_or(true),
            obj.point,
        )
    }) {
        Some(obj) => obj,
        None => return Ok(None),
    };

    Ok(Some((
        obj.x.unwrap_or(0.0) + layer.offsetx.unwrap_or(0.0),
        obj.y.unwrap_or(0.0) + layer.offsety.unwrap_or(0.0),
    )))
}

fn actor_spawn_candidate_matches(
    expected_actor_type: &str,
    candidate_actor_type: &str,
    layer_visible: bool,
    object_visible: bool,
    is_point: bool,
) -> bool {
    layer_visible && object_visible && is_point && candidate_actor_type == expected_actor_type
}

#[derive(Clone, Debug)]
pub struct RuntimeMapAdapter {
    pub tile_size: f32,
    pub width: usize,
    pub height: usize,
    pub solid: Vec<bool>,
}

impl RuntimeMapAdapter {
    pub fn from_tiled_json_wall_layer(
        path: impl AsRef<Path>,
    ) -> Result<Self, RuntimeMapAdapterError> {
        Self::from_tiled_json_named_wall_layer(path, "Wall_Layer")
    }

    pub fn from_tiled_json_named_wall_layer(
        path: impl AsRef<Path>,
        wall_layer_name: &str,
    ) -> Result<Self, RuntimeMapAdapterError> {
        let path = path.as_ref();
        let map = read_tiled_json_map(path)?;

        if map.tilewidth != map.tileheight {
            return Err(RuntimeMapAdapterError::UnsupportedNonSquareTiles {
                tilewidth: map.tilewidth,
                tileheight: map.tileheight,
            });
        }

        let wall_layer = map
            .layers
            .iter()
            .find(|layer| layer.kind == "tilelayer" && layer.name == wall_layer_name)
            .ok_or_else(|| RuntimeMapAdapterError::MissingWallLayer {
                path: path.to_path_buf(),
                layer_name: wall_layer_name.to_owned(),
            })?;

        if wall_layer.width != map.width || wall_layer.height != map.height {
            return Err(RuntimeMapAdapterError::LayerDimensionsMismatch {
                layer_name: wall_layer.name.clone(),
                map_width: map.width,
                map_height: map.height,
                layer_width: wall_layer.width,
                layer_height: wall_layer.height,
            });
        }

        let expected_len = map.width * map.height;
        if wall_layer.data.len() != expected_len {
            return Err(RuntimeMapAdapterError::InvalidLayerDataLen {
                layer_name: wall_layer.name.clone(),
                expected: expected_len,
                actual: wall_layer.data.len(),
            });
        }

        let solid = wall_layer.data.iter().map(|gid| *gid != 0).collect();

        Ok(Self {
            tile_size: map.tilewidth as f32,
            width: map.width,
            height: map.height,
            solid,
        })
    }

    #[inline]
    pub fn world_to_grid(&self, x: f32, y: f32) -> (usize, usize) {
        ((x / self.tile_size) as usize, (y / self.tile_size) as usize)
    }

    #[inline]
    pub fn grid_to_world(&self, x: usize, y: usize) -> (f32, f32) {
        (
            (x as f32) * self.tile_size + self.tile_size / 2.0,
            (y as f32) * self.tile_size + self.tile_size / 2.0,
        )
    }

    #[inline]
    pub fn is_wall(&self, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height {
            return true;
        }
        self.solid[self.idx(x, y)]
    }

    pub fn blocked_for_agent(&self, x: usize, y: usize) -> bool {
        if self.is_wall(x, y) {
            return true;
        }

        let r: isize = 0;
        let xi = x as isize;
        let yi = y as isize;

        for dy in -r..=r {
            for dx in -r..=r {
                let nx = xi + dx;
                let ny = yi + dy;
                if nx < 0 || ny < 0 {
                    return true;
                }

                let nx = nx as usize;
                let ny = ny as usize;
                if nx >= self.width || ny >= self.height {
                    return true;
                }
                if self.solid[self.idx(nx, ny)] {
                    return true;
                }
            }
        }

        false
    }

    #[inline]
    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.tile_size.to_bits() == other.tile_size.to_bits()
            && self.solid == other.solid
    }

    pub fn summary(&self) -> RuntimeMapAdapterSummary {
        RuntimeMapAdapterSummary {
            tile_size: self.tile_size,
            width: self.width,
            height: self.height,
            solid_cells: self.solid.iter().filter(|&&v| v).count(),
        }
    }
}

fn read_tiled_json_map(path: &Path) -> Result<TiledJsonMap, RuntimeMapAdapterError> {
    let text = std::fs::read_to_string(path).map_err(|source| RuntimeMapAdapterError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| RuntimeMapAdapterError::Json {
        path: path.to_path_buf(),
        source,
    })
}

#[derive(Debug)]
pub enum RuntimeMapAdapterError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    MissingWallLayer {
        path: PathBuf,
        layer_name: String,
    },
    UnsupportedNonSquareTiles {
        tilewidth: u32,
        tileheight: u32,
    },
    LayerDimensionsMismatch {
        layer_name: String,
        map_width: usize,
        map_height: usize,
        layer_width: usize,
        layer_height: usize,
    },
    InvalidLayerDataLen {
        layer_name: String,
        expected: usize,
        actual: usize,
    },
}

impl fmt::Display for RuntimeMapAdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "failed to read {}: {}", path.display(), source),
            Self::Json { path, source } => {
                write!(
                    f,
                    "failed to parse tiled json {}: {}",
                    path.display(),
                    source
                )
            }
            Self::MissingWallLayer { path, layer_name } => write!(
                f,
                "missing tile layer '{}' in {}",
                layer_name,
                path.display()
            ),
            Self::UnsupportedNonSquareTiles {
                tilewidth,
                tileheight,
            } => write!(
                f,
                "non-square tiles are not supported yet ({}x{})",
                tilewidth, tileheight
            ),
            Self::LayerDimensionsMismatch {
                layer_name,
                map_width,
                map_height,
                layer_width,
                layer_height,
            } => write!(
                f,
                "layer '{}' dimensions {}x{} do not match map dimensions {}x{}",
                layer_name, layer_width, layer_height, map_width, map_height
            ),
            Self::InvalidLayerDataLen {
                layer_name,
                expected,
                actual,
            } => write!(
                f,
                "layer '{}' data length mismatch: expected {}, got {}",
                layer_name, expected, actual
            ),
        }
    }
}

impl std::error::Error for RuntimeMapAdapterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Json { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuntimeMapAdapterSummary {
    pub tile_size: f32,
    pub width: usize,
    pub height: usize,
    pub solid_cells: usize,
}

#[derive(Debug)]
pub enum RuntimeMapInstallError {
    IncompatibleExisting {
        existing: RuntimeMapAdapterSummary,
        attempted: RuntimeMapAdapterSummary,
    },
}

impl fmt::Display for RuntimeMapInstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IncompatibleExisting {
                existing,
                attempted,
            } => write!(
                f,
                "runtime map already installed with incompatible data (existing: {}x{} @ {}, solid {}; attempted: {}x{} @ {}, solid {})",
                existing.width,
                existing.height,
                existing.tile_size,
                existing.solid_cells,
                attempted.width,
                attempted.height,
                attempted.tile_size,
                attempted.solid_cells
            ),
        }
    }
}

impl std::error::Error for RuntimeMapInstallError {}

#[derive(Deserialize)]
struct TiledJsonMap {
    width: usize,
    height: usize,
    tilewidth: u32,
    tileheight: u32,
    layers: Vec<TiledJsonLayer>,
}

#[derive(Deserialize)]
struct TiledJsonLayer {
    name: String,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    visible: Option<bool>,
    #[serde(default)]
    offsetx: Option<f32>,
    #[serde(default)]
    offsety: Option<f32>,
    #[serde(default)]
    width: usize,
    #[serde(default)]
    height: usize,
    #[serde(default)]
    data: Vec<u32>,
    #[serde(default)]
    objects: Vec<TiledJsonObject>,
}

#[derive(Deserialize)]
struct TiledJsonObject {
    #[serde(default, rename = "type")]
    kind: Option<String>,
    #[serde(default)]
    x: Option<f32>,
    #[serde(default)]
    y: Option<f32>,
    #[serde(default)]
    point: bool,
    #[serde(default)]
    visible: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::actor_spawn_candidate_matches;

    #[test]
    fn shared_spawn_filter_rejects_hidden_wrong_type_or_non_point_candidates() {
        assert!(actor_spawn_candidate_matches(
            "Enemy", "Enemy", true, true, true
        ));
        assert!(!actor_spawn_candidate_matches(
            "Enemy", "Enemy", false, true, true
        ));
        assert!(!actor_spawn_candidate_matches(
            "Enemy", "Enemy", true, false, true
        ));
        assert!(!actor_spawn_candidate_matches(
            "Enemy", "Enemy", true, true, false
        ));
        assert!(!actor_spawn_candidate_matches(
            "Enemy", "Player", true, true, true
        ));
    }
}
