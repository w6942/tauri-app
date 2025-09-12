use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use enigo::{Direction, Enigo, Mouse, Button, Coordinate};

// 使用 lazy_static 创建一个线程安全的全局变量来存储录制的位置
// Arc<Mutex<...>> 是一种标准模式，允许多个线程安全地读写数据。
lazy_static::lazy_static! {
    // 录制完成后存储的位置
    static ref RECORDED_POSITION: Arc<Mutex<Option<Position>>> = Arc::new(Mutex::new(None));

    // 【关键修正】在这里添加 IS_REPLAYING 的完整定义
    static ref IS_REPLAYING: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

// 定义一个我们想从前端发送到后端的数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Position {
    x: i32,
    y: i32,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn record() -> Result<(), String> {
    let enigo = Enigo::new(&enigo::Settings::default()).unwrap();
    let (x, y) = enigo.location().unwrap();
    println!("Current mouse position is: ({}, {})", x, y);
    *RECORDED_POSITION.lock().unwrap() = Option::from(Position { x, y });
    Ok(())
}

#[tauri::command]
fn replay(wait_time_ms: u64) -> Result<String, String> {
    println!("Rust: 收到重放请求...");

    {
        let mut is_replaying_guard = IS_REPLAYING.lock().unwrap();
        if *is_replaying_guard {
            return Err("错误：已经在重放中。".into());
        }
        *is_replaying_guard = true;
    }

    let mut enigo = Enigo::new(&enigo::Settings::default()).unwrap();

    let stored_pos_opt = RECORDED_POSITION.lock().unwrap();
    if let Some(target_pos) = &*stored_pos_opt {
        println!("Rust: 在保存的位置 ({}, {}) 开始持续点击...", target_pos.x, target_pos.y);

        let target_pos_clone = target_pos.clone();
        let is_replaying_arc = Arc::clone(&IS_REPLAYING);

        thread::spawn(move || {
            // 首先，移动到目标位置
            enigo.move_mouse(target_pos_clone.x, target_pos_clone.y, Coordinate::Abs).expect("无法移动鼠标");
            thread::sleep(Duration::from_millis(100));

            loop {
                let should_break;
                {
                    // 检查外部停止信号
                    if !*is_replaying_arc.lock().unwrap() {
                        println!("Replay Thread: 收到外部停止信号。");
                        should_break = true;
                    } else {
                        let (x, y) = enigo.location().unwrap();
                        if x != target_pos_clone.x || y != target_pos_clone.y {
                            should_break = true;
                            *IS_REPLAYING.lock().unwrap() = false;
                        } else {
                            should_break = false;
                        }
                    }
                }

                if should_break {
                    break;
                }

                // 如果一切正常，执行点击
                enigo.button(Button::Left, Direction::Click).expect("无法按下鼠标");
                thread::sleep(Duration::from_millis(wait_time_ms));
            }

            // 循环结束后，通知前端
            println!("Rust: 重放循环已停止。");
        });

        Ok("重放已开始，移动鼠标或按 ESC 停止。".into())
    } else {
        *IS_REPLAYING.lock().unwrap() = false;
        Err("错误：请先录制一个位置。".into())
    }
}

#[tauri::command]
fn stop() -> Result<(), String> {
    *IS_REPLAYING.lock().unwrap() = false;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, record, replay, stop])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}