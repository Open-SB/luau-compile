use axum::{extract::State, routing::post, Router};
use mlua::Compiler;

#[derive(Clone, Debug)]
struct AppState {
    compiler: Compiler,
}

#[tokio::main]
async fn main() {
    // Developer notes: https://docs.rs/axum/latest/axum/extract/index.html#common-extractors
    tracing_subscriber::fmt::init();

    let compiler = Compiler::new()
        .set_coverage_level(2) // ??? unknown
        .set_debug_level(2) // full debug level; includes upvalues
        .set_optimization_level(2) // inlines functions; hurts debugging
        .set_type_info_level(1) // doesn't matter; fiu isn't native codegen
        // roblox...
        .set_vector_lib("Vector3")
        .set_vector_ctor("new")
        .set_vector_type("Vector3");
    let router = Router::new()
        .route("/compile", post(compile_route))
        .with_state(AppState { compiler });
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

async fn compile_route(State(state): State<AppState>, body: String) -> Vec<u8> {
    // TODO: Check if the compiler outputs valid bytecode; as it puts errors into our output
    state.compiler.compile(body)
}
