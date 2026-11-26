/// What a read of a level answers: its value, or why it could not be read, as text a host shows as it is.
// todo: Answer `XrfResult` and leave the text to the host that shows it, once the readers' messages are typed errors.
pub type LevelSourceResult<T> = Result<T, String>;
