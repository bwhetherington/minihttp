use super::HttpError;
use super::Url;
use super::TlsConnector;
use super::Response;

use std::ascii::AsciiExt;
use std::collections::HashMap;
use std::io::prelude::*;
use std::net::TcpStream;
use std::time;

///proxy info object.
#[derive(Debug, Clone)]
pub struct Proxy{
    host: String,
    port: u16,
    scheme: String,
    url: Url,
}

///http request object.
#[derive(Debug, Clone)]
pub struct Request<'a>{
    host: String,
    port: u16,
    scheme: String,
    method: String,
    url: Url,
    headers: HashMap<&'a str,&'a str>,
    body: Option<Vec<u8>>,
    timeout:u64,
    proxy:Option<Proxy>,
    verify:bool
}

impl<'b> Request<'b>{

    ///return a Request object
    /// # Example
    /// ```
    /// use minihttp::request::Request;
    ///
    /// let mut http = Request::new("https://www.google.com").unwrap();
    /// ```
    pub fn new(url:&str)->Result<Request,HttpError>{
        let url:Url = Url::parse(url);

        let host  = match url.host{
            Some(ref h) => h.clone(),
            None =>return Err(HttpError::Parse("url parse error"))
        };
        Ok(
            Request{
                host:host,
                port:url.port,
                scheme:url.scheme.clone(),
                method:String::new(),
                url:url,
                headers:HashMap::new(),
                body:None,
                timeout:30,
                proxy:None,
                verify:true
            }
        )
    }

    ///set Request GET method
    /// # Example
    /// ```
    /// use minihttp::request::Request;
    ///
    /// let mut http = Request::new("https://www.google.com").unwrap();
    /// http.get();
    /// ```
    pub fn get(&mut self) -> &mut Self {
        self.method = "GET".to_owned();
        self
    }

    ///set Request POST method
    /// # Example
    /// ```
    /// use minihttp::request::Request;
    ///
    /// let mut http = Request::new("https://www.google.com").unwrap();
    /// http.post();
    /// ```
    pub fn post(&mut self) -> &mut Self {
        self.method = "POST".to_owned();
        self
    }

    ///set Request PUT method
    /// # Example
    /// ```
    /// use minihttp::request::Request;
    ///
    /// let mut http = Request::new("https://www.google.com").unwrap();
    /// http.put();
    /// ```
    pub fn put(&mut self) -> &mut Self {
        self.method = "PUT".to_owned();
        self
    }

    ///set Request HEAD method
    /// # Example
    /// ```
    /// use minihttp::request::Request;
    ///
    /// let mut http = Request::new("https://www.google.com").unwrap();
    /// http.head();
    /// ```
    pub fn head(&mut self) -> &mut Self {
        self.method = "HEAD".to_owned();
        self
    }

    ///set Request DELETE method
    /// # Example
    /// ```
    /// use minihttp::request::Request;
    ///
    /// let mut http = Request::new("https://www.google.com").unwrap();
    /// http.delete();
    /// ```
    pub fn delete(&mut self) -> &mut Self {
        self.method = "DELETE".to_owned();
        self
    }

    ///set Request OPTION method
    /// # Example
    /// ```
    /// use minihttp::request::Request;
    ///
    /// let mut http = Request::new("https://www.google.com").unwrap();
    /// http.options();
    /// ```
    pub fn options(&mut self) -> &mut Self {
        self.method = "OPTIONS".to_owned();
        self
    }

    ///set Request custom method
    /// # Example
    /// ```
    /// use minihttp::request::Request;
    ///
    /// let mut http = Request::new("https://www.google.com").unwrap();
    /// http.request("profile");
    /// ```
    pub fn request(&mut self, method: &str) -> &mut Self {
        self.method = method.to_string();
        self
    }

    ///set Request custom header
    /// # Example
    /// ```
    /// use minihttp::request::Request;
    /// use std::collections::HashMap;
    ///
    /// let mut http = Request::new("https://www.google.com").unwrap();
    /// let mut headers = HashMap::new();
    /// headers.insert("User-Agent","Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36");
    /// http.headers(headers);
    /// ```
    pub fn headers<'a:'b>(&mut self, data: HashMap<&'a str, &'a str>) -> &mut Self {
        self.headers = data;
        self
    }

    ///set Request body
    /// # Example
    /// ```
    /// use minihttp::request::Request;
    ///
    /// let mut http = Request::new("https://www.google.com").unwrap();
    /// let body = vec![0,1,2,3,4];
    /// http.body(body);
    /// ```
    pub fn body(&mut self, data:Vec<u8>) -> &mut Self {
        self.body = Some(data);
        self
    }

    ///set Request string body
    /// # Example
    /// ```
    /// use minihttp::request::Request;
    ///
    /// let mut http = Request::new("https://www.google.com").unwrap();
    /// let body = "hello";
    /// http.body_str(body);
    /// ```
    pub fn body_str(&mut self,data:&str) ->&mut Self{
        let body = data.as_bytes().to_owned();
        self.body = Some(body);
        self
    }

