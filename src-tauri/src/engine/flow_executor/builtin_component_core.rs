use crate::app_state::AppState;
use crate::engine::flow_executor::ExecutionContext;
use crate::models::DataEnvelope;
use anyhow::Result;
use strum_macros::EnumString;
use tauri::AppHandle;

mod call_flow;
mod clear_clipboard;
mod close_window;
mod copy;
mod emit_data;
mod eval_js;
mod html_window;
mod http_request;
mod key_listener;
mod llm;
mod ocr;
mod paste;
mod popup;
mod read_clipboard;
mod regex_replace;
mod router;
mod screen_capture;
mod shell;
mod simulate_key;
mod simulate_mouse;
mod typing;
mod write_clipboard;

#[derive(EnumString)]
pub enum BuiltinComponentType {
    CallFlow,
    ClearClipboard,
    CloseWindow,
    Copy,
    EmitData,
    EvalJs,
    HtmlWindow,
    HttpRequest,
    KeyListener,
    LLM,
    OCR,
    Popup,
    Paste,
    ReadClipboard,
    Regex,
    Router,
    ScreenCapture,
    SimulateKey,
    SimulateMouse,
    Typing,
    WriteClipboard,

    /// 自定义组件的本质是内置组件 Shell
    Shell,
}

pub async fn run_component_logic(
    app: AppHandle,
    _state: &AppState,
    builtin_comp_type: BuiltinComponentType,
    ctx: ExecutionContext,
) -> Result<DataEnvelope> {
    use BuiltinComponentType::*;
    match builtin_comp_type {
        ScreenCapture => screen_capture::execute().await,
        LLM => llm::execute(ctx).await,
        Paste => paste::execute(ctx).await,
        Copy => copy::execute().await,
        EmitData => emit_data::execute(ctx).await,
        EvalJs => eval_js::execute(app, ctx).await,
        ReadClipboard => read_clipboard::execute().await,
        Popup => popup::execute(app, ctx).await,
        HtmlWindow => html_window::execute(app, ctx).await,
        HttpRequest => http_request::execute(ctx).await,
        KeyListener => key_listener::execute(app, ctx).await,
        Typing => typing::execute(ctx).await,
        OCR => ocr::execute(ctx).await,
        WriteClipboard => write_clipboard::execute(ctx).await,
        Regex => regex_replace::execute(ctx).await,
        Router => router::execute(app, ctx).await,
        SimulateKey => simulate_key::execute(ctx).await,
        SimulateMouse => simulate_mouse::execute(ctx).await,
        CallFlow => call_flow::execute(app, ctx).await,
        ClearClipboard => clear_clipboard::execute().await,
        CloseWindow => close_window::execute(app, ctx).await,
        Shell => shell::execute(ctx).await,
    }
}
