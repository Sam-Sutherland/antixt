use crate::Html;
use std::any::{Any, TypeId, type_name};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::pin::Pin;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::task::{Context as TaskContext, Poll, Wake, Waker};
use std::thread;
use std::time::{Duration, Instant};

const MAX_REQUEST_BYTES: usize = 1024 * 1024;

const RELOAD_CLIENT: &str = r#"<script data-antixt-dev>
(() => {
  let version = null;
  setInterval(async () => {
    try {
      const next = await fetch('/__antixt/version', { cache: 'no-store' }).then(response => response.text());
      if (version === null) version = next;
      else if (version !== next) location.reload();
    } catch {}
  }, 100);
})();
</script>"#;

const CLIENT_RUNTIME: &str = r#"<script data-antixt-client>
(() => {
  const clientVersion = '__ANTIXT_CLIENT_QUERY__';
  const pending = new WeakMap();
  const mount = async (root = document) => {
    for (const island of root.querySelectorAll('[data-antixt-island]:not([data-antixt-mounted])')) {
      island.setAttribute('data-antixt-mounted', '');
      const name = island.getAttribute('data-antixt-island');
      const path = name.split('/').map(encodeURIComponent).join('/');
      try { (await import('/__antixt/client/' + path + '.js' + clientVersion)).default?.(island); }
      catch (error) { console.error('antixt island failed:', name, error); }
    }
  };
  const update = async (source, method, url, body) => {
    const selector = source.getAttribute('data-antixt-target');
    const target = selector ? document.querySelector(selector) : source;
    if (!target) return;
    pending.get(target)?.abort();
    const controller = new AbortController();
    pending.set(target, controller);
    try {
      const response = await fetch(url, {
        method,
        body,
        signal: controller.signal,
        headers: { 'antixt-fragment': 'true' },
      });
      if (response.redirected) { location.assign(response.url); return; }
      const html = await response.text();
      if (pending.get(target) !== controller) return;
      if (source.getAttribute('data-antixt-swap') === 'outer') target.outerHTML = html;
      else target.innerHTML = html;
      await mount(document);
    } catch (error) {
      if (error.name !== 'AbortError') console.error('antixt fragment failed:', error);
    } finally {
      if (pending.get(target) === controller) pending.delete(target);
    }
  };
  document.addEventListener('click', event => {
    const explicit = event.target.closest('[data-antixt-get],[data-antixt-post]');
    const submitted = event.target.closest('form[data-antixt-fragment]');
    const source = explicit || submitted;
    if (!source) return;
    event.preventDefault();
    if (source.matches('form')) {
      const method = (source.method || 'GET').toUpperCase();
      const values = new URLSearchParams(new FormData(source));
      const url = method === 'GET' ? source.action + '?' + values : source.action;
      update(source, method, url, method === 'GET' ? undefined : values);
    } else {
      const method = source.hasAttribute('data-antixt-post') ? 'POST' : 'GET';
      const url = source.getAttribute(method === 'POST' ? 'data-antixt-post' : 'data-antixt-get');
      update(source, method, url);
    }
  });
  document.addEventListener('submit', event => {
    const form = event.target.closest('form[data-antixt-fragment]');
    if (!form) return;
    event.preventDefault();
    const method = (form.method || 'GET').toUpperCase();
    const values = new URLSearchParams(new FormData(form));
    const url = method === 'GET' ? form.action + '?' + values : form.action;
    update(form, method, url, method === 'GET' ? undefined : values);
  });
  mount();
})();
</script>"#;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl Method {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "GET" => Some(Self::Get),
            "POST" => Some(Self::Post),
            "PUT" => Some(Self::Put),
            "PATCH" => Some(Self::Patch),
            "DELETE" => Some(Self::Delete),
            _ => None,
        }
    }

    pub const fn function_name(self) -> &'static str {
        match self {
            Self::Get => "page",
            Self::Post => "post",
            Self::Put => "put",
            Self::Patch => "patch",
            Self::Delete => "delete",
        }
    }

    pub const fn variant(self) -> &'static str {
        match self {
            Self::Get => "Get",
            Self::Post => "Post",
            Self::Put => "Put",
            Self::Patch => "Patch",
            Self::Delete => "Delete",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Value<'a>(&'a str);

impl<'a> Value<'a> {
    pub const fn encoded(self) -> &'a str {
        self.0
    }

    pub fn decode(self) -> Result<Cow<'a, str>, InputError> {
        decode(self.0)
    }

    pub fn parse<T>(self) -> Result<T, InputError>
    where
        T: FromStr,
        T::Err: fmt::Display,
    {
        self.decode()?
            .parse()
            .map_err(|error: T::Err| InputError::new(error.to_string()))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputError {
    message: String,
}

impl InputError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for InputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}

impl std::error::Error for InputError {}

#[derive(Clone, Copy, Debug)]
struct CapturedParam<'a> {
    name: &'static str,
    value: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartupError {
    message: String,
}

impl StartupError {
    fn duplicate_state(type_name: &'static str) -> Self {
        Self {
            message: format!("application state `{type_name}` is already registered"),
        }
    }
}

impl fmt::Display for StartupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}

impl std::error::Error for StartupError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateError {
    type_name: &'static str,
}

impl StateError {
    pub const fn type_name(&self) -> &'static str {
        self.type_name
    }
}

