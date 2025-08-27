fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    moonc_wasm::run_moonc(args)
}