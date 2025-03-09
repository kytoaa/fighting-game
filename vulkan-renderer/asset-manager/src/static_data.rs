use super::{AssetData, AssetManager, AssetNames, Image};

pub struct StaticAssets {
    pub images: &'static [(&'static str, Image)],
}

impl Into<AssetManager> for StaticAssets {
    fn into(self) -> AssetManager {
        AssetManager {
            images: self
                .images
                .into_iter()
                .map(|(name, image)| {
                    (
                        AssetNames::with_name(name),
                        Image {
                            size: image.size,
                            data: match image.data {
                                AssetData::Static(v) => AssetData::Static(v),
                                AssetData::Dynamic(_) => unreachable!(),
                            },
                        },
                    )
                })
                .collect(),
        }
    }
}
