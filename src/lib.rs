extern crate curl;
extern crate url;
extern crate serde;
#[macro_use(Deserialize,Serialize)] extern crate serde_derive;
extern crate serde_json;

use curl::easy::Easy;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str;
pub use url::Url;



pub type DlResult<T> = Result<T, DlErr>;

#[derive(Debug)]
pub enum DlErr {
    CurlErr(curl::Error),
    IoErr(io::Error),
    JsonErr(serde_json::Error),

    DowncastFailure { msg: String }
}

impl From<curl::Error> for DlErr {
    fn from(err: curl::Error) -> DlErr {  DlErr::CurlErr(err)  }
}

impl From<io::Error> for DlErr {
    fn from(err: io::Error) -> DlErr {  DlErr::IoErr(err)  }
}

impl From<serde_json::Error> for DlErr {
    fn from(err: serde_json::Error) -> DlErr {  DlErr::JsonErr(err)  }
}



#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DlStatus {
    Downloaded{ location: PathBuf, num_bytes: usize },
    FileExists(PathBuf),
    Replaced{ location: PathBuf, num_bytes: usize },
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dl {
    buffer: Vec<u8>,
    overwrite: bool,
    verbose: bool,
}

impl Dl {
    pub fn new() -> Self {
        Dl {
            buffer: vec![],
            overwrite: false,
            verbose: false,
        }
    }

    pub fn overwrite(mut self, overwrite_file: bool) -> Self {
        self.overwrite = overwrite_file;
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn download(&mut self, from: &Url, to: &Path) -> DlResult<DlStatus> {
        self.buffer.clear();
        let to = PathBuf::from(to);
        let headers = [
            // Authorization is handled by the `self.get()` call below,
            //    and there must not be an "Authorization" header here.
            format!("Accept: application/octet-stream"),
            format!("User-Agent: download"),
        ];

        if to.exists() && self.overwrite {
            self.get(from, &headers)?;
            let num_bytes = File::create(&to)?.write(&self.buffer)?;
            return Ok(DlStatus::Replaced{ location: to, num_bytes: num_bytes });
        } else if to.exists() && !self.overwrite {
            // TODO: Inform the user that the download is
            //    skipped because the file already exists.
            return Ok(DlStatus::FileExists(to));
        } else { // `to` does not exist
            self.get(from, &headers)?;
            let num_bytes = File::create(&to)?.write(&self.buffer)?;
            Ok(DlStatus::Downloaded{ location: to, num_bytes: num_bytes })
        }
    }


    fn get(&mut self, url: &Url, headers: &[String]) -> DlResult<()> {
        println!("[GET] url: {}", url);
        let mut easy: Easy = self.configure_easy(url.as_str(), headers)?;
        easy.get(true)?;
        let mut transfer = easy.transfer();
        transfer.write_function(|data: &[u8]| {
            self.buffer.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform().map_err(DlErr::CurlErr)
    }


    fn configure_easy(&mut self, url: &str, headers: &[String])
                      -> DlResult<Easy> {
        let mut easy = Easy::new();
        easy.url(url)?;
        easy.follow_location(true)?;
        easy.autoreferer(false)?;
        easy.verbose(self.verbose)?;
        // easy.username(&self.owner)?;     // TODO:
        // easy.password(&self.api_token)?; // TODO:
        easy.progress(true)?;

        let mut header_list = curl::easy::List::new();
        for header in headers {
            header_list.append(header)?;
        }
        easy.http_headers(header_list)?;
        Ok(easy)
    }


    // pub fn download(&mut self, release_tag: &str, asset: String) -> DlResult<()> {
    //     // GET /repos/:owner/:repo/releases/assets/:id
    //     self.buffer.clear();
    //     let found_assets: JsonValue = self.list_assets(release_tag)?;
    //     let value: JsonValue = serde_json::from_slice(&self.buffer)?;
    //     let asset_path = Path::new(&asset);
    //     if asset_path.exists() {  return Ok(()); /* Skip the download */  }
    //     let mut file = File::create(&asset_path)?;
    //     let downcast_fail = || DlErr::DowncastFailure {
    //         msg: String::from("asset_obj[\"id\"].as_u64()")
    //     };
    //     let mut asset_id: u64 = 0;
    //     match found_assets {
    //         JsonValue::Array(ref vec) => {
    //             for asset_obj in vec.iter().filter(|obj| obj["name"] == asset) {
    //                 asset_id = asset_obj["id"].as_u64().ok_or_else(downcast_fail)?;
    //                 break;
    //             }
    //         },
    //         _ => unimplemented!(),
    //     }
    //     let url = format!(
    //         "https://api.github.com/repos/{}/{}/releases/assets/{}",
    //         self.owner,
    //         self.repo,
    //         asset_id
    //     );
    //     let headers = [
    //         // Authorization is handled by the `self.get()` call below,
    //         //    and there must not be an "Authorization" header here.
    //         format!("Accept: application/octet-stream"),
    //         format!("User-Agent: ghdeploy"),
    //     ];
    //     self.buffer.clear();
    //     self.get(&url, &headers)?;
    //     let num_bytes = file.write(&self.buffer)?;
    //     println!("Wrote asset @ {}    ({} bytes)", asset, num_bytes);
    //     Ok(())
    // }


    // fn post(&mut self, url: &str, headers: &[String], file: Option<&File>)
    //         -> DlResult<()> {
    //     println!("[POST] url: {}", url);
    //     let mut easy: Easy = self.configure_easy(url, headers)?;
    //     easy.upload(true)?;
    //     easy.post(true)?;

    //     easy.perform()?;
    //     let mut easy: Easy = self.configure_easy(url, headers)?;

    //     let (bytes, num_bytes) = match file {
    //         None => (vec![], 0),
    //         Some(mut file) => {
    //             let mut bytes = vec![];
    //             file.read_to_end(&mut bytes)?;
    //             let len = bytes.len();
    //             (bytes, len)
    //         },
    //     };

    //     easy.in_filesize(num_bytes as u64)?;
    //     easy.post_field_size(num_bytes as u64)?;
    //     easy.post_fields_copy(&bytes)?;
    //     easy.perform().map_err(DlErr::CurlErr)
    // }


}












#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