    ///set Request read/write timeout(sec)
    /// # Example
    /// ```
    /// use minihttp::request::Request;
    ///
    /// let mut http = Request::new("https://www.google.com").unwrap();
    /// http.timeout(10);
    /// ```
    pub fn timeout(&mut self,time:u64) -> &mut Self{
        self.timeout = time;
        self
    }

    ///set https request  if verify the certificate(default true)
    /// # Example
    /// ```
    /// use minihttp::request::Request;
    ///
    /// let mut http = Request::new("https://www.google.com").unwrap();
    /// http.verify(false);
    /// ```
    pub fn verify(&mut self,verify:bool) -> Result<&mut Self,HttpError>{
        if self.scheme == "https".to_owned(){
            self.verify = verify;
        }else {
            return Err(HttpError::Config("Verify setting only for https"))
        }
        Ok(self)
    }

    ///set proxy info
    /// # Example
    /// ```
    /// use minihttp::request::Request;
    ///
    /// let mut http = Request::new("https://www.google.com").unwrap();
    /// http.proxy("https://127.0.0.1:1080");
    /// ```
    pub fn proxy(&mut self,proxy:&str) -> Result<&mut Self,HttpError>{
        let url:Url = Url::parse(proxy);
        if self.scheme == "https".to_owned() && url.scheme == "http".to_owned(){
            return  Err(HttpError::Proxy("Http proxy can only use http scheme."))
        }

        let host  = match url.host{
            Some(ref h) => h.clone(),
            None =>return Err(HttpError::Parse("url parse error"))
        };

        let proxy = Proxy{
            host:host,
            port:url.port,
            scheme:url.scheme.clone(),
            url:url
        };
        self.proxy = Some(proxy);
        Ok(self)
    }

    ///send https request
    /// # Example
    /// ```
    /// use minihttp::request::Request;
    ///
    /// let mut http = Request::new("https://www.google.com").unwrap();
    /// http.get().send();
    /// ```
    pub fn send(&mut self) -> Result<Response, HttpError> {
        if let Some(ref proxy) = self.proxy{
            if proxy.scheme == "http".to_owned(){
                let header = self.build_header();
                let addr = format!("{}:{}", proxy.host, proxy.port);
                let mut stream = TcpStream::connect(addr)?;
                stream.set_read_timeout(Some(time::Duration::from_secs(self.timeout)))?;
                stream.set_write_timeout(Some(time::Duration::from_secs(self.timeout)))?;
                stream.write(header.as_bytes())?;
                if let Some(ref body) = self.body{
                    stream.write(body.as_slice())?;
                }
                stream.flush()?;
                let mut res :Vec<u8>= Vec::new();
                stream.read_to_end(&mut res)?;
                let back = Response::new(res)?;
                Ok(back)
            }else {

                //CONNECT proxy.google.com:443 HTTP/1.1
                //Host: www.google.com:443
                //Proxy-Connection: keep-alive
                let mut connect_header = String::new();
                connect_header.push_str(&format!("CONNECT {}:{} HTTP/1.1\r\n",self.host,self.port));
                connect_header.push_str(&format!("Host: {}:{}\r\n",self.host,self.port));
                connect_header.push_str("\r\n");
                let addr = format!("{}:{}", proxy.host, proxy.port);
                let mut stream = TcpStream::connect(addr)?;
                stream.set_read_timeout(Some(time::Duration::from_secs(self.timeout)))?;
                stream.set_write_timeout(Some(time::Duration::from_secs(self.timeout)))?;
                stream.write(connect_header.as_bytes())?;
                stream.flush()?;

                //HTTP/1.1 200 Connection Established
                let mut res = [0u8;1024];
                stream.read(&mut res)?;

                let res_s = match String::from_utf8(res.to_owned().to_vec()){
                    Ok(r) => r,
                    Err(_) => return Err(HttpError::Proxy("parse proxy server response error."))
                };
                if !res_s.to_ascii_lowercase().contains("connection established"){
                    return Err(HttpError::Proxy("Proxy server response error."));
                }

                if self.scheme == "http".to_owned(){
                    let header = self.build_header();
                    stream.write(header.as_bytes())?;
                    if let Some(ref body) = self.body{
                        stream.write(body.as_slice())?;
                    }
                    stream.flush()?;
                    let mut res :Vec<u8>= Vec::new();
                    stream.read_to_end(&mut res)?;
                    let back = Response::new(res)?;
                    Ok(back)

                }else {
                    let connector = TlsConnector::builder()?.build()?;
                    let mut ssl_stream;
                    if self.verify{
                        ssl_stream = connector.connect(&&self.host, stream)?;
                    }else {
                        ssl_stream = connector.danger_connect_without_providing_domain_for_certificate_verification_and_server_name_indication(stream)?;
                    }
                    let header = self.build_header();
                    ssl_stream.write(header.as_bytes())?;
                    if let Some(ref body) = self.body{
                        ssl_stream.write(body.as_slice())?;
                    }
                    ssl_stream.flush()?;
                    let mut res :Vec<u8>= Vec::new();
                    ssl_stream.read_to_end(&mut res)?;
                    let back = Response::new(res)?;
                    Ok(back)
                }

            }

        }else {
            if self.scheme =="http".to_owned(){
                let header = self.build_header();
                let addr = format!("{}:{}", self.host, self.port);
                let mut stream = TcpStream::connect(addr)?;
                stream.set_read_timeout(Some(time::Duration::from_secs(self.timeout)))?;
                stream.set_write_timeout(Some(time::Duration::from_secs(self.timeout)))?;
                stream.write(header.as_bytes())?;
                if let Some(ref body) = self.body{
                    stream.write(body.as_slice())?;
                }
                stream.flush()?;
                let mut res :Vec<u8>= Vec::new();
                stream.read_to_end(&mut res)?;
                let back = Response::new(res)?;
                Ok(back)
            }else {
                let addr = format!("{}:{}", self.host, self.port);
                let stream = TcpStream::connect(addr)?;
                stream.set_read_timeout(Some(time::Duration::from_secs(self.timeout)))?;
                stream.set_write_timeout(Some(time::Duration::from_secs(self.timeout)))?;
                let connector = TlsConnector::builder()?.build()?;
                let mut ssl_stream;
                if self.verify{
                        ssl_stream = connector.connect(&&self.host, stream)?;
                }else {
                        ssl_stream = connector.danger_connect_without_providing_domain_for_certificate_verification_and_server_name_indication(stream)?;
                }
                let header = self.build_header();
                ssl_stream.write(header.as_bytes())?;
                if let Some(ref body) = self.body{
                    ssl_stream.write(body.as_slice())?;
                }
                ssl_stream.flush()?;

                let mut res :Vec<u8>= Vec::new();
                ssl_stream.read_to_end(&mut res)?;
                let back = Response::new(res)?;
                Ok(back)
            }
        }
    }

