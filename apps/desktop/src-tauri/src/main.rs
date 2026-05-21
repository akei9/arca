fn main() {
    if let Err(error) = arca_desktop::run() {
        eprintln!("failed to run Arca: {error}");
        std::process::exit(1);
    }
}
