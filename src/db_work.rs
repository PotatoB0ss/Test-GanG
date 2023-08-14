use native_dialog::{MessageDialog, MessageType};
use std::process;
use tokio_postgres::{NoTls, Error};


pub async fn initialize_key() -> Result<String, Error> {

    let (client, connection) =
        tokio_postgres::connect("postgresql://postgres:0Rb176zroxsVcToEWFTe@containers-us-west-49.railway.app:7098/railway", NoTls).await?;


    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Ошибка подключения к базе данных: {}", e);
        }
    });


    let mut _key_value: String = String::new();

    // Если нету свободных ключей то говорим юзеру что приложение использует много людей и просим подождать.
    if let Some(row) = client.query_opt("SELECT id, gpt_key, is_using FROM gpt_keys WHERE is_using is false LIMIT 1", &[]).await? {
        _key_value = row.get(1);
        key_request(&_key_value).expect("Ошибка при передаче ключа");
    } else {
        overload();
    }

    Ok(_key_value)
}

fn overload () {
    MessageDialog::new()
        .set_title("Ошибка")
        .set_text(&format!("Приложение использует много людей.\nПодождите пока кто-то закончит использование."))
        .set_type(MessageType::Warning)
        .show_alert().expect("Ошибка при открытии диалогово окна");
    process::exit(0);
}

fn key_request (key: &String) -> Result<(), ureq::Error> {
    let resp: String = ureq::post("http://localhost:8080/first-accept")
        .set("content-type", "application/json")
        .send_json(ureq::json!({
               "key": key,
           }))?
        .into_string()?;

    if resp != "200" {
        process::exit(0);
    }
    Ok(())
}

pub fn key_update_request (key: &String) -> Result<(), ureq::Error> {
     let _resp: String = ureq::post("http://localhost:8080/key-check")
         .set("content-type", "application/json")
         .send_json(ureq::json!({
                "key": key,
            }))?
         .into_string()?;
    Ok(())
}

pub fn key_deactivate_request (key: &String) -> Result<(), ureq::Error> {
    let _resp: String = ureq::post("http://localhost:8080/deactivate")
        .set("content-type", "application/json")
        .send_json(ureq::json!({
               "key": key,
           }))?
        .into_string()?;
    Ok(())
}



