mod passport;
mod weights;

use tide::{App, Context, EndpointResult, error::{Error as TideError, ResultExt}};
use leptess::tesseract::TessApi;
use std::path::Path;
use structopt::StructOpt;
use passport::{PassportData, PassportInfo};
use std::io;

#[derive(Clone, Debug, StructOpt)]
struct Opt {
    path: String,
}

struct Reader {
    api: TessApi,
}

impl Reader {
    fn new() -> Self {
        Self {
            api: TessApi::new(Some("./resource"), "mrz").unwrap(),
        }
    }

    fn read(&mut self, path: impl AsRef<Path>) -> io::Result<String> {
        use leptess::leptonica;
        let pixels = leptonica::pix_read(path.as_ref())
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Passport file not found"))?;
        
        self.api.set_image(&pixels);
        self.api.get_utf8_text()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}

fn main() {
    let mut app = App::new();
    app.at("/api/passport/:file").get(read_mrz);
    app.serve("127.0.0.1:8000").expect("fml");

    
    let serialized = serde_json::to_string_pretty(&value).unwrap();

    println!("{}", serialized);
}

async fn read_mrz(cx: Context<()>) -> EndpointResult<String> {    
    let file: String = cx.param("file").client_err()?;
    let mut reader = Reader::new();
    let raw_data = reader.read(path)
        .map_err(|e| TideError::
            
            io::Error::new(io::ErrorKind::Other, e))?;

    // Ok(PassportData::try_from_str(raw_data).ok_or_else(|| ))
    // let value = PassportInfo::from_data(PassportData::try_from_str(raw_data).expect("wtf?"));
    // println!("{}", file);
    Ok(file)
}

#[cfg(test)]
mod tests {
    #[test]
    fn check() {
        assert!(super::check("1204159", super::numeric).is_valid());
        assert!(super::check("7408122", super::numeric).is_valid());
        assert!(super::check("ZE184226B<<<<<1", super::alphanumeric).is_valid());
        assert!(super::check("<<<<<<<<<<<<<<<", super::alphanumeric).is_valid());
    }
}
