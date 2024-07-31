use bevy_app::{App, Plugin};
use bevy_asset::{io::Reader, AssetApp, AssetLoader, AsyncReadExt, LoadContext};

use crate::{LookupCurve, LookupCurveLoadError};

pub(crate) struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<LookupCurve>();
        app.register_asset_loader(LookupCurveAssetLoader);
    }
}

#[derive(Default)]
pub struct LookupCurveAssetLoader;

impl AssetLoader for LookupCurveAssetLoader {
    type Asset = LookupCurve;
    type Settings = ();
    type Error = LookupCurveLoadError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let lookup_curve = ron::de::from_bytes::<LookupCurve>(&bytes)?;
        Ok(lookup_curve)
    }

    fn extensions(&self) -> &[&str] {
        &["curve.ron"]
    }
}