    //build http request headers
    fn build_header(&self) ->String{
        if let  Some(ref proxy) = self.proxy{
            if proxy.scheme == "http".to_owned(){
                let mut headers = String::new();
                headers.push_str(&format!("{} {} HTTP/1.1\r\n",self.method,self.url.as_string()));
                headers.push_str(&format!("Host: {}:{}\r\n",self.host,self.port));
                headers.push_str(&format!("Connection: Close\r\n"));
                if let Some(ref body) = self.body{
                    headers.push_str(&format!("Content-Length: {}\r\n",body.len()));
                }
                for (i,k) in &self.headers{
                    headers.push_str(&format!("{}: {}\r\n",i,k));
                }
                headers.push_str("\r\n");
                headers
            }else {
                let mut headers = String::new();
                headers.push_str(&format!("{} {} HTTP/1.1\r\n",self.method,self.url.request_string()));
                headers.push_str(&format!("Host: {}:{}\r\n",self.host,self.port));
                headers.push_str(&format!("Connection: Close\r\n"));
                if let Some(ref body) = self.body{
                    headers.push_str(&format!("Content-Length: {}\r\n",body.len()));
                }
                for (i,k) in &self.headers{
                    headers.push_str(&format!("{}: {}\r\n",i,k));
                }
                headers.push_str("\r\n");
                headers
            }
        }else {
                let mut headers = String::new();
                headers.push_str(&format!("{} {} HTTP/1.1\r\n",self.method,self.url.request_string()));
                headers.push_str(&format!("Host: {}:{}\r\n",self.host,self.port));
                headers.push_str(&format!("Connection: Close\r\n"));
                if let Some(ref body) = self.body{
                    headers.push_str(&format!("Content-Length: {}\r\n",body.len()));
                }
                for (i,k) in &self.headers{
                    headers.push_str(&format!("{}: {}\r\n",i,k));
                }
                headers.push_str("\r\n");
                headers
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn https_get() {
        let mut http = Request::new("https://docs.rs/").unwrap();
        println!("{}",http.verify(false).unwrap().get().send().unwrap().status_code())
    }

    #[test]
    fn http_post() {
        let mut http = Request::new("https://docs.rs/").unwrap();
        println!("{}",http.post().body_str("username=bob").send().unwrap().status_code())
    }

    #[test]
    fn http_get_set_header() {
        let mut http = Request::new("https://docs.rs/").unwrap();
        let mut headers = HashMap::new();
        headers.insert("Content-Type","text/html; charset=utf-8");
        println!("{}",http.headers(headers).get().send().unwrap().status_code())
    }

    #[test]
    fn http_get_back_header() {
        let mut http = Request::new("https://docs.rs/").unwrap();
        let headers = http.get().send().unwrap().headers();
        for (k,v) in headers{
            println!("{}:{}",k,v);
        }
    }

    #[test]
    fn http_proxy() {
        let mut http = Request::new("https://docs.rs/").unwrap();
        let res = http.proxy("https://127.0.0.1:1080").unwrap().get().send().unwrap();
        println!("{}",res.status_code());
    }

}