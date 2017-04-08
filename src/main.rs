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

fn login_user(user: String, pass: String, client: &Client, head: Headers) -> (u64, String) {
    // create body
    let info = format!(r#"{{"email":"{}","password":"{}"}}"#, user, pass);

    // make request and get response
    let mut res = client.post("https://s1.zybooks.com/v1/signin")
        .body(&*info)
        .headers(head.clone())
        .send()
        .unwrap();

    let mut data = String::new();
    res.read_to_string(&mut data).unwrap();


    let login = Json::from_str(&data).unwrap();
    let user_id = login.search("user_id").unwrap().as_u64().unwrap();
    let token = login.search("auth_token").unwrap().as_string().unwrap().to_owned();

    (user_id, token)
}

fn get_books(user_id: u64, token: &str, client: &Client, head: Headers) -> Vec<String> {

    let url = format!(r#"https://s1.zybooks.com/v1/user/{}/zybooks?auth_token={}"#, user_id, token);

    let mut res = client.get(&url)
        .headers(head.clone())
        .send()
        .unwrap();

    let mut raw: Vec<u8> = Vec::new();
    res.read_to_end(&mut raw).unwrap();
    let mut gz = GzDecoder::new(raw.as_slice()).unwrap();

    let mut data = String::new();
    gz.read_to_string(&mut data).unwrap();

    let index = Json::from_str(&data).unwrap();
    let book_list = index.search("zybooks").unwrap().as_array().unwrap();

    let mut books: Vec<String> = Vec::new();


    for book in book_list {
        books.push(book.search("zybook_code").unwrap().as_string().unwrap().to_owned());
    }

    books
}

fn get_questions(user_id: u64, token: &str, book_code: &str, client: &Client, head: Headers) -> Vec<(String, i64)> {

    let url = format!(r#"https://s1.zybooks.com/v1/zybook/{}/activities/{}?auth_token={}"#, book_code, user_id, token);

    let mut res = client.get(&url)
        .headers(head.clone())
        .send()
        .unwrap();

    let mut raw: Vec<u8> = Vec::new();
    res.read_to_end(&mut raw).unwrap();
    let mut gz = GzDecoder::new(raw.as_slice()).unwrap();

    let mut data = String::new();
    gz.read_to_string(&mut data).unwrap();

    let info = Json::from_str(&data).unwrap();
    // print response
    println!("{}", data);

    let questions: Vec<(String, i64)> =  Vec::new();
    questions
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

    // TODO: figure out way of calling without clone
    let user_data = login_user(user, pass, &client, head.clone());

    head.remove::<ContentType>();
    head.set(AcceptEncoding(vec![qitem(Encoding::Gzip),
                                 qitem(Encoding::Deflate),
                                 qitem(Encoding::EncodingExt("br".to_owned())),]));

    let books = get_books(user_data.0, &user_data.1, &client, head.clone());
    for (i, book) in books.iter().enumerate() {
        println!("{}) {}", i+1, book);
    }
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    let choice_val = choice.trim().parse::<usize>().unwrap();
    get_questions(user_data.0, &user_data.1, &books[choice_val-1], &client, head.clone());
}
