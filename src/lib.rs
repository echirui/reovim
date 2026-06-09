pub mod sha256;
pub mod path;

#[unsafe(no_mangle)]
pub extern "C" fn reovim_hello() {
    println!("Hello from Rust!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reovim_hello() {
        // Assert that calling the function executes successfully without panicking.
        reovim_hello();
    }
}
