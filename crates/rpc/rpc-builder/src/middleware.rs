use crate::metrics::RpcRequestMetrics;
use jsonrpsee::server::middleware::rpc::RpcService;
use tower::{
    layer::util::{Identity, Stack},
    BoxError, Layer, Service,
};

/// A Helper alias trait for the RPC middleware supported by the server.
pub trait RethRpcMiddleware:
    Layer<
        RpcService,
        Service: jsonrpsee::server::middleware::rpc::RpcServiceT<
            MethodResponse = jsonrpsee::MethodResponse,
            BatchResponse = jsonrpsee::MethodResponse,
            NotificationResponse = jsonrpsee::MethodResponse,
        > + Send
                     + Sync
                     + Clone
                     + 'static,
    > + Clone
    + Send
    + 'static
{
}

impl<T> RethRpcMiddleware for T where
    T: Layer<
            RpcService,
            Service: jsonrpsee::server::middleware::rpc::RpcServiceT<
                MethodResponse = jsonrpsee::MethodResponse,
                BatchResponse = jsonrpsee::MethodResponse,
                NotificationResponse = jsonrpsee::MethodResponse,
            > + Send
                         + Sync
                         + Clone
                         + 'static,
        > + Clone
        + Send
        + 'static
{
}

/// A Helper alias trait for the HTTP middleware supported by the server.
pub trait RethHttpMiddleware<RpcMiddleware>:
    Layer<
        jsonrpsee::server::TowerServiceNoHttp<
            Stack<RpcMiddleware, Stack<RpcRequestMetrics, Identity>>,
        >,
        Service: Service<
            jsonrpsee::server::HttpRequest,
            Response = jsonrpsee::server::HttpResponse<jsonrpsee::server::HttpBody>,
            Error = BoxError,
            Future: Send,
        > + Clone
                     + Send
                     + 'static,
    > + Clone
    + Send
    + 'static
where
    RpcMiddleware: RethRpcMiddleware,
{
}

impl<T, RpcMiddleware> RethHttpMiddleware<RpcMiddleware> for T
where
    RpcMiddleware: RethRpcMiddleware,
    T: Layer<
            jsonrpsee::server::TowerServiceNoHttp<
                Stack<RpcMiddleware, Stack<RpcRequestMetrics, Identity>>,
            >,
            Service: Service<
                jsonrpsee::server::HttpRequest,
                Response = jsonrpsee::server::HttpResponse<jsonrpsee::server::HttpBody>,
                Error = BoxError,
                Future: Send,
            > + Clone
                         + Send
                         + 'static,
        > + Clone
        + Send
        + 'static,
{
}
