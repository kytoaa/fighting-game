use super::{AssetData, AssetManager, AssetNames, Image, Shader};

pub struct StaticAssets {
    pub images: &'static [(&'static str, Image)],
    pub shaders: &'static [(&'static str, Shader)],
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

            shaders: self
                .shaders
                .into_iter()
                .map(|(name, shader)| {
                    (
                        AssetNames::with_name(name),
                        Shader {
                            data: match shader.data {
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
