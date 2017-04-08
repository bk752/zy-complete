extern crate hyper;
extern crate hyper_native_tls;
extern crate rustc_serialize;
extern crate rpassword;
#[macro_use]
extern crate language_tags;

use std::io;
use std::io::Read;
use hyper::Client;
use hyper::header::*;
use hyper::mime::*;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use rpassword::read_password;

#[derive(RustcDecodable, RustcEncodable)]
struct Attempt {
    email: String,
    password: String,
}
fn main() {
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = Client::with_connector(connector);

    // get user info
    let mut user = String::new();
    println!("Username:");
    io::stdin().read_line(&mut user).unwrap();
    user = user.trim().to_owned();
    println!("Password:");
    let pass = read_password().unwrap();

    // create post header
    let mut head = Headers::new();
    head.set(Accept(vec![qitem(Mime(TopLevel::Application, SubLevel::Json, vec![])),
                         qitem(Mime(TopLevel::Text, SubLevel::Javascript, vec![])),
                         QualityItem::new(Mime(TopLevel::Star, SubLevel::Star, vec![]), Quality(10))]));
    head.set(AcceptEncoding(vec![qitem(Encoding::Gzip),
                                 qitem(Encoding::Deflate),
                                 qitem(Encoding::EncodingExt("br".to_owned())),]));
    head.set(AcceptLanguage(vec![qitem(langtag!(en;;;US)),
                                 QualityItem::new(langtag!(en), Quality(800)),]));
    head.set(Connection::keep_alive());
    head.set(ContentLength(34u64));
    head.set(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])));
    head.set(Host{hostname: "s1.zybooks.com".to_owned(), port: None,});
    head.set(Origin::new("https", "zybooks.zyante.com", None));
    head.set(Referer("https://zybooks.zyante.com/".to_owned()));
    head.set(UserAgent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/57.0.2987.133 Safari/537.36".to_owned()));

    // create body
    let body = format!(r#"{{"email":"{}","password":"{}"}}"#, user, pass);

    // make request
    let req = client.post("https://s1.zybooks.com/v1/signin")
        .body(&*body)
        .headers(head);
    let res = req.send();
    let mut unwrapped = res.unwrap();
    let mut buff: Vec<u8> = Vec::new();
    unwrapped.read_to_end(&mut buff).unwrap();

    // print response
    println!("{}",unwrapped.status);
    println!("{}",unwrapped.headers);
    println!("{}", String::from_utf8(buff).unwrap());
}