impl fmt::Display for StateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "application state `{}` is not registered",
            self.type_name
        )
    }
}

impl std::error::Error for StateError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoError {
    message: &'static str,
}

impl MemoError {
    fn cancelled() -> Self {
        Self {
            message: "request was cancelled",
        }
    }

    fn type_mismatch() -> Self {
        Self {
            message: "request memoization value type did not match its key",
        }
    }
}

impl fmt::Display for MemoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}

impl std::error::Error for MemoError {}

#[derive(Clone, Default)]
pub struct CancellationToken {
    inner: Arc<CancellationState>,
}

#[derive(Default)]
struct CancellationState {
    cancelled: AtomicBool,
    wakers: Mutex<Vec<Waker>>,
}

impl CancellationToken {
    pub fn cancel(&self) {
        if self.inner.cancelled.swap(true, Ordering::AcqRel) {
            return;
        }
        let wakers = {
            let mut wakers = lock(&self.inner.wakers);
            std::mem::take(&mut *wakers)
        };
        for waker in wakers {
            waker.wake();
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.inner.cancelled.load(Ordering::Acquire)
    }

    pub fn check(&self) -> Result<(), MemoError> {
        if self.is_cancelled() {
            Err(MemoError::cancelled())
        } else {
            Ok(())
        }
    }

    pub fn cancelled(&self) -> Cancelled {
        Cancelled {
            token: self.clone(),
        }
    }

    fn register(&self, waker: &Waker) {
        if self.is_cancelled() {
            waker.wake_by_ref();
            return;
        }
        let mut wakers = lock(&self.inner.wakers);
        if !wakers.iter().any(|registered| registered.will_wake(waker)) {
            wakers.push(waker.clone());
        }
    }
}

pub struct Cancelled {
    token: CancellationToken,
}

impl Future for Cancelled {
    type Output = ();

    fn poll(self: Pin<&mut Self>, context: &mut TaskContext<'_>) -> Poll<Self::Output> {
        if self.token.is_cancelled() {
            Poll::Ready(())
        } else {
            self.token.register(context.waker());
            if self.token.is_cancelled() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        }
    }
}

pub struct RequestStarted<'a> {
    pub id: u64,
    pub method: Method,
    pub path: &'a str,
}

pub struct RequestFinished<'a> {
    pub id: u64,
    pub method: Method,
    pub path: &'a str,
    pub status: u16,
    pub elapsed: Duration,
    pub disconnected: bool,
    pub cancelled: bool,
}

pub trait RequestLifecycle: Send + Sync + 'static {
    fn started(&self, _request: &RequestStarted<'_>) {}
    fn finished(&self, _request: &RequestFinished<'_>) {}
}

impl<T> RequestLifecycle for Arc<T>
where
    T: RequestLifecycle,
{
    fn started(&self, request: &RequestStarted<'_>) {
        T::started(self, request);
    }

    fn finished(&self, request: &RequestFinished<'_>) {
        T::finished(self, request);
    }
}

struct StateEntry {
    value: Box<dyn Any + Send + Sync>,
}

pub struct Application {
    routes: &'static [Route],
    clients: &'static [ClientAsset],
    states: HashMap<TypeId, StateEntry>,
    lifecycle: Vec<Arc<dyn RequestLifecycle>>,
    next_request_id: AtomicU64,
}

impl Application {
    pub fn new(routes: &'static [Route], clients: &'static [ClientAsset]) -> Self {
        Self {
            routes,
            clients,
            states: HashMap::new(),
            lifecycle: Vec::new(),
            next_request_id: AtomicU64::new(1),
        }
    }

    pub fn state<T>(&mut self, value: T) -> Result<(), StartupError>
    where
        T: Send + Sync + 'static,
    {
        let id = TypeId::of::<T>();
        if self.states.contains_key(&id) {
            return Err(StartupError::duplicate_state(type_name::<T>()));
        }
        self.states.insert(
            id,
            StateEntry {
                value: Box::new(value),
            },
        );
        Ok(())
    }

    pub fn lifecycle<T>(&mut self, observer: T)
    where
        T: RequestLifecycle,
    {
        self.lifecycle.push(Arc::new(observer));
    }

    fn state_ref<T>(&self) -> Result<&T, StateError>
    where
        T: Send + Sync + 'static,
    {
        self.states
            .get(&TypeId::of::<T>())
            .and_then(|entry| entry.value.downcast_ref())
            .ok_or(StateError {
                type_name: type_name::<T>(),
            })
    }

    fn request_scope(&self) -> Arc<RequestScope> {
        Arc::new(RequestScope {
            id: self.next_request_id.fetch_add(1, Ordering::Relaxed),
            started: Instant::now(),
            cancellation: CancellationToken::default(),
            memo: RequestMemo::default(),
        })
    }

    fn notify_started(&self, id: u64, method: Method, path: &str) {
        let request = RequestStarted { id, method, path };
        for observer in &self.lifecycle {
            observer.started(&request);
        }
    }

    fn notify_finished(
        &self,
        scope: &RequestScope,
        method: Method,
        path: &str,
        status: u16,
        disconnected: bool,
    ) {
        let request = RequestFinished {
            id: scope.id,
            method,
            path,
            status,
            elapsed: scope.started.elapsed(),
            disconnected,
            cancelled: scope.cancellation.is_cancelled(),
        };
        for observer in &self.lifecycle {
            observer.finished(&request);
        }
    }

    pub fn run(self) {
        let application = Arc::new(self);
        let arguments: Vec<String> = std::env::args().collect();
        if arguments.get(1).map(String::as_str) == Some("--render") {
            let target = arguments.get(2).map(String::as_str).unwrap_or("/");
            let method = arguments
                .get(3)
                .and_then(|value| Method::parse(value))
                .unwrap_or(Method::Get);
            let (path, query) = split_target(target);
            if let Some((route, params)) = find_route(application.routes, method, path) {
                let request = application.request_scope();
                application.notify_started(request.id, method, path);
                let context = Context {
                    method,
                    path,
                    query,
                    headers: &[],
                    body: &[],
                    params,
                    application: Arc::clone(&application),
                    request: Arc::clone(&request),
                };
                let mut response = (route.handler)(context);
                prepare_html(&mut response, None, false);
                let status = response.status;
                application.notify_finished(&request, method, path, status, false);
                print!("{}", response.into_body_string());
            } else {
                std::process::exit(2);
            }
            return;
        }

        let port = std::env::var("PORT").unwrap_or_else(|_| "8785".to_owned());
        let listener = TcpListener::bind(format!("127.0.0.1:{port}"))
            .unwrap_or_else(|error| panic!("could not bind server: {error}"));
        println!("antixt listening on http://127.0.0.1:{port}");
        for incoming in listener.incoming() {
            let Ok(stream) = incoming else { continue };
            let application = Arc::clone(&application);
            thread::spawn(move || handle(stream, application));
        }
    }
}

#[derive(Default)]
struct RequestMemo {
    entries: Mutex<Vec<MemoEntry>>,
}

struct MemoEntry {
    key: Box<dyn Any + Send + Sync>,
    key_type: TypeId,
    value_type: TypeId,
    cell: Arc<MemoCell>,
}

impl RequestMemo {
    fn cell<K, T>(&self, key: K) -> (Arc<MemoCell>, bool)
    where
        K: Eq + Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        let key_type = TypeId::of::<K>();
        let value_type = TypeId::of::<T>();
        let mut entries = lock(&self.entries);
        if let Some(entry) = entries.iter().find(|entry| {
            entry.key_type == key_type
                && entry.value_type == value_type
                && entry.key.downcast_ref::<K>() == Some(&key)
        }) {
            return (Arc::clone(&entry.cell), false);
        }
        let cell = Arc::new(MemoCell::new());
        entries.push(MemoEntry {
            key: Box::new(key),
            key_type,
            value_type,
            cell: Arc::clone(&cell),
        });
        (cell, true)
    }
}

