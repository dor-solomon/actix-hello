use actix_web::{dev::ServerHandle, get, post, web, App, HttpResponse, HttpServer, Responder};
use parking_lot::Mutex;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[get("/stop_server")]
async fn stop(stop_handle: web::Data<StopHandle>) -> HttpResponse {
    stop_handle.stop();
    HttpResponse::NoContent().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // Create the stop handle container
    let stop_handle = web::Data::new(StopHandle::default());

    let srv = HttpServer::new({
        let stop_handle = stop_handle.clone();    

        move || {
            App::new()
                .app_data(stop_handle.clone())
                .service(hello)
                .service(echo)
                .route("/hey", web::get().to(manual_hello))
                .service(stop)
        }
    })
    .bind(("127.0.0.1", 8080))?
    .run();
    // Register the server handle with the stop handle
    stop_handle.register(srv.handle());

    srv.await
}

#[derive(Default)]
struct StopHandle {
    inner: Mutex<Option<ServerHandle>>,
}

impl StopHandle {
    // Sets the server handle to stop
    pub(crate) fn register(&self, handle: ServerHandle) {
        *self.inner.lock() = Some(handle);
    }

    // Sends stop signal through contained server handle
    pub(crate) fn stop(&self) {
        #[allow(clippy::let_underscore_future)]
        let _ = self.inner.lock().as_ref().unwrap().stop(true);
    }
}