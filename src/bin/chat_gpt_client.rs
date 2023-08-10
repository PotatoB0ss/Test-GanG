use serde_json::{json, Value};


pub async fn get_answers(api_key: &str, question: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    println!("THE QUESTION IS {}", &question);
    let resp = client
        .post("https://api.openai.com/v1/engines/text-davinci-003/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "prompt": &question,
            "max_tokens": 50,
            "temperature": 0.2
        }))
        .send()
        .await?
        .text()
        .await?;

    let parsed_response: Value = serde_json::from_str(&resp).expect("Ошибка");

    println!("{resp}");
    let mut response = String::new();

    if let Some(text) = parsed_response["choices"][0]["text"].as_str() {
        response = text.to_string();
    }

    response = response.replace("\n", "");
    response = response.replace("\n\n", "");
    response = response.replace(" ", "");

    Ok(response)
}



