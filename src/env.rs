pub(crate) fn get_env_opt(env: &str) -> Option<String> {
    std::env::var(env).ok()
}

pub(crate) fn get_env(env: &str) -> String {
    get_env_opt(env).unwrap_or_else(|| panic!("Could not get the {} env variable", env))
}
