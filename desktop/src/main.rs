slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe { std::env::set_var("SLINT_BACKEND", "winit-software"); }

    let app = AppView::new()?;

    app.run().expect("failed to run");
    Ok(())
}
