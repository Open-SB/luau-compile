use axum::{http::StatusCode, routing::post, Json, Router};
use mlua::{Compiler, Error as LuaError};
use serde::Deserialize;

fn create_router() -> Router {
    // Developer notes: https://docs.rs/axum/latest/axum/extract/index.html#common-extractors
    Router::new().route("/compile", post(compile_route))
}

#[cfg(not(feature = "shuttle"))]
#[tokio::main]
async fn main() {
    let router = create_router();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

#[cfg(feature = "shuttle")]
#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = create_router();
    Ok(router.into())
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

async fn compile_route(
    Json(payload): Json<CompilerPayload>,
) -> Result<Vec<u8>, (StatusCode, String)> {
    let options = payload.options;
    let compiler = Compiler::new()
        .set_coverage_level(options.coverage_level)
        .set_debug_level(options.debug_level)
        .set_optimization_level(options.optimization_level)
        .set_type_info_level(options.type_info_level)
        .set_vector_ctor(format!("{}.{}", options.vector_lib, options.vector_ctor))
        .set_vector_type(options.vector_type);

    // Compile calls into luacode.h's luau_compile
    let result = compiler.compile(payload.source);
    drop(compiler);

    match result {
        // If OK then just return bytecode
        Ok(bytecode) => Ok(bytecode),

        Err(err) => match err {
            // If it is a syntax error, we return a 422 Unprocessable Content with the error message as body
            LuaError::SyntaxError {
                message,
                incomplete_input: _,
            } => Err((StatusCode::UNPROCESSABLE_ENTITY, message)),

            // Out of memory error
            LuaError::MemoryError(message) => Err((StatusCode::INSUFFICIENT_STORAGE, message)),

            _ => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unknown error".to_string(),
            )),
        },
    }
}
