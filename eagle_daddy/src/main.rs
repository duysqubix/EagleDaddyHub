mod console;
mod manager;
mod modules;
mod prelude;
use rustbee::{api, device::DigiMeshDevice};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = console::Console::new()?;
    session.repl()?;
    Ok(())
}
