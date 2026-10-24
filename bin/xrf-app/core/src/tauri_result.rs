/// What a command answers: its value, or the reason it failed as the page shows it.
pub type TauriResult<T = ()> = Result<T, String>;
