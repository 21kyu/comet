pub mod logging;

// temporary
#[macro_export]
macro_rules! run_command {
    ($command:expr $(, $args:expr)*) => {
        std::process::Command::new($command).args([$($args),*]).output()
            .expect("failed to run command")
    };
}
