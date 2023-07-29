use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // if cfg!(target_os = "windows") {
    //     let mut res = winres::WindowsResource::new();
    //     res.set_icon("assets\\tori.ico");
    //     res.compile()?;
    // }

    Ok(())
}
