use std::process::Command;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, hash_map};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct NotificationSettings {
    pub enabled: bool,
    pub show_connection: bool,
    pub show_disconnection: bool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            show_connection: true,
            show_disconnection: true,
        }
    }
}

pub struct NotificationManager {
    settings: Arc<Mutex<NotificationSettings>>,
    active_connections: Arc<Mutex<HashMap<String, Instant>>>,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            settings: Arc::new(Mutex::new(NotificationSettings::default())),
            active_connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn update_settings(&self, settings: NotificationSettings) {
        if let Ok(mut current_settings) = self.settings.lock() {
            *current_settings = settings;
        }
    }

    pub fn get_settings(&self) -> NotificationSettings {
        self.settings.lock().map(|settings| settings.clone()).unwrap_or_default()
    }

    pub fn notify_connection_started(&self, server_name: &str) {
        let settings = self.get_settings();
        if !settings.enabled || !settings.show_connection {
            return;
        }

        if let Ok(mut connections) = self.active_connections.lock() {
            connections.insert(server_name.to_string(), Instant::now());
        }

        self.send_notification(
            "SSH Connection",
            &format!("Connecting to {}...", server_name),
            "network-server",
        );
    }

    pub fn notify_connection_success(&self, server_name: &str) {
        let settings = self.get_settings();
        if !settings.enabled || !settings.show_connection {
            return;
        }

        self.send_notification(
            "SSH Connection Successful",
            &format!("Successfully connected to {}", server_name),
            "network-server",
        );
    }

    pub fn notify_connection_failed(&self, server_name: &str, error: &str) {
        let settings = self.get_settings();
        if !settings.enabled || !settings.show_connection {
            return;
        }

        if let Ok(mut connections) = self.active_connections.lock() {
            connections.remove(server_name);
        }

        self.send_notification(
            "SSH Connection Failed",
            &format!("Unable to connect to {}: {}", server_name, error),
            "network-error",
        );
    }

    pub fn notify_disconnection(&self, server_name: &str) {
        let settings = self.get_settings();
        if !settings.enabled || !settings.show_disconnection {
            return;
        }

        let duration = if let Ok(connections) = self.active_connections.lock() {
            if let Some(start_time) = connections.get(server_name) {
                Some(start_time.elapsed())
            } else {
                None
            }
        } else {
            None
        };

        if let Ok(mut connections) = self.active_connections.lock() {
            connections.remove(server_name);
        }

        let message = if let Some(duration) = duration {
            format!("Disconnected from {} (session: {:.1}s)", server_name, duration.as_secs_f64())
        } else {
            format!("Disconnected from {}", server_name)
        };

        self.send_notification(
            "SSH Disconnection",
            &message,
            "network-offline",
        );
    }

    fn send_notification(&self, title: &str, body: &str, icon: &str) {
        let _ = Command::new("notify-send")
            .args(&[
                "--icon", icon,
                "--app-name", "SSH Config Manager",
                title,
                body,
            ])
            .spawn();
    }

    pub fn cleanup_old_connections(&self, max_age: Duration) {
        if let Ok(mut connections) = self.active_connections.lock() {
            let now = Instant::now();
            connections.retain(|_, start_time| now.duration_since(*start_time) < max_age);
        }
    }
}

lazy_static::lazy_static! {
    pub static ref NOTIFICATION_MANAGER: NotificationManager = NotificationManager::new();
}

pub fn notify_connection_started(server_name: &str) {
    NOTIFICATION_MANAGER.notify_connection_started(server_name);
}

pub fn notify_connection_success(server_name: &str) {
    NOTIFICATION_MANAGER.notify_connection_success(server_name);
}

pub fn notify_connection_failed(server_name: &str, error: &str) {
    NOTIFICATION_MANAGER.notify_connection_failed(server_name, error);
}

pub fn notify_disconnection(server_name: &str) {
    NOTIFICATION_MANAGER.notify_disconnection(server_name);
}

pub fn update_notification_settings(settings: NotificationSettings) {
    NOTIFICATION_MANAGER.update_settings(settings);
}

pub fn get_notification_settings() -> NotificationSettings {
    NOTIFICATION_MANAGER.get_settings()
}
