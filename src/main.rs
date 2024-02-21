mod gpu_renderer;
mod soft_renderer;

fn main() {
    // soft_renderer::run((400, 400)).unwrap();
    gpu_renderer::run((400, 400)).unwrap();
}
