mod cli;
mod storage;
mod tui;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        tui::run()
    } else {
        cli::run()
    }
}
