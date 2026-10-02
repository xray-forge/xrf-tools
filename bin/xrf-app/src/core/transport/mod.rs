//! The loopback HTTP server bulk bytes cross by, so a response is never answered on the window's thread.
//!
//! Tauri answers every IPC call on the thread pumping the window's messages, and a level's textures and sectors are
//! gigabytes: the window stopped answering Windows for seconds at a time. What returns bytes is a route here instead,
//! fetched by the page or by the renderer worker with the token `transport|get_endpoint` hands out. Many calls may go
//! as one batch (`TransportServer::BATCH_PATH`), each answered by its part as it finishes, so a level's thousand
//! textures are a few requests rather than a thousand.

mod transport_answer;
mod transport_batch_call;
mod transport_body;
mod transport_endpoint;
mod transport_origins;
mod transport_part;
mod transport_refusal;
mod transport_route;
mod transport_routes;
mod transport_server;
mod transport_token;

pub(crate) use transport_answer::TransportAnswer;
pub(crate) use transport_batch_call::TransportBatchCall;
pub(crate) use transport_body::TransportBody;
pub(crate) use transport_endpoint::TransportEndpoint;
pub(crate) use transport_origins::TransportOrigins;
pub(crate) use transport_part::TransportPart;
pub(crate) use transport_refusal::TransportRefusal;
pub(crate) use transport_route::TransportRoute;
pub(crate) use transport_routes::TransportRoutes;
pub(crate) use transport_server::TransportServer;
pub(crate) use transport_token::TransportToken;

#[cfg(test)]
mod tests;
