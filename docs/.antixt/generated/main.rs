use antixt::{Application, ClientAsset, Context, IntoResponse, Method, Response, Route};
#[path = "../../app/config.rs"]
mod antixt_module_0;
#[path = "../../app/benchmarks/page.rs"]
mod antixt_module_1;
#[path = "../../app/layout.rs"]
mod antixt_module_2;
#[path = "../../app/docs/page.rs"]
mod antixt_module_3;
#[path = "../../app/docs/[slug]/page.rs"]
mod antixt_module_4;
#[path = "../../app/page.rs"]
mod antixt_module_5;
#[path = "../../components/mod.rs"]
pub mod components;
fn handle_0(context: Context<'_>) -> Response {
    let page = antixt_module_1::page(context.clone());
    let page = antixt_module_2::layout(context, page);
    page.into_response()
}
fn handle_1(context: Context<'_>) -> Response {
    let page = antixt_module_3::page(context.clone());
    let page = antixt_module_2::layout(context, page);
    page.into_response()
}
fn handle_2(context: Context<'_>) -> Response {
    let params = antixt_module_4::Params {
        slug: context.param("slug").expect("matched route parameter"),
    };
    let page = antixt_module_4::page(context.clone(), params);
    let page = antixt_module_2::layout(context, page);
    page.into_response()
}
fn handle_3(context: Context<'_>) -> Response {
    let page = antixt_module_5::page(context.clone());
    let page = antixt_module_2::layout(context, page);
    page.into_response()
}
static ROUTES: &[Route] = &[
    Route::new(Method::Get, "/benchmarks", handle_0),
    Route::new(Method::Get, "/docs", handle_1),
    Route::new(Method::Get, "/docs/:slug", handle_2),
    Route::new(Method::Get, "/", handle_3),
];
static CLIENT_ASSETS: &[ClientAsset] = &[ClientAsset::new(
    "docs-search",
    include_str!("../../client/docs-search.js"),
)];
fn main() {
    let mut application = Application::new(ROUTES, CLIENT_ASSETS);
    if let Err(error) = antixt_module_0::configure(&mut application) {
        eprintln!("antixt: {error}");
        std::process::exit(1);
    }
    application.run();
}
