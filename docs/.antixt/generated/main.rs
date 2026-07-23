use antixt::{ClientAsset, Context, IntoResponse, Method, Response, Route};
#[path = "../../app/benchmarks/page.rs"]
mod antixt_module_0;
#[path = "../../app/layout.rs"]
mod antixt_module_1;
#[path = "../../app/docs/page.rs"]
mod antixt_module_2;
#[path = "../../app/docs/[slug]/page.rs"]
mod antixt_module_3;
#[path = "../../app/page.rs"]
mod antixt_module_4;
#[path = "../../components/mod.rs"]
pub mod components;
fn handle_0(context: Context<'_>) -> Response {
    let page = antixt_module_0::page(context);
    let page = antixt_module_1::layout(page);
    page.into_response()
}
fn handle_1(context: Context<'_>) -> Response {
    let page = antixt_module_2::page(context);
    let page = antixt_module_1::layout(page);
    page.into_response()
}
fn handle_2(context: Context<'_>) -> Response {
    let params = antixt_module_3::Params {
        slug: context.param("slug").expect("matched route parameter"),
    };
    let page = antixt_module_3::page(context, params);
    let page = antixt_module_1::layout(page);
    page.into_response()
}
fn handle_3(context: Context<'_>) -> Response {
    let page = antixt_module_4::page(context);
    let page = antixt_module_1::layout(page);
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
    antixt::server::run(ROUTES, CLIENT_ASSETS);
}
