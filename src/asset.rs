use bevy_app::{App, Plugin};
use bevy_asset::{io::Reader, AssetApp, AssetLoader, AsyncReadExt, LoadContext};
use thiserror::Error;

use crate::LookupCurve;

pub(crate) struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<LookupCurve>();
        app.register_asset_loader(LookupCurveAssetLoader);
    }
}

#[derive(Default)]
pub struct LookupCurveAssetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum LookupCurveAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load lookup curve: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON for lookup curve: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for LookupCurveAssetLoader {
    type Asset = LookupCurve;
    type Settings = ();
    type Error = LookupCurveAssetLoaderError;

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

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum LookupCurveAssetSaverError {
    /// An [IO](std::io) Error
    #[error("Could not save lookup curve: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not serialize lookup curve to RON: {0}")]
    RonError(#[from] ron::error::Error),
}

/// Serializes the lookup curve and saves it as a RON file
pub fn save_lookup_curve(
    path: &str,
    curve: &LookupCurve,
) -> Result<(), LookupCurveAssetSaverError> {
    let config = ron::ser::PrettyConfig::new()
        .new_line("\n".to_string())
        .indentor("  ".to_string());

    let s = ron::ser::to_string_pretty(curve, config)?;
    std::fs::write(path, s.as_bytes())?;

    Ok(())
}

// pub struct LookupCurveAssetSaver;

// #[non_exhaustive]
// #[derive(Debug, Error)]
// pub enum LookupCurveAssetSaverError {
//   /// An [IO](std::io) Error
//   #[error("Could not save lookup curve: {0}")]
//   Io(#[from] std::io::Error),
//   /// A [RON](ron) Error
//   #[error("Could not serialize lookup curve to RON: {0}")]
//   RonError(#[from] ron::error::Error),
// }

// impl AssetSaver for LookupCurveAssetSaver {
//   type Asset = LookupCurve;
//   type Settings = ();
//   type OutputLoader = LookupCurveAssetLoader;
//   type Error = LookupCurveAssetSaverError;

//   fn save<'a>(
//     &'a self,
//     writer: &'a mut Writer,
//     asset: SavedAsset<'a, Self::Asset>,
//     _settings: &'a Self::Settings,
//   ) -> BoxedFuture<'a, Result<(), Self::Error>> {
//     Box::pin(async move {
//       let config = ron::ser::PrettyConfig::new()
//         .new_line("\n".to_string())
//         .indentor("  ".to_string());

//       let s = ron::ser::to_string_pretty(&*asset, config)?;

//       writer.write_all(s.as_bytes()).await?;
//       Ok(())
//     })
//   }
// }
