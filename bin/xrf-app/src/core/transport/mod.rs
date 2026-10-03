//! The loopback HTTP server bulk bytes cross by, so a response is never answered on the window's thread.
//!
//! Tauri answers every IPC call on the thread pumping the window's messages, and an archive's file or a texture's texels
//! can be hundreds of megabytes: the window would stop answering Windows while they cross. What returns bytes is a
//! route here instead, fetched by the page with the token `transport|get_endpoint` hands out.

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
