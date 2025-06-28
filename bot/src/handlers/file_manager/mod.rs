use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use tokio::fs;
use teloxide::{prelude::*, types::InputFile};

// Структура для управления файлами
pub struct FileManager {
    storage_path: PathBuf,
    share_links: Mutex<HashMap<String, SharedFile>>,
}

#[derive(Clone)]
struct SharedFile {
    owner_id: i64,
    file_path: PathBuf,
    file_name: String,
}

impl FileManager {
    // Создает новый экземпляр менеджера
    pub async fn new(base_path: impl AsRef<Path>) -> std::io::Result<Self> {
        let storage_path = base_path.as_ref().to_path_buf();
        fs::create_dir_all(&storage_path).await?;
        
        Ok(Self {
            storage_path,
            share_links: Mutex::new(HashMap::new()),
        })
    }

    // Создает папку для пользователя
    pub async fn create_user_dir(&self, user_id: i64) -> std::io::Result<PathBuf> {
        let user_dir = self.storage_path.join(user_id.to_string());
        if !user_dir.exists() {
            fs::create_dir(&user_dir).await?;
        }
        Ok(user_dir)
    }

    // Сохраняет файл пользователя
    pub async fn save_user_file(
        &self, 
        user_id: i64, 
        file_name: &str, 
        content: &[u8]
    ) -> std::io::Result<PathBuf> {
        let user_dir = self.create_user_dir(user_id).await?;
        let file_path = user_dir.join(file_name);
        fs::write(&file_path, content).await?;
        Ok(file_path)
    }

    // Генерирует ссылку для делегирования
    pub async fn generate_share_link(
        &self, 
        owner_id: i64, 
        file_name: &str
    ) -> std::io::Result<String> {
        let token = Uuid::new_v4().to_string();
        let user_dir = self.create_user_dir(owner_id).await?;
        let file_path = user_dir.join(file_name);
        
        if file_path.exists() {
            self.share_links.lock().await.insert(
                token.clone(),
                SharedFile {
                    owner_id,
                    file_path,
                    file_name: file_name.to_string(),
                }
            );
            
            Ok(format!("https://yourdomain.com/share/ {}", token))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound, 
                "File not found"
            ))
        }
    }

    // Обрабатывает запрос по ссылке
    pub async fn handle_shared_link(
        &self, 
        token: &str,
        bot: &Bot,
        chat_id: ChatId
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(shared) = self.share_links.lock().await.get(token) {
            let file = InputFile::file(&shared.file_path);
            bot.send_document(chat_id, file)
                .caption(format!("Файл от пользователя {}", shared.owner_id))
                .await?;
            Ok(())
        } else {
            Err("Ссылка недействительна или истекло время действия".into())
        }
    }

    // Очищает устаревшие ссылки (можно запускать периодически)
    pub async fn cleanup_expired_links(&self) {
        self.share_links.lock().await.retain(|_, _| {
            // Здесь можно добавить логику проверки времени жизни ссылок
            true // Пока оставляем все ссылки
        });
    }
}