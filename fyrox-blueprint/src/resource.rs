use fyrox::{
    asset::{
        io::ResourceIo,
        loader::{BoxedLoaderFuture, LoaderPayload, ResourceLoader},
        manager::ResourceManager,
        state::LoadError,
        Resource, ResourceData,
    },
    core::{
        reflect::prelude::*,
        type_traits::prelude::*,
        uuid::Uuid,
        visitor::prelude::*,
        SafeLock,
        TypeUuidProvider,
    },
};
use fyrox_visual_scripting::{model::GraphId, BlueprintGraph};
use std::{
    error::Error,
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(TypeUuidProvider, Debug, Clone, Visit, Reflect)]
#[type_uuid(id = "7f0ab4b7-3d28-45f1-a0e2-25f2cd5d4c10")]
pub struct BlueprintAsset {
    /// Asset format version. Useful for future migrations.
    #[visit(optional)]
    pub version: u32,

    /// Engine-agnostic serialized graph payload.
    ///
    /// For MVP this is JSON of `fyrox_visual_scripting::BlueprintGraph`.
    #[visit(optional)]
    pub graph_json: String,
}

impl Default for BlueprintAsset {
    fn default() -> Self {
        let graph = BlueprintGraph::new(GraphId("Blueprint".to_string()));
        let graph_json = serde_json::to_string(&graph).unwrap_or_else(|_| {
            // Fall back to a minimal, but valid graph JSON payload.
            r#"{\"id\":{\"0\":\"Blueprint\"},\"nodes\":{},\"links\":[],\"next_node_id\":1,\"next_pin_id\":1}"#
                .to_string()
        });

        Self {
            version: 1,
            graph_json,
        }
    }
}

impl BlueprintAsset {
    pub async fn from_file(path: &Path, io: &dyn ResourceIo) -> Result<Self, VisitError> {
        let bytes = io.load_file(path).await?;
        let mut visitor = Visitor::load_from_memory(&bytes)?;
        let mut asset = BlueprintAsset::default();
        asset.visit("Blueprint", &mut visitor)?;
        Ok(asset)
    }
}

pub type BlueprintResource = Resource<BlueprintAsset>;

impl ResourceData for BlueprintAsset {
    fn type_uuid(&self) -> Uuid {
        <Self as TypeUuidProvider>::type_uuid()
    }

    fn save(&mut self, path: &Path) -> Result<(), Box<dyn Error>> {
        let mut visitor = Visitor::new();
        self.visit("Blueprint", &mut visitor)?;
        visitor.save_ascii_to_file(path)?;
        Ok(())
    }

    fn can_be_saved(&self) -> bool {
        true
    }

    fn try_clone_box(&self) -> Option<Box<dyn ResourceData>> {
        Some(Box::new(self.clone()))
    }
}

#[derive(Default)]
pub struct BlueprintLoader;

impl BlueprintLoader {
    pub const EXT: &'static str = "blueprint";
}

impl ResourceLoader for BlueprintLoader {
    fn extensions(&self) -> &[&str] {
        &[Self::EXT]
    }

    fn is_native_extension(&self, ext: &str) -> bool {
        fyrox::core::cmp_strings_case_insensitive(ext, Self::EXT)
    }

    fn data_type_uuid(&self) -> Uuid {
        <BlueprintAsset as TypeUuidProvider>::type_uuid()
    }

    fn load(&self, path: PathBuf, io: Arc<dyn ResourceIo>) -> BoxedLoaderFuture {
        Box::pin(async move {
            let asset = BlueprintAsset::from_file(&path, io.as_ref())
                .await
                .map_err(LoadError::new)?;
            Ok(LoaderPayload::new(asset))
        })
    }
}

/// Registers `.blueprint` resource data + loader in the given resource manager.
///
/// Note: if the manager already loaded/scanned its registry, call
/// `resource_manager.state().update_or_load_registry()` afterwards to re-scan.
pub fn register_resources(resource_manager: &ResourceManager) {
    let state = resource_manager.state();

    state.constructors_container.add::<BlueprintAsset>();

    let mut loaders = state.loaders.safe_lock();
    loaders.set(BlueprintLoader);
}

#[cfg(test)]
mod tests {
    use super::*;
    use fyrox::core::futures::executor::block_on;
    use fyrox::asset::io::FsResourceIo;

    #[test]
    fn blueprint_asset_save_load_roundtrip() {
        let dir = std::env::temp_dir().join("fyrox_blueprint_tests");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join(format!("roundtrip.{}", BlueprintLoader::EXT));

        let mut asset = BlueprintAsset::default();
        asset.save(&path).unwrap();

        let io = FsResourceIo;
        let loaded = block_on(BlueprintAsset::from_file(&path, &io)).unwrap();
        assert_eq!(loaded.version, asset.version);
        assert_eq!(loaded.graph_json, asset.graph_json);
    }
}
