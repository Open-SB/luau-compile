use axum::{extract::State, routing::post, Json, Router};
use mlua::Compiler;
use serde::Deserialize;

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
    let router = Router::new().route("/compile", post(compile_route));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

// https://github.com/mlua-rs/mlua/blob/b77836920a5db89067892f4fd9c88db1a0483a8a/src/chunk.rs#L122
#[derive(Deserialize)]
pub struct CompilerOptions {
    pub coverage_level: u8,
    pub debug_level: u8,
    pub optimization_level: u8,
    pub type_info_level: u8,
    pub vector_lib: String,
    pub vector_ctor: String,
    pub vector_type: String,
}

#[derive(Deserialize)]
pub struct CompilerPayload {
    pub source: String,
    pub options: CompilerOptions,
}

async fn compile_route(Json(payload): Json<CompilerPayload>) -> Vec<u8> {
    let options = payload.options;
    let compiler = Compiler::new()
        .set_coverage_level(options.coverage_level)
        .set_debug_level(options.debug_level)
        .set_optimization_level(options.optimization_level)
        .set_type_info_level(options.type_info_level)
        .set_vector_lib(options.vector_lib)
        .set_vector_ctor(options.vector_ctor)
        .set_vector_type(options.vector_type);
    // TODO: Check if the compiler outputs valid bytecode; as it puts errors into our output
    let source = compiler.compile(payload.source);
    drop(compiler);
    source
}
