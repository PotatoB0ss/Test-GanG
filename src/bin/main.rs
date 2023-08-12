use hotkey;
use std::thread;
use std::process;
use std::ptr;
use winapi::um::synchapi::CreateMutexA;
use winapi::um::handleapi::CloseHandle;
use std::ffi::CString;
use std::panic;
use std::time::Duration;
use tokio::time::sleep;
use crate::db_work::{initialize_key, key_deactivate_request, key_update_request};
use crate::questions_getting::{get_questions, initialize_driver_and_browser, open_browser};


mod chat_gpt_client;
mod db_work;
mod questions_getting;


#[macro_use]
extern crate self_update;

#[derive(Clone)]
struct Key {
    key_value: String
}

impl Key {
    fn new(initial_value: String) -> Self {
        Key { key_value: initial_value }
    }

    fn get_value(&self) -> &String {
        &self.key_value
    }

}

#[tokio::main]
async fn main() {



    // Протект чтобы не могли открыть приложение больше 1 раза
    // В случае повторного открытия заново откроет браузер в дебаг моде
    let mutex_name = CString::new("TestHelper").expect("CString::new failed");

    unsafe {
        let mutex_handle = CreateMutexA(
            ptr::null_mut(),
            0,
            mutex_name.as_ptr(),
        );

        if mutex_handle == ptr::null_mut() {
            panic!("Ошибка создания Mutex");
        }

        if winapi::um::errhandlingapi::GetLastError() == 183 {
            println!("Приложение уже запущено");
            open_browser();
            CloseHandle(mutex_handle);
            return;
        }

        if let Err(e) = run(){
            println!("[ERROR] {}", e);
            ::std::process::exit(1);
        }

        let api_key: String = initialize_key().await.expect("Ошибка получения ключа");
        initialize_driver_and_browser().await.expect("Ошибка инициализации браузера или драйвера");


        let key = Key::new(api_key);
        let key_clone = key.clone();
        let key_clone2 = key.clone();

        let mut app;
        match systray2::Application::new() {
            Ok(w) => app = w,
            Err(_) => panic!("Не удалось создать окно!"),
        }

        app.add_menu_item("Закрыть", move|_window| {
            println!("О нет вы закрыли приложение");
            key_deactivate_request(key.get_value()).expect("Ошибка деактивации ключа");
            process::exit(0);
            Ok::<_, systray2::Error>(())
        }).expect("Ошибка при выходе");

        let hk_handle = thread::spawn(move || {
            let mut hk = hotkey::Listener::new();
            hk.register_hotkey(0, 'E' as u32, move || e_button(&key_clone2.key_value)).unwrap();
            hk.register_hotkey(0, 'R' as u32, || r_button()).unwrap();
            hk.listen();
        });

        let a = key_clone.key_value;
        tokio::spawn(async move {
            key_check(&a).await;
        });


        app.wait_for_message().expect("Ошибка в запуске трея");
        hk_handle.join().unwrap();
        CloseHandle(mutex_handle);
    }
}


fn e_button(key: &String) {

    let questions_future = get_questions("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");
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    runtime.block_on(async {
        if let Err(err) = questions_future.await {
            eprintln!("Ошибка: {:?}", err);
        }
    });
}
// http://dot.i-bteu.by/mod/quiz/attempt.php?attempt=196168
fn r_button() {
    println!("R PRESSED");
}


async fn key_check(value: &String) {
    loop {
        key_update_request(value).expect("Ошибка обновления ключа");
        sleep(Duration::from_secs(10)).await;
    }
}

fn run() -> Result<(), Box<dyn ::std::error::Error>> {
    self_update::backends::github::ReleaseList::configure()
        .repo_owner("PotatoB0ss")
        .repo_name("self_update")
        .build()?
        .fetch()?;

    self_update::backends::github::Update::configure()
        .repo_owner("PotatoB0ss")
        .repo_name("self_update")
        .bin_name("github")
        .show_download_progress(true)
        .show_output(false)
        .no_confirm(false)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;
    Ok(())
}