struct RequestScope {
    id: u64,
    started: Instant,
    cancellation: CancellationToken,
    memo: RequestMemo,
}

struct MemoCell {
    status: Mutex<MemoStatus>,
    ready: Condvar,
}

enum MemoStatus {
    Computing(Vec<Waker>),
    Ready(Arc<dyn Any + Send + Sync>),
    Cancelled,
}

impl MemoCell {
    fn new() -> Self {
        Self {
            status: Mutex::new(MemoStatus::Computing(Vec::new())),
            ready: Condvar::new(),
        }
    }

    fn complete<T>(&self, value: Arc<T>)
    where
        T: Send + Sync + 'static,
    {
        let wakers = {
            let mut status = lock(&self.status);
            let MemoStatus::Computing(wakers) =
                std::mem::replace(&mut *status, MemoStatus::Ready(value))
            else {
                return;
            };
            wakers
        };
        self.ready.notify_all();
        for waker in wakers {
            waker.wake();
        }
    }

    fn cancel(&self) {
        let wakers = {
            let mut status = lock(&self.status);
            let MemoStatus::Computing(wakers) =
                std::mem::replace(&mut *status, MemoStatus::Cancelled)
            else {
                return;
            };
            wakers
        };
        self.ready.notify_all();
        for waker in wakers {
            waker.wake();
        }
    }

    fn wait_sync<T>(&self, cancellation: &CancellationToken) -> Result<Arc<T>, MemoError>
    where
        T: Send + Sync + 'static,
    {
        let mut status = lock(&self.status);
        loop {
            cancellation.check()?;
            match &*status {
                MemoStatus::Ready(value) => {
                    return Arc::clone(value)
                        .downcast::<T>()
                        .map_err(|_| MemoError::type_mismatch());
                }
                MemoStatus::Cancelled => return Err(MemoError::cancelled()),
                MemoStatus::Computing(_) => {
                    let waited = self.ready.wait_timeout(status, Duration::from_millis(10));
                    status = match waited {
                        Ok((status, _)) => status,
                        Err(poisoned) => poisoned.into_inner().0,
                    };
                }
            }
        }
    }
}

struct MemoProducer {
    cell: Arc<MemoCell>,
    complete: bool,
}

impl Drop for MemoProducer {
    fn drop(&mut self) {
        if !self.complete {
            self.cell.cancel();
        }
    }
}

struct MemoWait<T> {
    cell: Arc<MemoCell>,
    cancellation: CancellationToken,
    marker: std::marker::PhantomData<T>,
}

