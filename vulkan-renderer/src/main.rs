fn main() {
    match vulkan_renderer::App::run() {
        Ok(_) => {}
        Err(e) => println!("[ERROR] - {:?}", e),
    }
}
