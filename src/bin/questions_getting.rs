
use tokio::time::Duration;

use std::process::Command;
use std::env;
use std::fmt::format;
use serde_json::Value;
use thirtyfour::{By, DesiredCapabilities, WebDriver, WebElement, WindowHandle};
use thirtyfour::error::{WebDriverError, WebDriverResult};
use tokio::io::AsyncWriteExt;
use crate::chat_gpt_client::get_answers;

const CHROMEDRIVER_BYTES: &[u8] = include_bytes!("chromedriver.exe");


pub async fn initialize_driver_and_browser() -> Result<(), WebDriverError> {

    driver_run().await;
    open_browser();

    let driver = open_driver().await?;
    driver.goto("http://dot.i-bteu.by/").await?;
    driver.quit().await?;
    Ok(())
}

pub async fn get_questions(api_key: &str) -> WebDriverResult<()> {

    let driver = open_driver().await?;
    let script = r#"
        var inputs = document.querySelectorAll('input[id^="q"]');
        for (var i = 0; i < inputs.length; i++) {
            inputs[i].removeAttribute('checked');
        }
    "#;
    driver.execute(script, vec![]).await?;

    let question_elements = driver.find_all(By::ClassName("que")).await?;

    if question_elements.is_empty() {
        interactive_question(&api_key).await?;
    }

    for question_element in question_elements {
        let mut i: i32 = 0;

        // Получаем текст вопроса
        let question_text_element = question_element.find(By::ClassName("qtext")).await?;
        let question_text = question_text_element.text().await?;

        // Получаем уникальный id вопроса для выбора ответа в будущем
        let question_id_element = question_element.find(By::Css("input[name^='q'][name$=':sequencecheck']")).await?;
        let question_id = question_id_element.attr("name").await?;

        // Получение всех ответов на вопрос
        let answer_elements = question_element.find_all(By::Tag("label")).await?;
        let mut answers = String::new();
        let mut question = String::new();
        let mut flag: bool = false;
        let mut sub_flag: bool = false;

        for answer_element in answer_elements {
            let answer_text = answer_element.text().await?;
            answers.push_str(&format!("Ответ {}: {}. ", i, answer_text));

            if i == 0 {
                if let Some(ttt) = answer_element.attr("for").await? {
                    if ttt.contains("choice") {
                        flag = true;
                    } else if ttt.contains("sub") {
                        sub_flag = true;
                    }
                }
            }
            i += 1;

        }

        if sub_flag {
            continue;
        }

        if flag {
            question = format!("В ответ дай мне только цифры наиболее подходящих ответов через запятую (скорее всего их несколько). Мне нужен точный ответ, так что можешь думать над ответом столько времени сколько нужно. Вопрос: {}. {}", question_text, answers);
        } else {
            question = format!("В ответ дай мне только цифру наиболее подходящего ответа. Мне нужен точный ответ, так что можешь думать над ответом столько времени сколько нужно. Вопрос: {}. {}", question_text, answers);
        }

        let pre_result = get_answers(&api_key, &question).await.expect("Ошибка ddsadasads");
        let mut result = String::new();


        if pre_result.contains(",") {

            let arr_answers: Vec<&str> = pre_result.split(',').collect();
            let mut index = 0;
            while index < arr_answers.len() {
                let el = arr_answers[index];
                result = question_id.is_some().to_string().replace(":sequencecheck", "choice") + el;
                let element = driver.find(By::Id(&*result)).await?;
                let script_args = vec![Value::String(element.to_string())];
                driver.execute("arguments[0].setAttribute('checked', 'checked');", script_args).await?;
                index += 1;
            }
        } else {
            result = question_id.expect("Ошибочка небольшая").to_string().replace(":sequencecheck", "answer") + &*pre_result.to_string();

            let js_script = format!(
                r#"
        var element = document.getElementById("{}");
        if (element) {{
            element.setAttribute('checked', 'checked');
        }}
        "#,
                result
            );
            driver.execute(&js_script, vec![]).await?;
        }

        let duration = Duration::from_secs(12);
        tokio::time::sleep(duration).await;
    }
    driver.quit().await?;
    Ok(())
}

async fn driver_run() {
    let temp_dir = env::temp_dir();
    let chromedriver_path = temp_dir.join("chromedriver.exe");
    let mut file = tokio::fs::File::create(&chromedriver_path).await.expect("Failed to create temporary file");
    file.write_all(CHROMEDRIVER_BYTES).await.expect("Failed to write to temporary file");
    tokio::process::Command::new("cmd")
        .args(&["/C", chromedriver_path.to_str().unwrap()])
        .spawn()
        .expect("Ошибка запуска драйвера, скорее всего уже запущен");
}

pub fn open_browser() {
    let command = "start chrome.exe --remote-debugging-port=9222 --user-data-dir=C:\\Windows";
    Command::new("cmd")
        .args(&["/C", command])
        .spawn()
        .expect("Не удалось запустить браузер");
}

async fn interactive_question (api_key: &str) -> WebDriverResult<()> {

    let driver = open_driver().await?;
    let div = driver.find(By::Id("scorm_content")).await?;

    let iframe = div.find(By::Tag("iframe")).await?;

    driver.switch_to().frame_element(&iframe).await?;

    let span = driver.find(By::Css("p[style='text-align:center;'] span")).await?;
    let question_text = span.text().await?;

    let answer_elements = driver.find_all(By::Css("p[style='text-align:left;'] span")).await?;

    let mut i: i32 = 0;
    let mut answers = String::new();
    let mut answers_array = vec![];

    for element in answer_elements {
        answers.push_str(&format!("Ответ {}: {}. ", i, element.text().await?));
        answers_array.push(element.text().await?);
        i += 1;
    }

    let question = format!("Дай мне цифру наиболее подходящего ответа. Мне нужен точный ответ, так что можешь думать над ответом столько времени сколько нужно. Вопрос: {} {}", question_text, answers);
    let result = get_answers(&api_key, &question).await.expect("Ошибка в интерактивном вопросе");
    let result_int : usize = result.parse().expect("Ошибка парсинга");

    let text_to_find = &answers_array[result_int];

    let script = format!(
        "var elements = document.getElementsByTagName('body')[0].getElementsByTagName('*');
         for (var i = 0; i < elements.length; i++) {{
             var element = elements[i];
             if (element.textContent === '{}') {{
                 return element;
             }}
         }}
         return null;",
        text_to_find
    );

    let text_element = driver.execute(&*script, vec![]).await?;
    let answer_element = text_element.element().expect("Ошибка");
    let duration = Duration::from_secs(12);
    tokio::time::sleep(duration).await;
    answer_element.click().await?;
    driver.quit().await?;
    Ok(())
}

async fn open_driver () -> Result<WebDriver, WebDriverError> {

    let mut caps = DesiredCapabilities::chrome();
    caps.add_chrome_arg("--window-size=800,600")?;
    caps.add_chrome_arg("--remote-allow-origins=*")?;
    caps.add_chrome_arg("--disable-extensions")?;
    caps.add_chrome_arg("--disable-dev-shm-usage")?;
    caps.add_chrome_arg("--disable-gpu")?;
    caps.add_chrome_arg("--no-sandbox")?;
    caps.add_chrome_option("debuggerAddress", "localhost:9222")?;
    let driver = WebDriver::new("http://localhost:9515", caps).await?;

    Ok(driver)
}



