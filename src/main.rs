fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = &args[1];
    rust_png_reader::read_and_print_png(path);
}
