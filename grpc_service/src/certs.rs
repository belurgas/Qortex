use std::path::PathBuf;

use logging::{log_debug, log_info, logger::setup_logger};
use tokio::fs;

// Получаем путь к папке certs в корне проекта
async fn get_certs_path() -> PathBuf {
    let path = std::env::current_dir().expect("Не могу получить текущий каталог");
    log_debug!("{:?}", path.to_string_lossy());
    return path.parent().unwrap().join("tls");
}

pub async fn load_certs() -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let certs_dir = get_certs_path().await;

    // Диагностические выводы
    log_debug!("Директория certs: {}", certs_dir.display());

    if !certs_dir.exists() {
        panic!("Директория {} не существует!", certs_dir.display());
    }

    let server_crt = certs_dir.join("server/server.crt");
    let server_key = certs_dir.join("server/server.key");
    let ca_cert = certs_dir.join("ca/ca.crt");

    for path in &[&server_crt, &server_key, &ca_cert] {
        if path.exists() {
            log_info!("✅ Файл {} найден", path.display());
        } else {
            log_info!("❌ Файл {} НЕ найден", path.display());
        }
    }

    // Проверяем наличие всех файлов перед чтением
    if !server_crt.exists() {
        panic!("Файл {} не найден", server_crt.display());
    }
    if !server_key.exists() {
        panic!("Файл {} не найден", server_key.display());
    }
    if !ca_cert.exists() {
        panic!("Файл {} не найден", ca_cert.display());
    }

    // Теперь читаем
    let server_cert = fs::read(&server_crt).await.expect("Не могу прочитать server.crt");
    let server_key = fs::read(&server_key).await.expect("Не могу прочитать server.key");
    let ca_cert = fs::read(&ca_cert).await.expect("Не могу прочитать ca.cert");

    log_info!("Все файлы успешно загружены!");

    (server_cert, server_key, ca_cert)
}