impl<T> Future for MemoWait<T>
where
    T: Send + Sync + 'static,
{
    type Output = Result<Arc<T>, MemoError>;

    fn poll(self: Pin<&mut Self>, context: &mut TaskContext<'_>) -> Poll<Self::Output> {
        if self.cancellation.is_cancelled() {
            return Poll::Ready(Err(MemoError::cancelled()));
        }
        let mut status = lock(&self.cell.status);
        match &mut *status {
            MemoStatus::Ready(value) => Poll::Ready(
                Arc::clone(value)
                    .downcast::<T>()
                    .map_err(|_| MemoError::type_mismatch()),
            ),
            MemoStatus::Cancelled => Poll::Ready(Err(MemoError::cancelled())),
            MemoStatus::Computing(wakers) => {
                if !wakers
                    .iter()
                    .any(|registered| registered.will_wake(context.waker()))
                {
                    wakers.push(context.waker().clone());
                }
                self.cancellation.register(context.waker());
                Poll::Pending
            }
        }
    }
}

#[derive(Clone)]
pub struct Context<'a> {
    pub method: Method,
    pub path: &'a str,
    query: &'a str,
    headers: &'a [(String, String)],
    body: &'a [u8],
    params: Vec<CapturedParam<'a>>,
    application: Arc<Application>,
    request: Arc<RequestScope>,
}

impl<'a> Context<'a> {
    pub fn param(&self, name: &str) -> Option<Value<'a>> {
        self.params
            .iter()
            .find(|param| param.name == name)
            .map(|param| Value(param.value))
    }

    pub fn query(&self, name: &str) -> Option<Value<'a>> {
        pair_value(self.query, name)
    }

    pub fn form(&self, name: &str) -> Option<Value<'a>> {
        let content_type = self.header("content-type")?;
        if !content_type
            .encoded()
            .starts_with("application/x-www-form-urlencoded")
        {
            return None;
        }
        pair_value(std::str::from_utf8(self.body).ok()?, name)
    }

    pub fn header(&self, name: &str) -> Option<Value<'a>> {
        self.headers
            .iter()
            .find(|(header, _)| header.eq_ignore_ascii_case(name))
            .map(|(_, value)| Value(value))
    }

    pub fn cookie(&self, name: &str) -> Option<Value<'a>> {
        let cookies = self.header("cookie")?.encoded();
        cookies.split(';').find_map(|cookie| {
            let (key, value) = cookie.trim().split_once('=')?;
            (key == name).then_some(Value(value))
        })
    }

    pub const fn body(&self) -> &'a [u8] {
        self.body
    }

    pub fn is_fragment(&self) -> bool {
        self.header("antixt-fragment")
            .is_some_and(|value| value.encoded().eq_ignore_ascii_case("true"))
    }

    pub fn request_id(&self) -> u64 {
        self.request.id
    }

    pub fn elapsed(&self) -> Duration {
        self.request.started.elapsed()
    }

    pub fn cancellation(&self) -> CancellationToken {
        self.request.cancellation.clone()
    }

    pub fn is_cancelled(&self) -> bool {
        self.request.cancellation.is_cancelled()
    }

    pub fn state<T>(&self) -> Result<&T, StateError>
    where
        T: Send + Sync + 'static,
    {
        self.application.state_ref::<T>()
    }

    pub fn memoize_sync<K, T, F>(&self, key: K, produce: F) -> Result<Arc<T>, MemoError>
    where
        K: Eq + Send + Sync + 'static,
        T: Send + Sync + 'static,
        F: FnOnce() -> T,
    {
        self.request.cancellation.check()?;
        let (cell, producer) = self.request.memo.cell::<K, T>(key);
        if !producer {
            return cell.wait_sync(&self.request.cancellation);
        }
        let mut producer = MemoProducer {
            cell: Arc::clone(&cell),
            complete: false,
        };
        let value = Arc::new(produce());
        self.request.cancellation.check()?;
        cell.complete(Arc::clone(&value));
        producer.complete = true;
        Ok(value)
    }

    pub async fn memoize<K, T, F, Fut>(&self, key: K, produce: F) -> Result<Arc<T>, MemoError>
    where
        K: Eq + Send + Sync + 'static,
        T: Send + Sync + 'static,
        F: FnOnce() -> Fut,
        Fut: Future<Output = T> + Send,
    {
        self.request.cancellation.check()?;
        let (cell, producer) = self.request.memo.cell::<K, T>(key);
        if !producer {
            return MemoWait {
                cell,
                cancellation: self.request.cancellation.clone(),
                marker: std::marker::PhantomData,
            }
            .await;
        }
        let mut producer = MemoProducer {
            cell: Arc::clone(&cell),
            complete: false,
        };
        let value = Arc::new(produce().await);
        self.request.cancellation.check()?;
        cell.complete(Arc::clone(&value));
        producer.complete = true;
        Ok(value)
    }
}

pub enum Body {
    Full(String),
    Stream(Box<dyn Iterator<Item = String> + Send>),
}

pub struct Response {
    pub status: u16,
    pub content_type: &'static str,
    pub headers: Vec<(String, String)>,
    body: Body,
}

impl Response {
    pub fn html(body: Html) -> Self {
        Self::full(200, "text/html; charset=utf-8", body.render())
    }

    pub fn text(body: impl Into<String>) -> Self {
        Self::full(200, "text/plain; charset=utf-8", body.into())
    }

    pub fn full(status: u16, content_type: &'static str, body: impl Into<String>) -> Self {
        Self {
            status,
            content_type,
            headers: Vec::new(),
            body: Body::Full(body.into()),
        }
    }

