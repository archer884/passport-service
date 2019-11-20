mod passport;
mod reader;
mod weights;

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::fmt::{self, Display};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

macro_rules! time {
    ($x:expr) => {{
        let mut timer = stopwatch::Stopwatch::start_new();
        let evaluated_expression = $x;
        timer.stop();
        (timer.elapsed(), evaluated_expression)
    }};
}

#[derive(Clone, Debug, StructOpt)]
struct Opt {
    path: String,
}

#[derive(Copy, Clone, Debug)]
enum ApplicationError {
    /// Requested passport file not found
    NotFound,
    /// Unable to parse text as passport mrz
    Unreadable,
}

impl Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApplicationError::NotFound => f.write_str("Not found"),
            ApplicationError::Unreadable => f.write_str("Unreadable"),
        }
    }
}

impl actix_web::error::ResponseError for ApplicationError {
    fn error_response(&self) -> HttpResponse {
        use actix_web::http;
        match self {
            ApplicationError::NotFound => HttpResponse::new(http::StatusCode::NOT_FOUND),
            ApplicationError::Unreadable => HttpResponse::new(http::StatusCode::BAD_REQUEST),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use actix_web::{App, HttpServer};

    HttpServer::new(|| App::new().route("/api/passport/{file}", web::get().to(read_mrz)))
        .bind("127.0.0.1:8000")
        .expect("Unable to bind server port")
        .run()?;

    Ok(())
}

fn read_mrz(request: HttpRequest) -> actix_web::Result<impl Responder> {
    use passport::{PassportData, PassportInfo};
    use reader::Reader;

    let file = request
        .match_info()
        .get("file")
        .expect("How the hell did you get routed here?!");

    let (elapsed, raw) = time!({
        let mut reader = Reader::new();
        reader.read(build_path(&file))
    });
    println!("ttr: {:?}", elapsed);

    let raw = raw.ok_or(ApplicationError::NotFound)?;
    let extracted = PassportData::try_from_str(raw).ok_or(ApplicationError::Unreadable)?;

    Ok(web::Json(PassportInfo::from_data(extracted)))
}

fn build_path(file: &str) -> PathBuf {
    Path::new("./resource").join(file)
}
