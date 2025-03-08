use asset_manager::*;
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro]
pub fn raw_image_as_rgba(input: TokenStream) -> TokenStream {
    let ast: syn::Lit = syn::parse(input).unwrap();
    let s = match ast {
        syn::Lit::Str(s) => s,
        _ => panic!("must be a string literal"),
    };

    let file = image::ImageReader::open(s.value()).expect("could not read file");
    let image = file.decode().expect("failed to decode image").into_rgba8();

    let (width, height) = (image.width() as usize, image.height() as usize);
    let pixels = image.pixels().map(|pixel| pixel.0).flatten();
    quote! {
        (
            (#width, #height),
            &[#(#pixels),*]
        )
    }
    .into()
}

#[proc_macro]
pub fn generate_static_asset_manager_from_dir(input: TokenStream) -> TokenStream {
    let dir: syn::ExprLit = syn::parse(input).unwrap();
    let root_dir = match dir.lit {
        syn::Lit::Str(val) => val.value(),
        _ => panic!("not an address"),
    };
    let addresses: Vec<_> = find_image_paths(&root_dir)
        .filter(|file| match file.extension().unwrap().to_str().unwrap() {
            "png" | "spv" => true,
            _ => false,
        })
        .collect();

    if addresses.len() == 0 {
        panic!(
            "dir {:?} has entries {:?}",
            &root_dir,
            std::fs::read_dir(&root_dir).unwrap().collect::<Vec<_>>()
        );
    }

    let mut images = vec![];
    let mut shaders = vec![];
    for addr in addresses.iter() {
        match addr.extension().unwrap().to_str().unwrap() {
            ext if ext == "png" => {
                let file = image::ImageReader::open(addr).expect("could not read file");
                let image = file.decode().expect("failed to decode image").into_rgba8();

                let (width, height) = (image.width() as usize, image.height() as usize);
                let pixels = image.pixels().map(|pixel| pixel.0).flatten();

                let addr = addr.to_str().unwrap();

                images.push(quote! {
                    (
                        #addr,
                        asset_manager::Image::from(((#width, #height), &[#(#pixels),*]))
                    )
                });
            }
            ext if ext == "spv" => {
                let data = std::fs::read(addr).expect("could not read file");

                let addr = addr.to_str().unwrap();

                shaders.push(quote! {
                    (
                        #addr,
                        asset_manager::Shader::from(&[#(#data),*])
                    )
                });
            }
            _ => unreachable!(),
        }
    }

    quote! {
        asset_manager::static_data::StaticAssets {
            images: &[#(#images),*],
            shaders: &[#(#shaders),*],
        }
    }
    .into()
}

fn find_image_paths(
    path: &impl AsRef<std::path::Path>,
) -> impl Iterator<Item = std::path::PathBuf> {
    std::fs::read_dir(path)
        .unwrap()
        .map(|r| r.unwrap())
        .map(|dir_entry| {
            if dir_entry.path().is_dir() {
                find_image_paths(&dir_entry.path()).collect()
            } else {
                vec![dir_entry.path()]
            }
            .into_iter()
        })
        .flatten()
}