    pub fn stream<I, S>(content_type: &'static str, chunks: I) -> Self
    where
        I: IntoIterator<Item = S>,
        I::IntoIter: Send + 'static,
        S: Into<String> + 'static,
    {
        Self {
            status: 200,
            content_type,
            headers: Vec::new(),
            body: Body::Stream(Box::new(chunks.into_iter().map(Into::into))),
        }
    }

    pub fn redirect(location: impl Into<String>) -> Self {
        Self::full(303, "text/plain; charset=utf-8", "").header("Location", location)
    }

    pub fn with_status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }

    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let name = name.into();
        let value = value.into();
        assert!(
            !name.contains(['\r', '\n']) && !value.contains(['\r', '\n']),
            "response headers cannot contain newlines"
        );
        self.headers.push((name, value));
        self
    }

    pub fn into_body_string(self) -> String {
        match self.body {
            Body::Full(body) => body,
            Body::Stream(chunks) => chunks.collect(),
        }
    }
}

pub trait IntoResponse {
    fn into_response(self) -> Response;
}

impl IntoResponse for Html {
    fn into_response(self) -> Response {
        Response::html(self)
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

pub struct AsyncResponse<'a>(Pin<Box<dyn Future<Output = Response> + Send + 'a>>);

pub fn async_response<'a, F, T>(future: F) -> AsyncResponse<'a>
where
    F: Future<Output = T> + Send + 'a,
    T: IntoResponse + 'a,
{
    AsyncResponse(Box::pin(async move { future.await.into_response() }))
}

impl IntoResponse for AsyncResponse<'_> {
    fn into_response(self) -> Response {
        block_on(self.0)
    }
}

pub struct Sleep {
    state: Arc<Mutex<SleepState>>,
}

struct SleepState {
    complete: bool,
    waker: Option<Waker>,
}

pub fn sleep(duration: Duration) -> Sleep {
    let state = Arc::new(Mutex::new(SleepState {
        complete: false,
        waker: None,
    }));
    let worker_state = Arc::clone(&state);
    thread::spawn(move || {
        thread::sleep(duration);
        let mut state = worker_state.lock().expect("antixt sleep state poisoned");
        state.complete = true;
        if let Some(waker) = state.waker.take() {
            waker.wake();
        }
    });
    Sleep { state }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, context: &mut TaskContext<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock().expect("antixt sleep state poisoned");
        if state.complete {
            Poll::Ready(())
        } else {
            state.waker = Some(context.waker().clone());
            Poll::Pending
        }
    }
}

struct ThreadWaker(thread::Thread);

impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}

fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = std::pin::pin!(future);
    let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
    let mut context = TaskContext::from_waker(&waker);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => thread::park(),
        }
    }
}

pub type Handler = for<'a> fn(Context<'a>) -> Response;

#[derive(Clone, Copy)]
pub struct Route {
    pub method: Method,
    pub pattern: &'static str,
    pub handler: Handler,
}

impl Route {
    pub const fn new(method: Method, pattern: &'static str, handler: Handler) -> Self {
        Self {
            method,
            pattern,
            handler,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ClientAsset {
    pub name: &'static str,
    pub source: &'static str,
}

impl ClientAsset {
    pub const fn new(name: &'static str, source: &'static str) -> Self {
        Self { name, source }
    }
}

pub fn run(routes: &'static [Route], clients: &'static [ClientAsset]) {
    Application::new(routes, clients).run();
}

fn find_route<'a>(
    routes: &'static [Route],
    method: Method,
    path: &'a str,
) -> Option<(&'static Route, Vec<CapturedParam<'a>>)> {
    routes.iter().find_map(|route| {
        if route.method != method {
            return None;
        }
        match_pattern(route.pattern, path).map(|params| (route, params))
    })
}

fn match_pattern<'a>(pattern: &'static str, path: &'a str) -> Option<Vec<CapturedParam<'a>>> {
    let mut pattern = pattern.trim_start_matches('/');
    let mut path = path.trim_start_matches('/');
    let mut params = Vec::new();
    while !pattern.is_empty() {
        let segment = take_segment(&mut pattern)?;
        if let Some(name) = segment.strip_prefix('*') {
            if path.is_empty() {
                return None;
            }
            params.push(CapturedParam { name, value: path });
            path = "";
            break;
        }
        let value = take_segment(&mut path)?;
        if let Some(name) = segment.strip_prefix(':') {
            params.push(CapturedParam { name, value });
        } else if segment != value {
            return None;
        }
    }
    path.is_empty().then_some(params)
}

fn take_segment<'a>(remaining: &mut &'a str) -> Option<&'a str> {
    if remaining.is_empty() {
        return None;
    }
    if let Some((segment, rest)) = remaining.split_once('/') {
        *remaining = rest;
        Some(segment)
    } else {
        let segment = *remaining;
        *remaining = "";
        Some(segment)
    }
}

