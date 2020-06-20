mod console;
mod manager;
mod modules;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = console::Console::new()?;
    session.manager.load_modules()?;
    session.repl()?;
    Ok(())
}
