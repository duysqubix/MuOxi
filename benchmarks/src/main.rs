mod io;
mod report;

use io::benchmark_io_json_100_000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    benchmark_io_json_100_000()?;
    Ok(())
}