struct ParsedRequest {
    method: Option<Method>,
    target: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

fn read_request(stream: &mut TcpStream) -> Result<ParsedRequest, String> {
    let mut bytes = Vec::with_capacity(8192);
    let mut buffer = [0_u8; 8192];
    let header_end = loop {
        let count = stream
            .read(&mut buffer)
            .map_err(|error| format!("could not read request: {error}"))?;
        if count == 0 {
            return Err("connection closed before request headers".to_owned());
        }
        bytes.extend_from_slice(&buffer[..count]);
        if bytes.len() > MAX_REQUEST_BYTES {
            return Err("request exceeds one MiB".to_owned());
        }
        if let Some(position) = find_bytes(&bytes, b"\r\n\r\n") {
            break position + 4;
        }
    };

    let head = std::str::from_utf8(&bytes[..header_end - 4])
        .map_err(|_| "request headers are not UTF-8".to_owned())?;
    let mut lines = head.split("\r\n");
    let mut request_line = lines.next().unwrap_or("").split_whitespace();
    let method = request_line.next().and_then(Method::parse);
    let target = request_line.next().unwrap_or("").to_owned();
    let mut headers = Vec::new();
    let mut content_length = 0_usize;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            return Err("malformed request header".to_owned());
        };
        let value = value.trim().to_owned();
        if name.eq_ignore_ascii_case("content-length") {
            content_length = value
                .parse()
                .map_err(|_| "invalid content-length".to_owned())?;
        }
        headers.push((name.to_owned(), value));
    }
    if header_end + content_length > MAX_REQUEST_BYTES {
        return Err("request exceeds one MiB".to_owned());
    }
    while bytes.len() < header_end + content_length {
        let count = stream
            .read(&mut buffer)
            .map_err(|error| format!("could not read request body: {error}"))?;
        if count == 0 {
            return Err("connection closed before request body".to_owned());
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
    Ok(ParsedRequest {
        method,
        target,
        headers,
        body: bytes[header_end..header_end + content_length].to_vec(),
    })
}

fn handle(mut stream: TcpStream, application: Arc<Application>) {
    let request = match read_request(&mut stream) {
        Ok(request) => request,
        Err(_) => {
            let _ = send(
                &mut stream,
                Response::full(400, "text/plain; charset=utf-8", "Bad request"),
            );
            return;
        }
    };
    let (path, query) = split_target(&request.target);
    let development_version = std::env::var("ANTIXT_DEV_VERSION").ok();

    if request.method == Some(Method::Get) && path == "/__antixt/version" {
        let _ = send(
            &mut stream,
            Response::text(development_version.as_deref().unwrap_or("production")),
        );
        return;
    }
    if request.method == Some(Method::Get)
        && let Some(name) = path
            .strip_prefix("/__antixt/client/")
            .and_then(|path| path.strip_suffix(".js"))
        && let Some(asset) = application.clients.iter().find(|asset| asset.name == name)
    {
        let _ = send(
            &mut stream,
            Response::full(200, "text/javascript; charset=utf-8", asset.source),
        );
        return;
    }

    let Some(method) = request.method else {
        let _ = send(
            &mut stream,
            Response::full(400, "text/plain; charset=utf-8", "Bad request"),
        );
        return;
    };
    let Some((route, params)) = find_route(application.routes, method, path) else {
        let _ = send(
            &mut stream,
            Response::full(404, "text/plain; charset=utf-8", "Not found"),
        );
        return;
    };
    let request_scope = application.request_scope();
    application.notify_started(request_scope.id, method, path);
    let context = Context {
        method,
        path,
        query,
        headers: &request.headers,
        body: &request.body,
        params,
        application: Arc::clone(&application),
        request: Arc::clone(&request_scope),
    };
    let fragment = context.is_fragment();
    let mut response = (route.handler)(context);
    prepare_html(&mut response, development_version.as_deref(), fragment);
    let status = response.status;
    let disconnected = send(&mut stream, response).is_err();
    if disconnected {
        request_scope.cancellation.cancel();
    }
    application.notify_finished(&request_scope, method, path, status, disconnected);
}

fn prepare_html(response: &mut Response, development_version: Option<&str>, fragment: bool) {
    if fragment || !response.content_type.starts_with("text/html") {
        return;
    }
    let Body::Full(document) = &mut response.body else {
        return;
    };
    let enhanced = document.contains("data-antixt-island")
        || document.contains("data-antixt-get")
        || document.contains("data-antixt-post")
        || document.contains("data-antixt-fragment");
    if enhanced && !document.contains("data-antixt-client") {
        let query = development_version
            .map(|version| format!("?v={version}"))
            .unwrap_or_default();
        let client_runtime = CLIENT_RUNTIME.replace("__ANTIXT_CLIENT_QUERY__", &query);
        *document = inject_before_body(document, &client_runtime);
    }
    if development_version.is_some() && !document.contains("data-antixt-dev") {
        *document = inject_before_body(document, RELOAD_CLIENT);
    }
}

fn inject_before_body(document: &str, script: &str) -> String {
    if let Some(position) = document.rfind("</body>") {
        format!(
            "{}{}{}",
            &document[..position],
            script,
            &document[position..]
        )
    } else {
        format!("{document}{script}")
    }
}

fn send(stream: &mut TcpStream, mut response: Response) -> std::io::Result<()> {
    let reason = match response.status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        303 => "See Other",
        400 => "Bad Request",
        404 => "Not Found",
        _ => "Response",
    };
    let mut header = format!(
        "HTTP/1.1 {} {reason}\r\nContent-Type: {}\r\nConnection: close\r\n",
        response.status, response.content_type
    );
    for (name, value) in &response.headers {
        let _ = fmt::Write::write_fmt(&mut header, format_args!("{name}: {value}\r\n"));
    }
    match response.body {
        Body::Full(body) => {
            let _ = fmt::Write::write_fmt(
                &mut header,
                format_args!("Content-Length: {}\r\n\r\n", body.len()),
            );
            stream.write_all(header.as_bytes())?;
            stream.write_all(body.as_bytes())?;
        }
        Body::Stream(ref mut chunks) => {
            header.push_str("Transfer-Encoding: chunked\r\n\r\n");
            stream.write_all(header.as_bytes())?;
            for chunk in chunks {
                let bytes = chunk.as_bytes();
                write!(stream, "{:x}\r\n", bytes.len())?;
                stream.write_all(bytes)?;
                stream.write_all(b"\r\n")?;
                stream.flush()?;
            }
            stream.write_all(b"0\r\n\r\n")?;
        }
    }
    Ok(())
}

