/// Work run in a change's turn: telling everyone of the change, or what a follow put off until it is heard.
pub(crate) type AssetChangeWork = Box<dyn FnOnce() + Send>;
