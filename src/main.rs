extern crate hyper;
extern crate hyper_native_tls;
extern crate rustc_serialize;
extern crate rpassword;
extern crate flate2;
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
use rustc_serialize::json::Json;
use flate2::read::GzDecoder;

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
    //head.set(ContentLength(34u64));
    head.set(ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![])));
    head.set(Host{hostname: "s1.zybooks.com".to_owned(), port: None,});
    head.set(Origin::new("https", "zybooks.zyante.com", None));
    head.set(Referer("https://zybooks.zyante.com/".to_owned()));
    head.set(UserAgent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/57.0.2987.133 Safari/537.36".to_owned()));

    // create body
    let info = format!(r#"{{"email":"{}","password":"{}"}}"#, user, pass);

    // make request and get response
    let mut login_res = client.post("https://s1.zybooks.com/v1/signin")
        .body(&*info)
        .headers(head.clone())
        .send()
        .unwrap();

    let mut login_data = String::with_capacity((*login_res.headers.get::<ContentLength>().unwrap()).checked_add(0).unwrap() as usize);
    login_res.read_to_string(&mut login_data).unwrap();


    let login = Json::from_str(&login_data).unwrap();
    let user_id = login.search("user_id").unwrap().as_u64().unwrap();
    let token = login.search("auth_token").unwrap().as_string().unwrap();

    let index = format!(r#"https://s1.zybooks.com/v1/user/{}/zybooks?auth_token={}"#, user_id, token);

    head.remove::<ContentType>();
    head.set(AcceptEncoding(vec![qitem(Encoding::Gzip),
                                 qitem(Encoding::Deflate),
                                 qitem(Encoding::EncodingExt("br".to_owned())),]));

    let mut index_res = client.get(&index)
        .headers(head.clone())
        .send()
        .unwrap();

    let mut index_data: Vec<u8> = Vec::new();
    index_res.read_to_end(&mut index_data).unwrap();
    let mut gz = GzDecoder::new(index_data.as_slice()).unwrap();

    let mut index_dat = String::new();
    gz.read_to_string(&mut index_dat).unwrap();
    // print response
    println!("{}", index_res.status);
    println!("{}", index_res.headers);
    println!("{}", index_dat);
    println!("{}", index);
}
