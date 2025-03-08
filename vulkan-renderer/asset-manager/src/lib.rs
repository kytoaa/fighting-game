pub mod static_data;

#[derive(Clone, Copy)]
pub struct SpriteHandle(usize);
#[derive(Clone, Copy)]
pub struct ShaderHandle(usize);

pub struct Image {
    size: (usize, usize),
    data: AssetData,
}

pub struct Shader {
    data: AssetData,
}

pub enum AssetData {
    Dynamic(Box<[u8]>),
    Static(&'static [u8]),
}

impl Image {
    pub const fn from(value: ((usize, usize), &'static [u8])) -> Self {
        Self {
            size: value.0,
            data: AssetData::Static(value.1),
        }
    }
}
impl Shader {
    pub const fn from(data: &'static [u8]) -> Self {
        Self {
            data: AssetData::Static(data),
        }
    }
}
impl AssetData {
    pub const fn data<'a>(&'a self) -> &'a [u8] {
        match self {
            Self::Static(v) => v,
            Self::Dynamic(v) => &v,
        }
    }
}

pub struct AssetManager {
    images: Vec<(AssetNames, Image)>,
    shaders: Vec<(AssetNames, Shader)>,
}

impl AssetManager {}

pub struct AssetNames {
    name: &'static str,
    aliases: Vec<Box<str>>,
}

impl AssetManager {
    pub fn get_sprite(&self, handle: SpriteHandle) -> &Image {
        &self.images[handle.0].1
    }
    pub fn get_sprite_handle(&self, sprite_name: &str) -> Option<SpriteHandle> {
        self.images
            .iter()
            .enumerate()
            .find(|(_, image)| {
                image.0.name == sprite_name
                    || image
                        .0
                        .aliases
                        .iter()
                        .find(|alias| alias.as_ref() == sprite_name)
                        .is_some()
            })
            .map(|v| SpriteHandle(v.0))
    }
    pub fn get_shader(&self, handle: ShaderHandle) -> &Shader {
        &self.shaders[handle.0].1
    }
    pub fn get_shader_handle(&self, shader_name: &str) -> Option<ShaderHandle> {
        self.shaders
            .iter()
            .enumerate()
            .find(|(_, shader)| {
                shader.0.name == shader_name
                    || shader
                        .0
                        .aliases
                        .iter()
                        .find(|alias| alias.as_ref() == shader_name)
                        .is_some()
            })
            .map(|v| ShaderHandle(v.0))
    }
}

impl AssetNames {
    pub const fn with_name(name: &'static str) -> Self {
        Self {
            name,
            aliases: vec![],
        }
    }
}
