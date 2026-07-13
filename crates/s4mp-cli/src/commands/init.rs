use s4mp_core::Result;
use s4mp_workspace::Workspace;

pub fn run(path: &str) -> Result<()> {
    let _workspace = Workspace::open(path);
    println!("initialized S4MP workspace at {path}");
    Ok(())
}
