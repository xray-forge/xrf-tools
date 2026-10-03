//! The loopback HTTP server bulk bytes cross by, so a response is never answered on the window's thread.
//!
//! Tauri answers every IPC call on the thread pumping the window's messages, and a level's textures and sectors are
//! gigabytes: the window stopped answering Windows for seconds at a time. What returns bytes is a route here instead,
//! fetched by the page with the token `transport|get_endpoint` hands out.

mod transport_answer;
mod transport_endpoint;
mod transport_origins;
mod transport_refusal;
mod transport_route;
mod transport_routes;
mod transport_server;
mod transport_token;

pub(crate) use transport_answer::TransportAnswer;
pub(crate) use transport_endpoint::TransportEndpoint;
pub(crate) use transport_origins::TransportOrigins;
pub(crate) use transport_refusal::TransportRefusal;
pub(crate) use transport_route::TransportRoute;
pub(crate) use transport_routes::TransportRoutes;
pub(crate) use transport_server::TransportServer;
pub(crate) use transport_token::TransportToken;

#[cfg(test)]
mod tests;