fn split_target(target: &str) -> (&str, &str) {
    target.split_once('?').unwrap_or((target, ""))
}

fn pair_value<'a>(pairs: &'a str, wanted: &str) -> Option<Value<'a>> {
    pairs.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let decoded = decode(key).ok()?;
        (decoded == wanted).then_some(Value(value))
    })
}

fn decode(value: &str) -> Result<Cow<'_, str>, InputError> {
    if !value
        .as_bytes()
        .iter()
        .any(|byte| *byte == b'%' || *byte == b'+')
    {
        return Ok(Cow::Borrowed(value));
    }
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => output.push(b' '),
            b'%' if index + 2 < bytes.len() => {
                let high = hex(bytes[index + 1])?;
                let low = hex(bytes[index + 2])?;
                output.push(high * 16 + low);
                index += 2;
            }
            b'%' => return Err(InputError::new("incomplete percent escape")),
            byte => output.push(byte),
        }
        index += 1;
    }
    String::from_utf8(output)
        .map(Cow::Owned)
        .map_err(|_| InputError::new("decoded value is not UTF-8"))
}

fn hex(byte: u8) -> Result<u8, InputError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(InputError::new("invalid percent escape")),
    }
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html;
    use std::sync::Barrier;
    use std::time::Instant;

    static NO_ROUTES: &[Route] = &[];
    static NO_CLIENTS: &[ClientAsset] = &[];

    fn test_context(application: Arc<Application>) -> Context<'static> {
        let request = application.request_scope();
        Context {
            method: Method::Get,
            path: "/test",
            query: "",
            headers: &[],
            body: &[],
            params: Vec::new(),
            application,
            request,
        }
    }

    #[test]
    fn converts_html_into_a_response() {
        let response = html::h1().text("Hello").into_response();
        assert_eq!(response.into_body_string(), "<h1>Hello</h1>");
    }

    #[test]
    fn matches_dynamic_and_catch_all_routes() {
        let dynamic = match_pattern("/blog/:slug", "/blog/hello").unwrap();
        assert_eq!(dynamic[0].name, "slug");
        assert_eq!(dynamic[0].value, "hello");
        let catch_all = match_pattern("/docs/*path", "/docs/rust/start").unwrap();
        assert_eq!(catch_all[0].value, "rust/start");
        assert!(match_pattern("/blog/:slug", "/blog").is_none());
    }

    #[test]
    fn decodes_and_parses_typed_values() {
        let value = pair_value("name=antixt+Rust&count=42", "name").unwrap();
        assert_eq!(value.decode().unwrap(), "antixt Rust");
        let count: u32 = pair_value("count=42", "count").unwrap().parse().unwrap();
        assert_eq!(count, 42);
    }

    #[test]
    fn reads_typed_request_surfaces_and_builds_redirects() {
        let headers = vec![
            (
                "Content-Type".to_owned(),
                "application/x-www-form-urlencoded".to_owned(),
            ),
            ("Cookie".to_owned(), "session=abc123; theme=dark".to_owned()),
            ("X-Request-Id".to_owned(), "request-7".to_owned()),
        ];
        let application = Arc::new(Application::new(NO_ROUTES, NO_CLIENTS));
        let request = application.request_scope();
        let context = Context {
            method: Method::Post,
            path: "/submit",
            query: "page=3",
            headers: &headers,
            body: b"email=hello%40example.com",
            params: Vec::new(),
            application,
            request,
        };
        assert_eq!(context.query("page").unwrap().parse::<u32>().unwrap(), 3);
        assert_eq!(
            context.form("email").unwrap().decode().unwrap(),
            "hello@example.com"
        );
        assert_eq!(context.cookie("theme").unwrap().encoded(), "dark");
        assert_eq!(
            context.header("x-request-id").unwrap().encoded(),
            "request-7"
        );

        let response = Response::redirect("/complete");
        assert_eq!(response.status, 303);
        assert_eq!(
            response.headers,
            [("Location".to_owned(), "/complete".to_owned())]
        );
    }

    #[test]
    fn injects_opt_in_clients_before_body_end() {
        let mut response = Response::full(
            200,
            "text/html; charset=utf-8",
            "<html><body><div data-antixt-island=\"counter\"></div></body></html>",
        );
        prepare_html(&mut response, Some("test"), false);
        let output = response.into_body_string();
        assert!(output.contains("data-antixt-client"));
        assert!(output.contains("data-antixt-dev"));
        assert!(output.contains("const clientVersion = '?v=test'"));
        assert!(output.find("data-antixt-client").unwrap() < output.find("</body>").unwrap());
    }

    #[test]
    fn resolves_async_responses_without_an_external_runtime() {
        let started = Instant::now();
        let response = async_response(async {
            sleep(Duration::from_millis(2)).await;
            html::p().text("done")
        })
        .into_response();
        assert_eq!(response.into_body_string(), "<p>done</p>");
        assert!(started.elapsed() >= Duration::from_millis(2));
    }

    #[test]
    fn collects_streamed_response_for_static_rendering() {
        let response = Response::stream("text/html; charset=utf-8", ["one", "two"]);
        assert_eq!(response.into_body_string(), "onetwo");
    }

    #[test]
    fn registers_typed_state_and_reports_configuration_errors() {
        struct SiteName(&'static str);
        #[derive(Debug)]
        struct Missing;

        let mut application = Application::new(NO_ROUTES, NO_CLIENTS);
        application.state(SiteName("antixt")).unwrap();
        let duplicate = application.state(SiteName("duplicate")).unwrap_err();
        assert!(duplicate.to_string().contains("already registered"));

        let context = test_context(Arc::new(application));
        assert_eq!(context.state::<SiteName>().unwrap().0, "antixt");
        let missing = context.state::<Missing>().unwrap_err();
        assert!(missing.to_string().contains(type_name::<Missing>()));
    }

    #[test]
    fn memoizes_sync_values_once_per_request() {
        let context = test_context(Arc::new(Application::new(NO_ROUTES, NO_CLIENTS)));
        let calls = AtomicU64::new(0);
        let first = context
            .memoize_sync("navigation", || {
                calls.fetch_add(1, Ordering::Relaxed);
                String::from("cached")
            })
            .unwrap();
        let second = context
            .memoize_sync("navigation", || {
                calls.fetch_add(1, Ordering::Relaxed);
                String::from("duplicate")
            })
            .unwrap();
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        assert_eq!(first.as_str(), "cached");
        assert!(Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn deduplicates_concurrent_async_memoization() {
        let context = test_context(Arc::new(Application::new(NO_ROUTES, NO_CLIENTS)));
        let calls = Arc::new(AtomicU64::new(0));
        let barrier = Arc::new(Barrier::new(3));
        let mut workers = Vec::new();
        for _ in 0..2 {
            let context = context.clone();
            let calls = Arc::clone(&calls);
            let barrier = Arc::clone(&barrier);
            workers.push(thread::spawn(move || {
                barrier.wait();
                block_on(context.memoize("catalog", || async move {
                    calls.fetch_add(1, Ordering::Relaxed);
                    sleep(Duration::from_millis(5)).await;
                    String::from("shared")
                }))
                .unwrap()
            }));
        }
        barrier.wait();
        let values: Vec<_> = workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect();
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        assert!(Arc::ptr_eq(&values[0], &values[1]));
    }

    #[test]
    fn cancellation_wakes_waiting_futures() {
        let token = CancellationToken::default();
        let waiting = token.clone();
        let worker = thread::spawn(move || block_on(waiting.cancelled()));
        thread::sleep(Duration::from_millis(2));
        token.cancel();
        worker.join().unwrap();
        assert!(token.is_cancelled());
        assert_eq!(
            token.check().unwrap_err().to_string(),
            "request was cancelled"
        );
    }

    #[test]
    fn cancellation_releases_async_memo_waiters() {
        let context = test_context(Arc::new(Application::new(NO_ROUTES, NO_CLIENTS)));
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let producer_context = context.clone();
        let producer = thread::spawn(move || {
            block_on(producer_context.memoize("slow", || async move {
                started_tx.send(()).unwrap();
                sleep(Duration::from_millis(20)).await;
                String::from("finished")
            }))
        });
        started_rx.recv().unwrap();
        let waiting_context = context.clone();
        let waiter = thread::spawn(move || {
            block_on(
                waiting_context.memoize("slow", || async { String::from("unexpected producer") }),
            )
        });
        thread::sleep(Duration::from_millis(2));
        context.cancellation().cancel();
        assert_eq!(
            waiter.join().unwrap().unwrap_err().to_string(),
            "request was cancelled"
        );
        assert_eq!(
            producer.join().unwrap().unwrap_err().to_string(),
            "request was cancelled"
        );
    }

    #[test]
    fn lifecycle_observers_receive_request_outcomes() {
        #[derive(Default)]
        struct Observer {
            events: Mutex<Vec<String>>,
        }

        impl RequestLifecycle for Observer {
            fn started(&self, request: &RequestStarted<'_>) {
                lock(&self.events).push(format!("start:{}:{}", request.id, request.path));
            }

            fn finished(&self, request: &RequestFinished<'_>) {
                lock(&self.events).push(format!(
                    "finish:{}:{}:{}",
                    request.id, request.status, request.disconnected
                ));
            }
        }

        let observer = Arc::new(Observer::default());
        let mut application = Application::new(NO_ROUTES, NO_CLIENTS);
        application.lifecycle(Arc::clone(&observer));
        let scope = application.request_scope();
        application.notify_started(scope.id, Method::Get, "/health");
        application.notify_finished(&scope, Method::Get, "/health", 204, false);
        assert_eq!(
            *lock(&observer.events),
            ["start:1:/health", "finish:1:204:false"]
        );
    }
}
