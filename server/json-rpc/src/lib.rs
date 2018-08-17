extern crate jsonrpc_http_server;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate graph;

use graph::prelude::{JsonRpcServer as JsonRpcServerTrait, *};
use jsonrpc_http_server::{
    jsonrpc_core::{self, IoHandler, Params, Response, Value},
    RestApi, Server, ServerBuilder,
};
use std::fmt;
use std::io;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;

//pub use 
#[derive(Debug, Serialize, Deserialize)]
struct SubgraphAddParams {
    name: String,
    ipfs_hash: String,
}

impl fmt::Display for SubgraphAddParams {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self)
    }
}

pub struct JsonRpcServer {}

impl JsonRpcServerTrait for JsonRpcServer {
    fn serve(
        port: u16,
        provider: Arc<impl SubgraphProvider>,
        logger: Logger,
    ) -> Result<Server, io::Error> {
        let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port);

        let mut handler = IoHandler::new();

        // `subgraph_add` handler.
        let add_provider = provider.clone();
        let add_logger = logger.clone();
        let subgraph_added = false;
        handler.add_method("subgraph_add", move |params: Params| {
            let provider = add_provider.clone();
            let logger = add_logger.clone();
            future::result(params.parse()).and_then(move |params: SubgraphAddParams| {
                info!(logger, "Received subgraph_add request"; "params" => params.to_string());

                if subgraph_added {
                    Err(json_rpc_error(
                        1,
                        "adding multiple subgraphs is not yet supported".to_owned(),
                    ))
                } else {
                    Ok(())
                }.into_future()
                    .and_then(move |_| {
                        provider
                            .add(format!("/ipfs/{}", params.ipfs_hash))
                            .map_err(|e| json_rpc_error(0, e.to_string()))
                            .map(|_| Ok(Value::Null))
                            .flatten()
                    })
            })
        });

        ServerBuilder::new(handler)
            // Enable REST API:
            // POST /<method>/<param1>/<param2>
            .rest_api(RestApi::Secure)
            .start_http(&addr.into())
    }
}

fn json_rpc_error(code: i64, message: String) -> jsonrpc_core::Error {
    jsonrpc_core::Error {
        code: jsonrpc_core::ErrorCode::ServerError(code),
        message,
        data: None,
    }
}

pub fn json_rpc_result(response: Response) -> jsonrpc_core::Result<Value> {
    match response {
        Response::Single(response) => response.into(),
        Response::Batch(_) => panic!("batch responses not supported"),
    }
}
